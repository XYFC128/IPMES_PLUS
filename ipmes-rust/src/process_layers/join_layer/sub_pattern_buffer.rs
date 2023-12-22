mod tests;

use crate::sub_pattern::SubPattern;
use crate::sub_pattern_match::EarliestFirst;
use std::cmp::max;

use crate::match_event::MatchEvent;
use crate::pattern::Pattern;
use itertools::Itertools;
use petgraph::adj::DefaultIx;
use petgraph::graph::NodeIndex;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;
use log::debug;
use crate::process_layers::join_layer::sub_pattern_buffer::TimeOrder::{FirstToSecond, SecondToFirst};

#[derive(Clone, Debug)]
enum TimeOrder {
    FirstToSecond,
    SecondToFirst,
}

#[derive(Clone, Debug)]
pub struct Relation {
    /// shared_entities.len() == num_node
    /// If node 'i' is shared, shared_entities[i] = true.
    /// 'i': pattern node id
    /// "shared_entities" seems useless (?)
    /// (The "structure" has guaranteed nodes to be shared properly, when doing "SubPatternMatch::try_merge_nodes()".)
    shared_entities: Vec<bool>,

    /// event_orders: (pattern_id1, pattern_id2, TimeOrder)
    /// "pattern_id1" comes from "sub_pattern1", which is the left part ("sub_pattern2" is the right part)
    /// left and right is the "relative position" on the "sub_pattern_buffer tree"
    event_orders: Vec<(usize, usize, TimeOrder)>,
}

impl Relation {
    pub fn new() -> Self {
        Self {
            shared_entities: Vec::new(),
            event_orders: Vec::new(),
        }
    }

    pub fn check_order_relation(
        &self,
        timestamps: &Vec<u64>,
    ) -> bool {
        self.event_orders
            .iter()
            .all(|(pattern_id1, pattern_id2, time_order)| {
                match time_order {
                    FirstToSecond => timestamps[*pattern_id1] <= timestamps[*pattern_id2],
                    SecondToFirst => timestamps[*pattern_id1] >= timestamps[*pattern_id2],
                }
            })
    }

    pub fn is_entity_shared(&self, id: usize) -> bool {
        self.shared_entities[id]
    }
}

#[derive(Clone, Debug)]
pub struct SubPatternBuffer<'p> {
    pub id: usize,
    sibling_id: usize,
    /// IDs of pattern nodes contained in this sub-pattern.
    node_id_list: HashSet<usize>,
    /// IDs of pattern events contained in this sub-pattern.
    edge_id_list: HashSet<usize>,
    pub(crate) buffer: BinaryHeap<EarliestFirst<'p>>,
    pub(crate) new_match_buffer: BinaryHeap<EarliestFirst<'p>>,

    pub relation: Relation,
    /// number of nodes in the "whole" pattern
    pub max_num_entities: usize,
    pub max_num_events: usize
}

impl<'p> SubPatternBuffer<'p> {
    pub fn new(
        id: usize,
        sibling_id: usize,
        sub_pattern: &SubPattern,
        max_num_entities: usize,
        max_num_events: usize,
    ) -> Self {
        let mut node_id_list = HashSet::new();
        let mut edge_id_list = HashSet::new();
        for &edge in &sub_pattern.events {
            node_id_list.insert(edge.subject);
            node_id_list.insert(edge.object);
            edge_id_list.insert(edge.id);
        }
        Self {
            id,
            sibling_id,
            node_id_list,
            edge_id_list,
            buffer: BinaryHeap::new(),
            new_match_buffer: BinaryHeap::new(),
            relation: Relation::new(),
            max_num_entities,
            max_num_events
        }
    }

    pub fn generate_relations(
        pattern: &Pattern,
        sub_pattern_buffer1: &SubPatternBuffer,
        sub_pattern_buffer2: &SubPatternBuffer,
        // distances_table: &HashMap<(NodeIndex, NodeIndex), i32>,
    ) -> Relation {
        let mut shared_entities = vec![false; pattern.num_entities];
        let mut event_orders = Vec::new();

        // identify shared nodes
        for i in 0..pattern.num_entities {
            if sub_pattern_buffer1.node_id_list.contains(&i)
                && sub_pattern_buffer2.node_id_list.contains(&i)
            {
                shared_entities[i] = true;
            }
        }

        // generate order-relation
        for eid1 in &sub_pattern_buffer1.edge_id_list {
            for eid2 in &sub_pattern_buffer2.edge_id_list {
                let distance_1_2 = pattern.order.get_distance(eid1, eid2);
                let distance_2_1 = pattern.order.get_distance(eid2, eid1);

                // "2" is "1"'s parent
                if distance_1_2 == i32::MAX && distance_2_1 != i32::MAX {
                    event_orders.push((*eid1, *eid2, SecondToFirst));
                } else if distance_1_2 != i32::MAX && distance_2_1 == i32::MAX {
                    event_orders.push((*eid1, *eid2, FirstToSecond));
                }
            }
        }

        Relation {
            shared_entities,
            event_orders,
        }
    }

    pub fn merge_buffers(
        sub_pattern_buffer1: &SubPatternBuffer,
        sub_pattern_buffer2: &SubPatternBuffer,
    ) -> Self {
        let mut node_id_list = sub_pattern_buffer1.node_id_list.clone();
        let mut edge_id_list = sub_pattern_buffer1.edge_id_list.clone();
        node_id_list.extend(&sub_pattern_buffer2.node_id_list);
        edge_id_list.extend(&sub_pattern_buffer2.edge_id_list);

        let id = max(sub_pattern_buffer1.id, sub_pattern_buffer2.id) + 1;

        Self {
            id,
            sibling_id: id + 1,
            node_id_list,
            edge_id_list,
            buffer: BinaryHeap::new(),
            new_match_buffer: BinaryHeap::new(),
            relation: Relation::new(),
            max_num_entities: sub_pattern_buffer1.max_num_entities,
            max_num_events: sub_pattern_buffer1.max_num_events
        }
    }

    /// "merge match_edge" and "check edge uniqueness"
    /// Analogous to "try_merge_nodes"
    ///
    /// Since "check_edge_uniqueness" guarantees the bijective relationship between
    /// pattern events and input events, the index of "timestamps" can be "pattern edge id".
    ///
    /// All pattern events are unique.
    pub fn try_merge_match_events(
        &self,
        a: &[MatchEvent<'p>],
        b: &[MatchEvent<'p>],
    ) -> Option<(Vec<MatchEvent<'p>>, Vec<u64>)> {
        let mut timestamps = vec![0u64; self.max_num_events];
        let mut merged = Vec::with_capacity(a.len() + b.len());

        let mut p1 = a.iter();
        let mut p2 = b.iter();
        let mut next1 = p1.next();
        let mut next2 = p2.next();

        while let (Some(edge1), Some(edge2)) = (next1, next2) {
            if edge1.input_event.id < edge2.input_event.id {
                merged.push(edge1.clone());
                timestamps[edge1.matched.id] = edge1.input_event.timestamp;
                next1 = p1.next();
            } else if edge1.input_event.id > edge2.input_event.id {
                merged.push(edge2.clone());
                timestamps[edge2.matched.id] = edge2.input_event.timestamp;
                next2 = p2.next();
            } else {
                if edge1.matched.id != edge2.matched.id {
                    debug!("pattern edge not shared!");
                    return None;
                }
                merged.push(edge1.clone());
                timestamps[edge1.matched.id] = edge1.input_event.timestamp;
                next1 = p1.next();
                next2 = p2.next();
            }
        }

        if next1.is_none() {
            p1 = p2;
            next1 = next2;
        }

        while let Some(edge) = next1 {
            timestamps[edge.matched.id] = edge.input_event.timestamp;
            merged.push(edge.clone());
            next1 = p1.next();
        }

        Some((merged, timestamps))
    }

    /// Try to merge match nodes, and handle "shared node" and "node uniqueness" in the process.
    /// If the mentioned checks didn't pass, return None.
    ///
    /// a and b are slices over (input node id, pattern node id)
    pub fn try_merge_entities(
        &self,
        a: &[(u64, u64)],
        b: &[(u64, u64)],
    ) -> Option<Vec<(u64, u64)>> {
        let mut used_nodes = vec![false; self.max_num_entities];
        let mut merged = Vec::with_capacity(a.len() + b.len());

        let mut p1 = a.iter();
        let mut p2 = b.iter();

        let mut next1 = p1.next();
        let mut next2 = p2.next();

        while let (Some(node1), Some(node2)) = (next1, next2) {
            if used_nodes[node1.1 as usize] || used_nodes[node2.1 as usize] {
                return None;
            }

            if node1.0 < node2.0 {
                merged.push(*node1);
                used_nodes[node1.1 as usize] = true;
                next1 = p1.next();
            } else if node1.0 > node2.0 {
                merged.push(*node2);
                used_nodes[node2.1 as usize] = true;
                next2 = p2.next();
            } else {
                if node1.1 != node2.1 {
                    return None;
                }
                merged.push(*node1);
                used_nodes[node1.1 as usize] = true;
                next1 = p1.next();
                next2 = p2.next();
            }
        }

        if next1.is_none() {
            p1 = p2;
            next1 = next2;
        }

        while let Some(node) = next1 {
            if used_nodes[node.1 as usize] {
                return None;
            }
            used_nodes[node.1 as usize] = true;
            merged.push(*node);
            next1 = p1.next();
        }

        Some(merged)
    }
}