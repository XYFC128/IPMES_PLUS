use crate::match_edge::MatchEdge;
use crate::process_layers::join_layer::SubPatternBuffer;
use itertools::Itertools;
use std::cmp::Ordering;
use std::cmp::{max, min};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct SubPatternMatch<'p> {
    pub id: usize,
    /// The timestamp of the last edge (in "match_edges"), which is also the latest timestamp; indicating "current time".
    pub latest_time: u64,
    /// The timestamp of the earliest edge; for determining expiry.
    pub earliest_time: u64,

    /// (input node id, pattern node id)
    /// match_nodes.len() == number of nodes in this sun-pattern match
    pub match_nodes: Vec<(u64, u64)>,

    /// "edge_id_map[matched_id] = input_id"
    /// The term "matched edge" and "pattern edge" is used interchangeably.
    /// edge_id_map.len() == number of edges in the "whole pattern"
    pub edge_id_map: Vec<Option<u64>>,

    /// sort this by 'input edge id' for uniqueness determination
    pub match_edges: Vec<MatchEdge<'p>>,
}

/// Since pattern-edges in sub-patterns are disjoint, we need not check uniqueness.
fn merge_edge_id_map(
    edge_id_map1: &Vec<Option<u64>>,
    edge_id_map2: &Vec<Option<u64>>,
) -> Vec<Option<u64>> {
    let mut edge_id_map = vec![None; edge_id_map1.len()];
    for i in 0..edge_id_map1.len() {
        match edge_id_map1[i] {
            Some(T) => edge_id_map[i] = Some(T),
            None => match edge_id_map2[i] {
                Some(T) => edge_id_map[i] = Some(T),
                None => (),
            },
        }
    }
    edge_id_map
}

fn check_edge_uniqueness(match_edges: &Vec<MatchEdge>) -> bool {
    let mut prev_id = u64::MAX;
    for edge in match_edges {
        if edge.input_edge.id == prev_id {
            return false;
        }
        prev_id = edge.input_edge.id;
    }
    true
}

impl<'p> SubPatternMatch<'p> {
    /// todo: check correctness
    pub fn merge_matches(
        sub_pattern_buffer: &mut SubPatternBuffer<'p>,
        sub_pattern_match1: &Self,
        sub_pattern_match2: &Self,
    ) -> Option<Self> {
        /// merge "match_edges" (WITHOUT checking "edge uniqueness")
        let match_edges = sub_pattern_buffer.try_merge_match_edges(
            &sub_pattern_match1.match_edges,
            &sub_pattern_match2.match_edges,
        )?;

        /// handle "edge uniqueness"
        if !check_edge_uniqueness(&match_edges) {
            return None;
        }

        /// check "order relation"
        if !sub_pattern_buffer.relation.check_order_relation(
            &sub_pattern_buffer.timestamps,
        ) {
            return None;
        }

        /// handle "shared node" and "node uniqueness"
        let mut match_nodes = sub_pattern_buffer.try_merge_nodes(
            &sub_pattern_match1.match_nodes,
            &sub_pattern_match1.match_nodes,
        )?;

        /// merge "edge_id_map"
        let edge_id_map = merge_edge_id_map(
            &sub_pattern_match1.edge_id_map,
            &sub_pattern_match2.edge_id_map,
        );

        /// clear workspace
        sub_pattern_buffer.clear_workspace();

        Some(SubPatternMatch {
            /// 'id' is meaningless here
            id: 0,
            latest_time: max(
                sub_pattern_match1.latest_time,
                sub_pattern_match2.latest_time,
            ),
            earliest_time: min(
                sub_pattern_match1.earliest_time,
                sub_pattern_match2.earliest_time,
            ),
            match_nodes,
            edge_id_map,
            match_edges,
        })
    }
}

#[derive(Clone)]
pub struct EarliestFirst<'p>(pub SubPatternMatch<'p>);

impl Eq for EarliestFirst<'_> {}

impl PartialEq<Self> for EarliestFirst<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.earliest_time.eq(&other.0.earliest_time)
    }
}

impl Ord for EarliestFirst<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.earliest_time.cmp(&other.0.earliest_time).reverse()
    }
}

impl PartialOrd<Self> for EarliestFirst<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}