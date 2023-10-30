use crate::pattern_match::PatternMatch;
use crate::sub_pattern::SubPattern;
use crate::sub_pattern_match::{EarliestFirst, SubPatternMatch};
use std::cmp::{max, min};

use crate::input_edge::InputEdge;
use crate::match_edge::MatchEdge;
use crate::pattern::Pattern;
use crate::process_layers::join_layer::TimeOrder::{FirstToSecond, SecondToFirst};
use crate::process_layers::ord_match_layer::PartialMatch;
use crate::sub_pattern_match;
use itertools::{assert_equal, Itertools};
use petgraph::adj::DefaultIx;
use petgraph::data::Element::Node;
use petgraph::graph::NodeIndex;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;
use std::path::Component::ParentDir;
use std::rc::Rc;
use serde_json::to_string;

/*
   It seems that "Huffman encoding" is not feasible: The sizes of Relation are the weights, but we need
   to know "who is my sibling" when generating relations.

   Really? Reflect on this more.
*/

enum TimeOrder {
    FirstToSecond,
    SecondToFirst,
}

#[derive(Clone)]
pub struct Relation {
    /// shared_nodes.len() == num_node
    /// If node 'i' is shared, shared_nodes[i] = true.
    /// 'i': pattern node id
    shared_nodes: Vec<bool>,
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
        myself: &SubPatternMatch,
        sibling: &SubPatternMatch,
    ) -> bool {
        self.edge_orders.iter().all(|(id1, id2, time_order)| {
            if let (Some(e1), Some(e2)) = (myself.edge_id_map[id1], sibling.edge_id_map[id2]) {
                match time_order {
                    FirstToSecond => e1.timestamp <= e2.timestamp,
                    SecondToFirst => e1.timestamp >= e2.timestamp,
                }
            } else {
                false
            }
        })
    }

    pub fn is_node_shared(&self, id: &u64) -> bool {
        self.shared_nodes[id]
    }
}

pub struct SubPatternBuffer<'p> {
    id: usize,
    sibling_id: usize,
    /// List of ids of pattern nodes (edges) contained in this sub-pattern.
    node_id_list: HashSet<usize>,
    edge_id_list: HashSet<usize>,
    buffer: BinaryHeap<EarliestFirst<'p>>,
    new_match_buffer: BinaryHeap<EarliestFirst<'p>>,
    pub relation: Relation,
}

impl SubPatternBuffer {
    pub fn new(id: usize, sibling_id: usize, sub_pattern: &SubPattern) -> Self {
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
        }
    }

    pub fn generate_relations(
        // &mut self,
        pattern: &Pattern,
        sub_pattern_buffer1: &SubPatternBuffer,
        sub_pattern_buffer2: &SubPatternBuffer,
        distances_table: &HashMap<(NodeIndex, NodeIndex), i32>,
    ) -> Relation {
        let mut shared_nodes = vec![false, pattern.num_nodes];
        let mut edge_orders = Vec::new();

        for i in 0..pattern.num_nodes {
            if sub_pattern_buffer1.node_id_list.contains(&i)
                && sub_pattern_buffer2.node_id_list.contains(&i)
            {
                shared_nodes[i] = true;
            }
        }

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
        &sub_pattern_buffer1: &SubPatternBuffer,
        &sub_pattern_buffer2: &SubPatternBuffer,
    ) -> Self {
        let mut node_id_list = sub_pattern_buffer1.node_id_list.clone();
        let mut edge_id_list = sub_pattern_buffer1.edge_id_list.clone();
        node_id_list.extend(sub_pattern_buffer2.node_id_list);
        edge_id_list.extend(sub_pattern_buffer2.edge_id_list);

        let id = max(sub_pattern_buffer1.id, sub_pattern_buffer2.id) + 1;

        Self {
            id,
            sibling_id: id + 1,
            node_id_list,
            edge_id_list,
            buffer: BinaryHeap::new(),
            new_match_buffer: BinaryHeap::new(),
            relation: Relation::new(),
        }
    }
}

/// todo: check "match uniqueness" (matches in buffers); To be handled by "hashing".
pub struct JoinLayer<'p, P> {
    prev_layer: P,
    pattern: &'p Pattern,
    num_sub_patterns: usize,
    sub_pattern_buffers: Vec<SubPatternBuffer<'p>>,

    /// The uniqueness of answers should be handled. (All buffers should do.)
    patterns_matched: Vec<PatternMatch>,
    window_size: u64,
}

impl<'p, P> JoinLayer<'p, P> {
    fn create_buffer_pair(
        id: usize,
        pattern: &'p Pattern,
        sub_patterns: &'p Vec<SubPattern>,
        sub_pattern_buffers: &mut Vec<SubPatternBuffer>,
        distances_table: &HashMap<(NodeIndex, NodeIndex), i32>,
    ) {
        let mut sub_pattern_buffer1 = sub_pattern_buffers.pop().unwrap();
        let mut sub_pattern_buffer2 = SubPatternBuffer::new(
            sub_pattern_buffer1.id + 1,
            sub_pattern_buffer1.id,
            &sub_patterns[id],
        );
        let relations = SubPatternBuffer::generate_relations(
            &pattern,
            &sub_pattern_buffer1,
            &sub_pattern_buffer2,
            &distances_table,
        );
        sub_pattern_buffer1.relation = relations.clone();
        sub_pattern_buffer2.relation = relations;

        sub_pattern_buffers.push(sub_pattern_buffer1);
        sub_pattern_buffers.push(sub_pattern_buffer2);
        sub_pattern_buffers.push(SubPatternBuffer::merge_buffers(
            &sub_pattern_buffer1,
            &sub_pattern_buffer2,
        ));
    }
    pub fn new(
        prev_layer: P,
        pattern: &'p Pattern,
        sub_patterns: &'p Vec<SubPattern>,
        window_size: u64,
    ) -> Self {
        let distances_table = pattern.order.calculate_distances().unwrap();
        let mut sub_pattern_buffers = Vec::with_capacity(2 * sub_patterns.len() - 1);

        sub_pattern_buffers.push(SubPatternBuffer::new(0, 1, &sub_patterns[0]));

        for i in 1..sub_patterns.len() {
            Self::create_buffer_pair(
                i,
                pattern,
                sub_patterns,
                &mut sub_pattern_buffers,
                &distances_table,
            );
        }

        Self {
            prev_layer,
            pattern,
            num_sub_patterns: sub_patterns.len(),
            sub_pattern_buffers,
            patterns_matched: Vec::new(),
            window_size,
        }
    }

    fn to_pattern_match(&self, buffer_id: usize) -> Vec<PatternMatch> {
        let empty_input_edge = Rc::new(InputEdge{
            timestamp: 0,
            signature: "".to_string(),
            id: 0,
            start: 0,
            end: 0,
        });
        let mut pattern_matches = Vec::new();

        for sub_pattern_match in &self.sub_pattern_buffers[buffer_id].buffer {
            let mut matched_edges = vec![empty_input_edge, self.pattern.edges.len()];
            for match_edge in &sub_pattern_match.0.match_edges {
                matched_edges[match_edge.matched.id] = Rc::clone(&match_edge.input_edge);
            }

            /// for testing
            /// no two pattern edges should match to the same input edge
            assert!(matched_edges.iter().all_unique());
            /// every pattern edge should be matched
            assert!(!matched_edges.iter().any(|&x| x.eq(&empty_input_edge)));

            pattern_matches.push(PatternMatch{matched_edges});
        }
        pattern_matches
    }

    /// The uniqueness of matches should be handled (Just like any other buffer.).
    fn add_to_answer(&mut self) {
        let num_buffers = self.sub_pattern_buffers.len();
        self.patterns_matched.extend(self.to_pattern_match(num_buffers - 1));

        /// Clear used matches.
        self.sub_pattern_buffers[num_buffers - 1].buffer.clear();
    }

    fn get_left_buffer_id(buffer_id: usize) -> usize {
        buffer_id - buffer_id % 2
    }

    /// Siblings' buffer ids only differ by their LSB.
    fn get_sibling_id(buffer_id: usize) -> usize {
        buffer_id ^ 1
    }

    fn get_parent_id(buffer_id: usize) -> usize {
        Self::get_left_buffer_id(buffer_id) + 2
    }

    fn clear_expired(&self, latest_time: u64, buffer_id: usize) {
        while let Some(sub_pattern_match) = self.sub_pattern_buffers[buffer_id].buffer.peek() {
            if latest_time.saturating_sub(self.window_size) > sub_pattern_match.0.earliest_time {
                self.sub_pattern_buffers[buffer_id].buffer.pop();
            }
        }
    }

    /// My new_match_buffer, joined with sibling's buffer.
    fn join_with_sibling(&self, my_id: usize, sibling_id: usize) -> BinaryHeap<EarliestFirst<'p>> {
        let mut matches_to_parent = BinaryHeap::new();
        for sub_pattern_match1 in &self.sub_pattern_buffers[my_id].new_match_buffer {
            for sub_pattern_match2 in &self.sub_pattern_buffers[sibling_id].buffer {
                if self.sub_pattern_buffers[my_id]
                    .relation
                    .check(&sub_pattern_match1.0, &sub_pattern_match2.0)
                {
                    if let Some(merged) = SubPatternMatch::merge_matches(
                        &self.sub_pattern_buffers[my_id],
                        &sub_pattern_match1.0,
                        &sub_pattern_match2.0,
                    ) {
                        matches_to_parent.push(EarliestFirst(merged));
                    }
                }
            }
        }
        matches_to_parent
    }

    /// Join new-matches with matches in its sibling buffer, in a button-up fashion.
    fn join(&mut self, current_time: u64, mut buffer_id: usize) {
        loop {
            let new_matches = &self.sub_pattern_buffers[buffer_id].new_match_buffer;

            let parent_id = Self::get_parent_id(buffer_id);
            self.sub_pattern_buffers[buffer_id]
                .buffer
                .extend(&new_matches);

            /// Clear only sibling buffer, since we can clear current buffer when needed (deferred).
            self.clear_expired(current_time, Self::get_sibling_id(buffer_id));

            self.sub_pattern_buffers[parent_id]
                .new_match_buffer
                .extend(&self.join_with_sibling(buffer_id, Self::get_sibling_id(buffer_id)));

            /// root reached
            if parent_id == self.sub_pattern_buffers.len() - 1 {
                self.add_to_answer();
                break;
            }

            buffer_id = parent_id;
        }
    }
}

fn convert_node_id_map(node_id_map: &mut Vec<(u64, u64)>, node_ids: &Vec<u64>) {
    for (i, node_id) in node_ids.iter().enumerate() {
        /// "node_id == 0": i-th node is not matched
        if node_id == 0 {
            continue;
        }
        node_id_map.push((node_id.clone(), i as u64));
    }

    node_id_map.sort();
}

/// Return the "earliest time" of all edges' timestamps.
fn create_edge_id_map(edge_id_map: &mut Vec<Option<u64>>, edges: &Vec<MatchEdge>) -> u64 {
    let mut earliest_time = u64::MAX;
    for edge in edges {
        earliest_time = min(earliest_time, edge.input_edge.timestamp);
        edge_id_map[edge.matched.id] = Some(edge.input_edge.id);
    }
    earliest_time
}

impl<'p, P> Iterator for JoinLayer<'p, P>
where
    P: Iterator<Item = Vec<PartialMatch<'p>>>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Vec<Self::Item>> {
        /// todo: return the "fully matched results"
        while self.patterns_matched.is_empty() {
            let partial_matches = self.prev_layer.next()?;
            let mut sub_pattern_matches = Vec::new();

            /// Convert PartialMatch to SubPatternMatch
            for partial_match in partial_matches {
                let mut node_id_map = vec![(0, 0), self.pattern.num_nodes];
                let mut edge_id_map = vec![None, self.pattern.edges.len()];
                convert_node_id_map(&mut node_id_map, &partial_match.node_id_map);
                let earliest_time = create_edge_id_map(&mut edge_id_map, &partial_match.edges);

                let mut match_edges = partial_match.edges;
                match_edges.sort_by(|x, y| x.input_edge.id.cmp(&y.input_edge.id));

                let sub_pattern_match = SubPatternMatch {
                    id: partial_match.id,
                    latest_time: partial_match.timestamp,
                    earliest_time,
                    node_id_map,
                    edge_id_map,
                    match_edges,
                };

                /// put the sub-pattern match to its corresponding buffer
                self.sub_pattern_buffers[sub_pattern_match.id]
                    .new_match_buffer
                    .push(EarliestFirst(sub_pattern_match));
            }

            for buffer_id in 0..self.sub_pattern_buffers.len() {
                let new_match_buffer = &self.sub_pattern_buffers[buffer_id].new_match_buffer;
                if !(new_match_buffer.is_empty()) {
                    let current_time = new_match_buffer[0].timestamp;
                    self.join(current_time, buffer_id);
                }
            }
        }

        let patterns_matched = self.patterns_matched.iter().clone();
        self.patterns_matched.clear();
        Some(patterns_matched.collect())
    }
}