mod tests;

use crate::sub_pattern::SubPattern;
use crate::sub_pattern_match::{EarliestFirst, SubPatternMatch};
use std::cmp::{max, min};

use crate::match_edge::MatchEdge;
use crate::pattern::Pattern;
use itertools::Itertools;
use petgraph::adj::DefaultIx;
use petgraph::graph::NodeIndex;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;
use crate::process_layers::join_layer::sub_pattern_buffer::TimeOrder::{FirstToSecond, SecondToFirst};

#[derive(Clone)]
enum TimeOrder {
    FirstToSecond,
    SecondToFirst,
}

#[derive(Clone)]
pub struct Relation {
    // shared_nodes.len() == num_node
    // If node 'i' is shared, shared_nodes[i] = true.
    // 'i': pattern node id
    /// "shared_nodes" seems useless.
    /// (The "structure" has guaranteed nodes to be shared properly, when doing "SubPatternMatch::try_merge_nodes()".)
    shared_nodes: Vec<bool>,

    /// edge_orders: (pattern_id1, pattern_id2, TimeOrder)
    /// "pattern_id1" comes from "sub_pattern1", which is the left part ("sub_pattern2" is the right part)
    /// left and right is the "relative position" on the "sub_pattern_buffer tree"
    edge_orders: Vec<(usize, usize, TimeOrder)>,
}

impl Relation {
    pub fn new() -> Self {
        Self {
            shared_nodes: Vec::new(),
            edge_orders: Vec::new(),
        }
    }

    pub fn check_order_relation(
        &self,
        timestamps: &Vec<u64>,
    ) -> bool {
        self.edge_orders
            .iter()
            .all(|(pattern_id1, pattern_id2, time_order)| {
                match time_order {
                    FirstToSecond => timestamps[*pattern_id1] <= timestamps[*pattern_id2],
                    SecondToFirst => timestamps[*pattern_id1] >= timestamps[*pattern_id2],
                }
            })
    }

    pub fn is_node_shared(&self, id: usize) -> bool {
        self.shared_nodes[id]
    }
}

#[derive(Clone)]
pub struct SubPatternBuffer<'p> {
    pub id: usize,
    sibling_id: usize,
    /// List of ids of pattern nodes contained in this sub-pattern.
    node_id_list: HashSet<usize>,
    /// List of ids of pattern edges contained in this sub-pattern.
    edge_id_list: HashSet<usize>,
    pub(crate) buffer: BinaryHeap<EarliestFirst<'p>>,
    pub(crate) new_match_buffer: BinaryHeap<EarliestFirst<'p>>,

    pub relation: Relation,
    /// number of nodes in the "whole" pattern
    pub max_num_nodes: usize,

    /// make sure the largest pattern edge id is "pattern.edges.len() - 1"
    pub used_nodes: Vec<bool>,
    /// '0' means timestamp not recorded
    pub timestamps: Vec<u64>,
}

impl<'p> SubPatternBuffer<'p> {
    pub fn new(
        id: usize,
        sibling_id: usize,
        sub_pattern: &SubPattern,
        max_num_nodes: usize,
        max_num_edges: usize,
    ) -> Self {
        let mut node_id_list = HashSet::new();
        let mut edge_id_list = HashSet::new();
        for &edge in &sub_pattern.edges {
            node_id_list.insert(edge.start);
            node_id_list.insert(edge.end);
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
            max_num_nodes,
            used_nodes: vec![false; max_num_nodes],
            timestamps: vec![0; max_num_edges]
        }
    }

    pub fn generate_relations(
        // &mut self,
        pattern: &Pattern,
        sub_pattern_buffer1: &SubPatternBuffer,
        sub_pattern_buffer2: &SubPatternBuffer,
        distances_table: &HashMap<(NodeIndex, NodeIndex), i32>,
    ) -> Relation {
        let mut shared_nodes = vec![false; pattern.num_nodes];
        let mut edge_orders = Vec::new();

        // identify shared nodes
        for i in 0..pattern.num_nodes {
            if sub_pattern_buffer1.node_id_list.contains(&i)
                && sub_pattern_buffer2.node_id_list.contains(&i)
            {
                shared_nodes[i] = true;
            }
        }

        // generate order-relation
        for eid1 in &sub_pattern_buffer1.edge_id_list {
            for eid2 in &sub_pattern_buffer2.edge_id_list {
                let id1 = NodeIndex::<DefaultIx>::new(eid1.clone());
                let id2 = NodeIndex::<DefaultIx>::new(eid2.clone());
                let distance_1_2 = distances_table.get(&(id1, id2)).unwrap();
                let distance_2_1 = distances_table.get(&(id2, id1)).unwrap();

                // "2" is "1"'s parent
                if distance_1_2 == &i32::MAX && distance_2_1 != &i32::MAX {
                    edge_orders.push((eid1.clone(), eid2.clone(), SecondToFirst));
                } else if distance_1_2 != &i32::MAX && distance_2_1 == &i32::MAX {
                    edge_orders.push((eid1.clone(), eid2.clone(), FirstToSecond));
                }
            }
        }

        Relation {
            shared_nodes,
            edge_orders,
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
            max_num_nodes: sub_pattern_buffer1.max_num_nodes,
            used_nodes: vec![false; sub_pattern_buffer1.max_num_nodes],
            timestamps: vec![0; sub_pattern_buffer1.timestamps.len()]
        }
    }

    pub fn clear_workspace(&mut self) {
        self.used_nodes.fill(false);
        self.timestamps.fill(0);
    }

    /// "merge match_edge" and "check edge uniqueness"
    /// Analogous to "try_merge_nodes"
    ///
    /// Since "check_edge_uniqueness" guarantees the bijective relationship between
    /// pattern edges and input edges, the index of "timestamps" can be "pattern edge id".
    ///
    /// All pattern edges are unique.
    pub fn try_merge_match_edges(
        &mut self,
        a: &[MatchEdge<'p>],
        b: &[MatchEdge<'p>],
    ) -> Option<Vec<MatchEdge<'p>>> {
        let mut merged = Vec::with_capacity(a.len() + b.len());

        let mut p1 = a.iter();
        let mut p2 = b.iter();
        let mut next1 = p1.next();
        let mut next2 = p2.next();

        while let (Some(edge1), Some(edge2)) = (next1, next2) {
            if edge1.input_edge.id < edge2.input_edge.id {
                merged.push(edge1.clone());
                self.timestamps[edge1.matched.id] = edge1.input_edge.timestamp;
                next1 = p1.next();
            } else if edge1.input_edge.id > edge2.input_edge.id {
                merged.push(edge2.clone());
                self.timestamps[edge2.matched.id] = edge2.input_edge.timestamp;
                next2 = p2.next();
            } else {
                if edge1.matched.id != edge2.matched.id {
                    println!("pattern edge not shared!");
                    return None;
                }
                merged.push(edge1.clone());
                self.timestamps[edge1.matched.id] = edge1.input_edge.timestamp;
                next1 = p1.next();
                next2 = p2.next();
            }
        }

        if next1.is_none() {
            p1 = p2;
            next1 = next2;
        }

        while let Some(edge) = next1 {
            self.timestamps[edge.matched.id] = edge.input_edge.timestamp;
            merged.push(edge.clone());
            next1 = p1.next();
        }

        Some(merged)
    }

    /// Try to merge match nodes, and handle "shared node" and "node uniqueness" in the process.
    /// If the mentioned checks didn't pass, return None.
    ///
    /// a and b are slices over (input node id, pattern node id)
    pub fn try_merge_nodes(
        &mut self,
        a: &[(u64, u64)],
        b: &[(u64, u64)],
    ) -> Option<Vec<(u64, u64)>> {
        let mut merged = Vec::with_capacity(a.len() + b.len());

        let mut p1 = a.iter();
        let mut p2 = b.iter();
        let mut next1 = p1.next();
        let mut next2 = p2.next();
        while let (Some(node1), Some(node2)) = (next1, next2) {
            if self.used_nodes[node1.1 as usize] || self.used_nodes[node2.1 as usize] {
                return None;
            }

            if node1.0 < node2.0 {
                merged.push(node1.clone());
                self.used_nodes[node1.1 as usize] = true;
                next1 = p1.next();
            } else if node1.0 > node2.0 {
                merged.push(node2.clone());
                self.used_nodes[node2.1 as usize] = true;
                next2 = p2.next();
            } else {
                if node1.1 != node2.1 {
                    return None;
                }
                merged.push(node1.clone());
                self.used_nodes[node1.1 as usize] = true;
                next1 = p1.next();
                next2 = p2.next();
            }
        }

        if next1.is_none() {
            p1 = p2;
            next1 = next2;
        }

        while let Some(node) = next1 {
            if self.used_nodes[node.1 as usize] {
                return None;
            }
            self.used_nodes[node.1 as usize] = true;
            merged.push(node.clone());
            next1 = p1.next();
        }

        Some(merged)
    }
}