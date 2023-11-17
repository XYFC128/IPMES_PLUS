mod sub_pattern_buffer;

use crate::pattern_match::PatternMatch;
use crate::sub_pattern::SubPattern;
use crate::sub_pattern_match::{EarliestFirst, SubPatternMatch};
use std::cmp::{max, min};
pub use sub_pattern_buffer::SubPatternBuffer;

use crate::input_edge::InputEdge;
use crate::match_edge::MatchEdge;
use crate::pattern::Pattern;
use crate::process_layers::ord_match_layer::PartialMatch;
use itertools::Itertools;
use petgraph::graph::NodeIndex;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;
use std::mem;
use std::rc::Rc;
use log::warn;

pub struct JoinLayer<'p, P> {
    prev_layer: P,
    pattern: &'p Pattern,
    sub_pattern_buffers: Vec<SubPatternBuffer<'p>>,
    window_size: u64,
    full_match: HashSet<PatternMatch>,
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
            sub_pattern_buffer1.max_num_nodes,
            pattern.edges.len(),
        );
        let relations = SubPatternBuffer::generate_relations(
            &pattern,
            &sub_pattern_buffer1,
            &sub_pattern_buffer2,
            &distances_table,
        );
        sub_pattern_buffer1.relation = relations.clone();
        sub_pattern_buffer2.relation = relations;

        sub_pattern_buffers.push(sub_pattern_buffer1.clone());
        sub_pattern_buffers.push(sub_pattern_buffer2.clone());
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

        sub_pattern_buffers.push(SubPatternBuffer::new(
            0,
            1,
            &sub_patterns[0],
            pattern.num_nodes,
            pattern.edges.len(),
        ));

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
            sub_pattern_buffers,
            window_size,
            full_match: HashSet::new(),
        }
    }

    /// change the name of the function
    /// "match_edges" is sorted by its match edge ids, and thus "matched_edges" is in good order.
    fn convert_pattern_match(buffer: &mut BinaryHeap<EarliestFirst<'p>>) -> HashSet<PatternMatch> {
        let mut pattern_matches = HashSet::new();

        for mut sub_pattern_match in buffer.drain() {
            let mut matched_edges = Vec::new();
            sub_pattern_match
                .0
                .match_edges
                .sort_by(|a, b| a.matched.id.cmp(&b.matched.id));

            for match_edge in &sub_pattern_match.0.match_edges {
                matched_edges.push(Rc::clone(&match_edge.input_edge));
            }

            pattern_matches.insert(PatternMatch { matched_edges });
        }
        pattern_matches
    }

    /// The uniqueness of matches should be handled.
    fn add_to_answer(&mut self) {
        let root_id = self.sub_pattern_buffers.len() - 1;
        self.full_match.extend(Self::convert_pattern_match(
            &mut self.sub_pattern_buffers[root_id].buffer,
        ));
        self.full_match.extend(Self::convert_pattern_match(
            &mut self.sub_pattern_buffers[root_id].new_match_buffer,
        ));

        /// for testing
        assert!(self.sub_pattern_buffers[root_id].buffer.is_empty());
        assert!(self.sub_pattern_buffers[root_id]
            .new_match_buffer
            .is_empty());
    }

    fn get_left_buffer_id(buffer_id: usize) -> usize {
        buffer_id - buffer_id % 2
    }

    /// Siblings' buffer ids only differ by their LSB.
    fn get_sibling_id(&self, buffer_id: usize) -> usize {
        // root has no sibling
        if buffer_id == self.get_root_buffer_id() {
            return buffer_id;
        }
        buffer_id ^ 1
    }

    fn get_parent_id(&self, buffer_id: usize) -> usize {
        // root has no parent
        if buffer_id == self.get_root_buffer_id() {
            return buffer_id;
        }
        Self::get_left_buffer_id(buffer_id) + 2
    }

    fn get_root_buffer_id(&self) -> usize {
        self.sub_pattern_buffers.len() - 1
    }

    fn clear_expired(&mut self, latest_time: u64, buffer_id: usize) {
        while let Some(sub_pattern_match) = self.sub_pattern_buffers[buffer_id].buffer.peek() {
            if latest_time.saturating_sub(self.window_size) > sub_pattern_match.0.earliest_time {
                self.sub_pattern_buffers[buffer_id].buffer.pop();
            } else {
                break;
            }
        }
    }

    /// My new_match_buffer, joined with sibling's buffer.
    fn join_with_sibling(
        &mut self,
        my_id: usize,
        sibling_id: usize,
    ) -> BinaryHeap<EarliestFirst<'p>> {
        let mut matches_to_parent = BinaryHeap::new();
        let buffer1 = self.sub_pattern_buffers[my_id].new_match_buffer.clone();
        let buffer2 = self.sub_pattern_buffers[sibling_id].buffer.clone();

        for sub_pattern_match1 in &buffer1 {
            for sub_pattern_match2 in &buffer2 {
                if let Some(merged) = SubPatternMatch::merge_matches(
                    &mut self.sub_pattern_buffers[my_id],
                    &sub_pattern_match1.0,
                    &sub_pattern_match2.0,
                ) {
                    matches_to_parent.push(EarliestFirst(merged));
                }
            }
        }

        matches_to_parent
    }

    /// Join new-matches with matches in its sibling buffer, in a button-up fashion.
    fn join(&mut self, current_time: u64, mut buffer_id: usize) {
        loop {
            /// root reached
            if buffer_id == self.get_root_buffer_id() {
                self.add_to_answer();
                break;
            }

            /// Clear only sibling buffer, since we can clear current buffer when needed (deferred).
            self.clear_expired(current_time, self.get_sibling_id(buffer_id));

            let joined = self.join_with_sibling(buffer_id, self.get_sibling_id(buffer_id));
            let parent_id = self.get_parent_id(buffer_id);
            self.sub_pattern_buffers[parent_id]
                .new_match_buffer
                .extend(joined);


            /// move new matches to buffer
            let new_matches = mem::replace(
                &mut self.sub_pattern_buffers[buffer_id].new_match_buffer,
                BinaryHeap::new(),
            );
            self.sub_pattern_buffers[buffer_id]
                .buffer
                .extend(new_matches);


            buffer_id = parent_id;
        }
    }
}

fn convert_node_id_map(node_id_map: &mut Vec<(u64, u64)>, node_ids: &Vec<u64>) {
    for (i, node_id) in node_ids.iter().enumerate() {
        // "node_id == 0": i-th node is not matched
        if node_id == &0u64 {
            continue;
        }
        node_id_map.push((node_id.clone(), i as u64));
    }

    node_id_map.sort();
}

/// Convert vector of edges to vector map of edges
fn create_edge_id_map(edge_id_map: &mut Vec<Option<u64>>, edges: &Vec<MatchEdge>) {
    for edge in edges {
        edge_id_map[edge.matched.id] = Some(edge.input_edge.id);
    }
}

impl<'p, P> Iterator for JoinLayer<'p, P>
where
    P: Iterator<Item = Vec<PartialMatch<'p>>>,
{
    type Item = HashSet<PatternMatch>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.full_match.is_empty() {
            let partial_matches = self.prev_layer.next()?;

            /// Convert PartialMatch to SubPatternMatch
            /// Maybe isolate it to be a function?
            for partial_match in partial_matches {
                let latest_time = if let Some(edge) = partial_match.edges.last() {
                    edge.input_edge.timestamp
                } else {
                    warn!("Got an empty PartialMatch");
                    continue;
                };

                let mut node_id_map = vec![(0, 0); self.pattern.num_nodes];
                let mut edge_id_map = vec![None; self.pattern.edges.len()];
                convert_node_id_map(&mut node_id_map, &partial_match.node_id_map);
                create_edge_id_map(&mut edge_id_map, &partial_match.edges);

                let mut match_edges = partial_match.edges;
                match_edges.sort_by(|x, y| x.input_edge.id.cmp(&y.input_edge.id));

                let sub_pattern_match = SubPatternMatch {
                    id: partial_match.id,
                    latest_time,
                    earliest_time: partial_match.timestamp,
                    match_nodes: node_id_map,
                    edge_id_map,
                    match_edges,
                };

                // put the sub-pattern match to its corresponding buffer
                self.sub_pattern_buffers[sub_pattern_match.id]
                    .new_match_buffer
                    .push(EarliestFirst(sub_pattern_match));
            }

            for buffer_id in 0..self.sub_pattern_buffers.len() {
                let new_match_buffer = &self.sub_pattern_buffers[buffer_id].new_match_buffer;
                if !(new_match_buffer.is_empty()) {
                    let current_time = new_match_buffer.peek().unwrap().0.latest_time;
                    self.join(current_time, buffer_id);
                }
            }
        }

        Some(mem::replace(&mut self.full_match, HashSet::new()))
    }
}
