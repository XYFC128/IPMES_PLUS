use crate::pattern_match::PatternMatch;
use crate::sub_pattern::SubPattern;
use crate::sub_pattern_match::SubPatternMatch;
use std::cmp::{min, max};

use crate::pattern::{Pattern};
use crate::process_layers::join_layer::TimeOrder::{FirstToSecond, SecondToFirst};
use petgraph::adj::DefaultIx;
use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use itertools::Itertools;
use log::SetLoggerError;

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
struct Relation {
    common_nodes: Vec<usize>,
    common_edges: Vec<usize>,
    edge_orders: Vec<(usize, usize, TimeOrder)>,
}

impl Relation {
    pub fn new() -> Self {
        Self {
            common_nodes: Vec::new(),
            common_edges: Vec::new(),
            edge_orders: Vec::new(),
        }
    }
    pub fn check(&self, myself: &SubPatternMatch, sibling: &SubPatternMatch) -> bool {
        for common_node in &self.common_nodes {
            if let (Some(n1), Some(n2)) = (
                myself.matched_nodes_table.get(&common_node),
                sibling.matched_nodes_table.get(&common_node),
            ) {
                if n1 != n2 {
                    return false;
                }
            } else {
                return false;
            }
        }

        for common_edge in &self.common_edges {
            if let (Some(e1), Some(e2)) = (
                myself.matched_edges_table.get(&common_edge),
                sibling.matched_edges_table.get(&common_edge),
            ) {
                if e1.id != e2.id {
                    return false;
                }
            } else {
                return false;
            }
        }

        self.edge_orders.iter().all(|(id1, id2, time_order)| {
            if let (Some(e1), Some(e2)) = (
                myself.matched_edges_table.get(id1),
                sibling.matched_edges_table.get(id2),
            ) {
                match time_order {
                    FirstToSecond => e1.timestamp <= e2.timestamp,
                    SecondToFirst => e1.timestamp >= e2.timestamp,
                }
            } else {
                false
            }
        })
    }
}

struct SubPatternBuffer<'p> {
    id: usize,
    sibling_id: usize,
    node_id_list: HashSet<usize>,
    edge_id_list: HashSet<usize>,
    buffer: Vec<SubPatternMatch<'p>>,
    new_match_buffer: Vec<SubPatternMatch<'p>>,
    relation: Relation,
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
            buffer: Vec::new(),
            new_match_buffer: Vec::new(),
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
        let mut common_nodes = Vec::new();
        let mut common_edges = Vec::new();
        let mut edge_orders = Vec::new();

        for i in 0..pattern.num_nodes {
            if sub_pattern_buffer1.node_id_list.contains(&i)
                && sub_pattern_buffer2.node_id_list.contains(&i)
            {
                common_nodes.push(i);
            }
        }

        for i in 0..pattern.edges.len() {
            if sub_pattern_buffer1.edge_id_list.contains(&i)
                && sub_pattern_buffer2.edge_id_list.contains(&i)
            {
                common_edges.push(i);
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
            common_nodes,
            common_edges,
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
            buffer: Vec::new(),
            new_match_buffer: Vec::new(),
            relation: Relation::new(),
        }
    }
}

pub struct JoinLayer<'p, P>
where
    P: Iterator<Item = Vec<SubPatternMatch<'p>>>,
{
    prev_layer: P,
    num_sub_patterns: usize,
    sub_pattern_buffers: Vec<SubPatternBuffer<'p>>,
    patterns_matched: Vec<PatternMatch>,
    window_size: usize
}

impl<'p, P> JoinLayer<'p, P>
where
    P: Iterator<Item = Vec<SubPatternMatch<'p>>>,
{
    fn create_buffer_pair(
        id: usize,
        pattern: &'p Pattern,
        sub_patterns: &'p Vec<SubPattern>,
        sub_pattern_buffers: &mut Vec<SubPatternBuffer>,
        distances_table: &HashMap<(NodeIndex, NodeIndex), i32>)
    {
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
    pub fn new(prev_layer: P, pattern: &'p Pattern, sub_patterns: &'p Vec<SubPattern>, window_size: usize) -> Self {
        let distances_table = pattern.order.calculate_distances().unwrap();
        let mut sub_pattern_buffers = Vec::with_capacity(2 * sub_patterns.len() - 1);

        sub_pattern_buffers.push(SubPatternBuffer::new(0, 1, &sub_patterns[0]));

        for i in 1..sub_patterns.len() {
            Self::create_buffer_pair(i, pattern, sub_patterns, &mut sub_pattern_buffers, &distances_table);
        }

        Self {
            prev_layer,
            num_sub_patterns: sub_patterns.len(),
            sub_pattern_buffers,
            patterns_matched: Vec::new(),
            window_size
        }
    }

    fn add_to_answer(&mut self) {
        /// Check whether match_edges is ok (has duplicate or not?)

    }

    fn get_left_buffer_id(buffer_id: usize) -> usize {
        buffer_id - buffer_id % 2
    }

    fn get_sibling_id(buffer_id: usize) -> usize {
        buffer_id ^ 1
    }

    fn get_parent_id(buffer_id: usize) -> usize {
        Self::get_left_buffer_id(buffer_id) + 2
    }

    fn clear_expired(&self, current_time: u64, buffer_id: usize) {
        while self.sub_pattern_buffers[buffer_id].buffer.get()
    }

    /// My new_match_buffer, joined with sibling's buffer.
    fn join_with_sibling(&self, my_id: usize, sibling_id: usize) -> Vec<SubPatternMatch>
    {
        let mut matches_to_parent = Vec::new();
        for sub_pattern_match1 in &self.sub_pattern_buffers[my_id].new_match_buffer {
            for sub_pattern_match2 in &self.sub_pattern_buffers[sibling_id].buffer {
                if self.sub_pattern_buffers[my_id].relation.check(sub_pattern_match1, sub_pattern_match2) {
                    let mut merged = SubPatternMatch::merge_matches(sub_pattern_match1, sub_pattern_match2);

                    matches_to_parent.push(
                        SubPatternMatch {
                            id: 0,
                            timestamp: min(sub_pattern_match1.timestamp, sub_pattern_match2.timestamp),
                            matched_nodes_table: merged.matched_nodes_table,
                            matched_edges_table: merged.matched_edges_table,
                            match_edges: merged.match_edges,
                        }
                    )
                }
            }
        }
        matches_to_parent
    }
    fn join(&mut self, current_time: u64, mut buffer_id: usize) {
        loop {
            let mut new_matches = &self.sub_pattern_buffers[buffer_id].new_match_buffer;

            let parent_id = Self::get_parent_id(buffer_id);
            self.sub_pattern_buffers[buffer_id].buffer.append(&mut new_matches);
            self.clear_expired(current_time, Self::get_sibling_id(buffer_id));

            self.sub_pattern_buffers[parent_id].new_match_buffer.append(&mut self.join_with_sibling(buffer_id, Self::get_sibling_id(buffer_id)));


            /// out of bound
            if parent_id >= self.sub_pattern_buffers.len() - 1 {
                Self::add_to_answer();
                break;
            }

            buffer_id = parent_id;
        }
    }
}

impl<'p, P> Iterator for JoinLayer<'p, P>
where
    P: Iterator<Item = Vec<SubPatternMatch<'p>>>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Self::Item> {
        while self.patterns_matched.is_empty() {
            let mut sub_pattern_matches = self.prev_layer.next()?;
            for sub_pattern_match in sub_pattern_matches {
                self.sub_pattern_buffers[sub_pattern_match.id].new_match_buffer.push(sub_pattern_match);
            }

            for buffer_id in 0..self.sub_pattern_buffers.len() {
                let new_match_buffer = &self.sub_pattern_buffers[buffer_id].new_match_buffer;
                if !(new_match_buffer.is_empty()) {
                    let current_time = new_match_buffer[0].timestamp;
                    self.join(current_time, buffer_id);
                }
            }
        }
        todo!()
    }
}
