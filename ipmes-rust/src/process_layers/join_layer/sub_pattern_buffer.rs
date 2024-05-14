use super::sub_pattern_match::EarliestFirst;
use crate::match_event::MatchEvent;
use crate::pattern::Pattern;
use crate::process_layers::join_layer::sub_pattern_buffer::TimeOrder::{
    FirstToSecond, SecondToFirst,
};
use crate::sub_pattern::SubPattern;
use crate::universal_match_event::UniversalMatchEvent;
use log::debug;
use std::collections::{BinaryHeap, HashSet};

use super::{get_parent_id, get_sibling_id};

#[derive(Clone, Debug, PartialEq)]
enum TimeOrder {
    FirstToSecond,
    SecondToFirst,
}

/// A structure that holds order relations between sibling buffers.
#[derive(Clone, Debug)]
pub struct Relation {
    /// `shared_entities.len() == num_node`
    ///
    /// If node `i` is shared, `shared_entities[i] == true`.
    ///
    /// `i`: pattern node id
    ///
    /// (The "overall structure" has guaranteed nodes be shared properly, when performing "SubPatternMatch::try_merge_nodes()".)
    shared_entities: Vec<bool>,

    /// `event_orders: (pattern_id1, pattern_id2, TimeOrder)`
    ///
    /// `pattern_id1` (`pattern_id2`, respectively) is the id of some left  (right, respectively) buffer on `JoinLayer::sub_pattern_buffers`, where the two buffers are siblings.
    event_orders: Vec<(usize, usize, TimeOrder)>,
}

impl Relation {
    pub fn new() -> Self {
        Self {
            shared_entities: Vec::new(),
            event_orders: Vec::new(),
        }
    }

    /// Check whether order relations are violated between two pattern matches.
    pub fn check_order_relation(&self, match_event_map: &[Option<UniversalMatchEvent>]) -> bool {
        for (idx1, idx2, time_order) in &self.event_orders {
            if let (Some(event1), Some(event2)) = (&match_event_map[*idx1], &match_event_map[*idx2])
            {
                if !Self::satisfy_order(event1, event2, time_order) {
                    return false;
                }
            } else {
                debug!("failed to get events specified in the order relation");
                return false;
            }
        }

        true
    }

    fn satisfy_order(
        event1: &UniversalMatchEvent,
        event2: &UniversalMatchEvent,
        time_order: &TimeOrder,
    ) -> bool {
        match time_order {
            FirstToSecond => event1.end_time <= event2.start_time,
            SecondToFirst => event2.end_time <= event1.start_time,
        }
    }

    pub fn is_entity_shared(&self, id: usize) -> bool {
        self.shared_entities[id]
    }
}

impl Default for Relation {
    fn default() -> Self {
        Self::new()
    }
}

/// A Buffer that holds sub-pattern matches that correspond to a certain sub-pattern.
#[derive(Clone, Debug)]
pub struct SubPatternBuffer<'p> {
    /// Buffer id.
    pub id: usize,
    /// Mainly for debugging.
    sibling_id: usize,
    /// Ids of pattern entities (nodes) contained in this sub-pattern.
    node_id_list: HashSet<usize>,
    /// Ids of pattern events (edges) contained in this sub-pattern.
    edge_id_list: HashSet<usize>,
    /// A buffer that holds sub-pattern matches.
    pub(crate) buffer: BinaryHeap<EarliestFirst<'p>>,
    /// A buffer that holds newly came sub-pattern matches.
    pub(crate) new_match_buffer: BinaryHeap<EarliestFirst<'p>>,
    /// The order relations between the sub-pattern this buffer corresponds to and the one its sibling buffer corresponds to.
    pub relation: Relation,
    /// Number of entities in the overall pattern.
    pub max_num_entities: usize,
    /// Number of events in the overall pattern.
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
            node_id_list.insert(edge.subject.id);
            node_id_list.insert(edge.object.id);
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

    /// Precalculate order relations between sibling buffers.
    pub fn generate_relations(
        pattern: &Pattern,
        sub_pattern_buffer1: &SubPatternBuffer,
        sub_pattern_buffer2: &SubPatternBuffer,
        // distances_table: &HashMap<(NodeIndex, NodeIndex), i32>,
    ) -> Relation {
        let mut shared_entities = vec![false; pattern.entities.len()];
        let mut event_orders = Vec::new();

        // identify shared nodes
        for i in 0..pattern.entities.len() {
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

    /// Merge two sub-pattern buffers.
    pub fn merge_buffers(
        sub_pattern_buffer1: &SubPatternBuffer,
        sub_pattern_buffer2: &SubPatternBuffer,
    ) -> Self {
        let mut node_id_list = sub_pattern_buffer1.node_id_list.clone();
        let mut edge_id_list = sub_pattern_buffer1.edge_id_list.clone();
        node_id_list.extend(&sub_pattern_buffer2.node_id_list);
        edge_id_list.extend(&sub_pattern_buffer2.edge_id_list);

        let id = get_parent_id(sub_pattern_buffer1.id);

        Self {
            id,
            sibling_id: get_sibling_id(id),
            node_id_list,
            edge_id_list,
            buffer: BinaryHeap::new(),
            new_match_buffer: BinaryHeap::new(),
            relation: Relation::new(),
            max_num_entities: sub_pattern_buffer1.max_num_entities,
            max_num_events: sub_pattern_buffer1.max_num_events,
        }
    }

    /// Try to merge two match events and check event uniqueness.
    pub fn try_merge_match_events(
        &self,
        a: &[MatchEvent<'p>],
        b: &[MatchEvent<'p>],
    ) -> Option<(Vec<MatchEvent<'p>>, Vec<u64>)> {
        // `timestamps[pattern_event_id] = the timestamp of the corresponding input event.`
        let mut timestamps = vec![0u64; self.max_num_events];
        let mut merged = Vec::with_capacity(a.len() + b.len());

        let mut p1 = a.iter();
        let mut p2 = b.iter();
        let mut next1 = p1.next();
        let mut next2 = p2.next();

        while let (Some(edge1), Some(edge2)) = (next1, next2) {
            if edge1.input_event.event_id < edge2.input_event.event_id {
                merged.push(edge1.clone());
                timestamps[edge1.matched.id] = edge1.input_event.timestamp;
                next1 = p1.next();
            } else if edge1.input_event.event_id > edge2.input_event.event_id {
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

    /// Try to merge match entities, and handle "shared entities" and "entity uniqueness".
    ///
    /// (`a` and `b` are slices over `(input node id, pattern node id)`.)
    pub fn try_merge_entities(
        &self,
        a: &[(u64, u64)],
        b: &[(u64, u64)],
    ) -> Option<Box<[(u64, u64)]>> {
        let mut used_nodes = vec![false; self.max_num_entities];
        let mut merged = Vec::with_capacity(a.len() + b.len());

        let mut p1 = a.iter();
        let mut p2 = b.iter();

        let mut next1 = p1.next();
        let mut next2 = p2.next();

        while let (Some(node1), Some(node2)) = (next1, next2) {
            if used_nodes[node1.1 as usize] || used_nodes[node2.1 as usize] {
                debug!("different inputs match the same pattern");
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
                    debug!("an input match to different pattern");
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

        Some(merged.into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_event::InputEvent;
    use crate::pattern::{PatternEntity, PatternEvent, PatternEventType};
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
        let ans = [(0, 17), (2, 19), (7, 20), (11, 9), (25, 27)];

        let merged = sub_pattern_buffer.try_merge_entities(&a, &b);
        assert_ne!(merged, None);
        assert!(merged.unwrap().iter().eq(&ans));
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
        let ans = [(0, 17), (2, 19), (7, 20), (11, 9), (25, 27)];

        let merged = sub_pattern_buffer.try_merge_entities(&a, &b);
        assert_ne!(merged, None);
        assert!(merged.unwrap().iter().eq(&ans));
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
        assert!(res.is_some());

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
                unreachable!()
            }
        }
    }

    fn gen_match_edges<'p>(
        pattern_edges: &'p [PatternEvent],
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

    fn gen_edge(id: usize) -> PatternEvent {
        PatternEvent {
            id,
            event_type: PatternEventType::Default,
            signature: "".to_string(),
            subject: PatternEntity {
                id: 0,
                signature: String::new(),
            },
            object: PatternEntity {
                id: 0,
                signature: String::new(),
            },
        }
    }

    fn gen_input_edge(id: u64, timestamp: u64) -> InputEvent {
        InputEvent::new(timestamp, id, "", 0, "", 0, "")
    }

    /// only check ids
    fn cmp_match_edge(edge1: &MatchEvent, edge2: &MatchEvent) -> bool {
        edge1.matched.id == edge2.matched.id
            && edge1.input_event.event_id == edge2.input_event.event_id
    }
}
