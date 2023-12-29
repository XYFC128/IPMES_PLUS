use crate::sub_pattern::SubPattern;
use crate::sub_pattern_match::EarliestFirst;
use std::cmp::max;

use crate::match_event::MatchEvent;
use crate::pattern::Pattern;
use crate::process_layers::join_layer::sub_pattern_buffer::TimeOrder::{
    FirstToSecond, SecondToFirst,
};
use log::debug;
use std::collections::{BinaryHeap, HashSet};

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

    pub fn check_order_relation(&self, timestamps: &Vec<u64>) -> bool {
        self.event_orders
            .iter()
            .all(|(pattern_id1, pattern_id2, time_order)| match time_order {
                FirstToSecond => timestamps[*pattern_id1] <= timestamps[*pattern_id2],
                SecondToFirst => timestamps[*pattern_id1] >= timestamps[*pattern_id2],
            })
    }

    pub fn is_entity_shared(&self, id: usize) -> bool {
        self.shared_entities[id]
    }
}

#[derive(Clone, Debug)]
pub struct SubPatternBuffer<'p> {
    pub id: usize,
    /// mainly for debugging
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
    pub max_num_events: usize,
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
            max_num_events,
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

        // dummy code (no use)
        sub_pattern_buffer1.sibling_id;

        Self {
            id,
            sibling_id: id + 1,
            node_id_list,
            edge_id_list,
            buffer: BinaryHeap::new(),
            new_match_buffer: BinaryHeap::new(),
            relation: Relation::new(),
            max_num_entities: sub_pattern_buffer1.max_num_entities,
            max_num_events: sub_pattern_buffer1.max_num_events,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_event::InputEvent;
    use crate::pattern::Event;
    use std::rc::Rc;
    #[test]
    /// shared node not shared between input nodes: Fail
    fn test_try_merge_nodes1() {
        let max_node_id = 100;
        let tmp_sub_pattern = SubPattern {
            id: 0,
            events: vec![],
        };
        let sub_pattern_buffer = SubPatternBuffer::new(0, 0, &tmp_sub_pattern, max_node_id, 0);
    
        let a = vec![(2, 19), (7, 20), (11, 9)];
    
        let b = vec![(0, 17), (2, 22), (9, 11)];
    
        let ans = None;
    
        let merged = sub_pattern_buffer.try_merge_entities(&a, &b);
        assert_eq!(merged, ans);
    }
    
    #[test]
    /// input node not unique: Fail
    fn test_try_merge_nodes2() {
        let max_node_id = 100;
        let tmp_sub_pattern = SubPattern {
            id: 0,
            events: vec![],
        };
        let sub_pattern_buffer = SubPatternBuffer::new(0, 0, &tmp_sub_pattern, max_node_id, 0);
    
        let a = vec![(2, 19), (7, 20), (11, 9)];
    
        let b = vec![(0, 17), (25, 20)];
        let ans = None;
    
        let merged = sub_pattern_buffer.try_merge_entities(&a, &b);
        assert_eq!(merged, ans);
    }
    
    #[test]
    /// Pass ("a" finished first)
    fn test_try_merge_nodes3() {
        let max_node_id = 100;
        let tmp_sub_pattern = SubPattern {
            id: 0,
            events: vec![],
        };
        let sub_pattern_buffer = SubPatternBuffer::new(0, 0, &tmp_sub_pattern, max_node_id, 0);
    
        let a = vec![(2, 19), (7, 20), (11, 9)];
    
        let b = vec![(0, 17), (25, 27)];
        let ans = vec![(0, 17), (2, 19), (7, 20), (11, 9), (25, 27)];
    
        let merged = sub_pattern_buffer.try_merge_entities(&a, &b);
        assert_ne!(merged, None);
        assert_eq!(merged.unwrap(), ans);
    }
    
    #[test]
    /// Pass ("a" finished first)
    fn test_try_merge_nodes4() {
        let max_node_id = 100;
        let tmp_sub_pattern = SubPattern {
            id: 0,
            events: vec![],
        };
        let sub_pattern_buffer = SubPatternBuffer::new(0, 0, &tmp_sub_pattern, max_node_id, 0);
    
        let b = vec![(2, 19), (7, 20), (11, 9)];
    
        let a = vec![(0, 17), (25, 27)];
        let ans = vec![(0, 17), (2, 19), (7, 20), (11, 9), (25, 27)];
    
        let merged = sub_pattern_buffer.try_merge_entities(&a, &b);
        assert_ne!(merged, None);
        assert_eq!(merged.unwrap(), ans);
    }
    
    #[test]
    /// pattern edge not shared: Fail
    fn test_try_merge_edges1() {
        // num_edges1 = 2;
        // num_edges2 = 3;
    
        let num_edges = 20;
        let tmp_sub_pattern = SubPattern {
            id: 0,
            events: vec![],
        };
        let sub_pattern_buffer = SubPatternBuffer::new(0, 0, &tmp_sub_pattern, 0, num_edges);
    
        let pattern_edge_ids1 = vec![1, 3];
        let pattern_edge_ids2 = vec![2, 4, 5];
    
        let input_edge_data1 = vec![(2, 0), (5, 0)];
    
        let input_edge_data2 = vec![(2, 0), (10, 0), (12, 0)];
    
        let mut pattern_edges1 = vec![];
        for id in pattern_edge_ids1 {
            pattern_edges1.push(gen_edge(id));
        }
        let mut pattern_edges2 = vec![];
        for id in pattern_edge_ids2 {
            pattern_edges2.push(gen_edge(id));
        }
    
        let match_edge1 = gen_match_edges(&pattern_edges1, &input_edge_data1);
        let match_edge2 = gen_match_edges(&pattern_edges2, &input_edge_data2);
        assert!(sub_pattern_buffer
            .try_merge_match_events(&match_edge1, &match_edge2)
            .is_none());
    }
    
    #[test]
    /// Pass
    fn test_try_merge_edges2() {
        // num_edges1 = 2;
        // num_edges2 = 3;
    
        let num_edges = 20;
        let tmp_sub_pattern = SubPattern {
            id: 0,
            events: vec![],
        };
        let sub_pattern_buffer = SubPatternBuffer::new(0, 0, &tmp_sub_pattern, 0, num_edges);
    
        let pattern_edge_ids1 = vec![1, 3];
        let pattern_edge_ids2 = vec![1, 4, 5];
    
        let input_edge_data1 = vec![(2, 0), (5, 0)];
    
        let input_edge_data2 = vec![(2, 0), (10, 0), (12, 0)];
    
        let mut pattern_edges1 = vec![];
        for id in pattern_edge_ids1 {
            pattern_edges1.push(gen_edge(id));
        }
        let mut pattern_edges2 = vec![];
        for id in pattern_edge_ids2 {
            pattern_edges2.push(gen_edge(id));
        }
    
        let match_edge1 = gen_match_edges(&pattern_edges1, &input_edge_data1);
        let match_edge2 = gen_match_edges(&pattern_edges2, &input_edge_data2);
    
        let res = sub_pattern_buffer.try_merge_match_events(&match_edge1, &match_edge2);
        assert!(!res.is_none());
    
        let ans_pattern_edge_ids = vec![1, 3, 4, 5];
        let mut ans_pattern_edges = vec![];
        for id in ans_pattern_edge_ids {
            ans_pattern_edges.push(gen_edge(id));
        }
        let ans_input_edge_data = vec![(2, 0), (5, 0), (10, 0), (12, 0)];
        let ans_match_edge = gen_match_edges(&ans_pattern_edges, &ans_input_edge_data);
        let match_edge = res.unwrap().0;
        for i in 0..match_edge.len() {
            if !cmp_match_edge(&match_edge[i], &ans_match_edge[i]) {
                assert!(false);
            }
        }
    }
    
    fn gen_match_edges<'p>(
        pattern_edges: &'p Vec<Event>,
        input_edge_data: &Vec<(u64, u64)>,
    ) -> Vec<MatchEvent<'p>> {
        let num_edges = input_edge_data.len();
    
        let mut input_edges = vec![];
        for (id, timestamp) in input_edge_data {
            input_edges.push(gen_input_edge(*id, *timestamp));
        }
    
        let mut match_edges = vec![];
        for i in 0..num_edges {
            match_edges.push(MatchEvent {
                input_event: Rc::new(input_edges[i].clone()),
                matched: &pattern_edges[i],
            });
        }
    
        match_edges
    }
    
    fn gen_edge(id: usize) -> Event {
        Event {
            id,
            signature: "".to_string(),
            subject: 0,
            object: 0,
        }
    }
    
    fn gen_input_edge(id: u64, timestamp: u64) -> InputEvent {
        InputEvent {
            timestamp,
            signature: "".to_string(),
            id,
            subject: 0,
            object: 0,
        }
    }
    
    /// only check ids
    fn cmp_match_edge(edge1: &MatchEvent, edge2: &MatchEvent) -> bool {
        if edge1.matched.id != edge2.matched.id {
            return false;
        } else if edge1.input_event.id != edge2.input_event.id {
            return false;
        }
        true
    }
}