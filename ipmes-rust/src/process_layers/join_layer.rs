mod sub_pattern_buffer;

pub use sub_pattern_buffer::SubPatternBuffer;
use crate::pattern_match::PatternMatch;
use crate::sub_pattern::SubPattern;
use crate::sub_pattern_match::{EarliestFirst, SubPatternMatch};
use std::cmp::min;

use crate::input_edge::InputEdge;
use crate::match_edge::MatchEdge;
use crate::pattern::Pattern;
use crate::process_layers::ord_match_layer::PartialMatch;
use itertools::Itertools;
use petgraph::graph::NodeIndex;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;
use std::rc::Rc;


// todo: check "answer uniqueness"
pub struct JoinLayer<'p, P> {
    prev_layer: P,
    pattern: &'p Pattern,
    sub_pattern_buffers: Vec<SubPatternBuffer<'p>>,
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
        }
    }

    fn to_pattern_match(&self, buffer_id: usize) -> Vec<PatternMatch> {
        let empty_input_edge = Rc::new(InputEdge {
            timestamp: 0,
            signature: "".to_string(),
            id: 0,
            start: 0,
            end: 0,
        });
        let empty_matches = vec![empty_input_edge.clone(); self.pattern.edges.len()];
        let mut pattern_matches = Vec::new();

        for sub_pattern_match in &self.sub_pattern_buffers[buffer_id].buffer {
            // let mut matched_edges = vec![empty_input_edge; self.pattern.edges.len()];
            let mut matched_edges = empty_matches.clone();
            for match_edge in &sub_pattern_match.0.match_edges {
                matched_edges[match_edge.matched.id] = Rc::clone(&match_edge.input_edge);
            }

            // for testing
            // no two pattern edges should match to the same input edge
            // assert!(matched_edges.iter().all_unique());
            // every pattern edge should be matched
            assert!(!matched_edges.iter().any(|x| x.eq(&empty_input_edge)));

            pattern_matches.push(PatternMatch { matched_edges });
        }
        pattern_matches
    }

    /// The uniqueness of matches should be handled.
    fn add_to_answer(&mut self, results: &mut Vec<PatternMatch>) {
        let root_id = self.sub_pattern_buffers.len() - 1;
        results.extend(self.to_pattern_match(root_id));

        /// Clear used matches.
        self.sub_pattern_buffers[root_id].buffer.clear();
    }

    fn get_left_buffer_id(buffer_id: usize) -> usize {
        buffer_id - buffer_id % 2
    }

    // Siblings' buffer ids only differ by their LSB.
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

    fn get_root_buffer_id(&self) -> usize { self.sub_pattern_buffers.len() - 1 }

    fn clear_expired(&mut self, latest_time: u64, buffer_id: usize) {
        while let Some(sub_pattern_match) = self.sub_pattern_buffers[buffer_id].buffer.peek() {
            if latest_time.saturating_sub(self.window_size) > sub_pattern_match.0.earliest_time {
                self.sub_pattern_buffers[buffer_id].buffer.pop();
            }
        }
    }

    /// My new_match_buffer, joined with sibling's buffer.
    fn join_with_sibling(&mut self, my_id: usize, sibling_id: usize) -> BinaryHeap<EarliestFirst<'p>> {
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
    fn join(&mut self, current_time: u64, mut buffer_id: usize, results: &mut Vec<PatternMatch>) {
        // root is the only buffer
        if buffer_id == self.get_root_buffer_id() {
            let new_matches = self.sub_pattern_buffers[buffer_id].new_match_buffer.clone();
            self.sub_pattern_buffers[buffer_id]
                .buffer
                .extend(new_matches);
            return;
        }

        loop {
            let new_matches = self.sub_pattern_buffers[buffer_id].new_match_buffer.clone();
            self.sub_pattern_buffers[buffer_id]
                .buffer
                .extend(new_matches);


            /// Clear only sibling buffer, since we can clear current buffer when needed (deferred).
            self.clear_expired(current_time, self.get_sibling_id(buffer_id));

            let parent_id = self.get_parent_id(buffer_id);
            let joined = self.join_with_sibling(buffer_id, self.get_sibling_id(buffer_id));
            self.sub_pattern_buffers[parent_id]
                .new_match_buffer
                .extend(joined);
            /// Clear used matches.
            self.sub_pattern_buffers[buffer_id].new_match_buffer.clear();


            /// root reached
            if parent_id == self.get_root_buffer_id() {
                self.add_to_answer(results);
                break;
            }

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

// Return the "earliest time" of all edges' timestamps.
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
    type Item = Vec<PatternMatch>;

    fn next(&mut self) -> Option<Self::Item> {
        // todo: return the "fully matched results"
        let mut results = Vec::new();
        while results.is_empty() {
            let partial_matches = self.prev_layer.next()?;

            // Convert PartialMatch to SubPatternMatch
            for partial_match in partial_matches {
                let mut node_id_map = vec![(0, 0); self.pattern.num_nodes];
                let mut edge_id_map = vec![None; self.pattern.edges.len()];
                convert_node_id_map(&mut node_id_map, &partial_match.node_id_map);
                let earliest_time = create_edge_id_map(&mut edge_id_map, &partial_match.edges);

                let mut match_edges = partial_match.edges;
                match_edges.sort_by(|x, y| x.input_edge.id.cmp(&y.input_edge.id));

                let sub_pattern_match = SubPatternMatch {
                    id: partial_match.id,
                    latest_time: partial_match.timestamp,
                    earliest_time,
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
                    self.join(current_time, buffer_id, &mut results);
                }
            }
        }

        Some(results)
    }
}
