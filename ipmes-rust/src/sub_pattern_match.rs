use crate::input_edge::InputEdge;
use crate::match_edge::MatchEdge;
use crate::process_layers::join_layer::SubPatternBuffer;
use itertools::Itertools;
use std::cmp::Ordering;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone)]
pub struct SubPatternMatch<'p> {
    pub id: usize,
    /// The timestamp of the last edge (in "match_edges"), which is also the latest timestamp; indicating "current time".
    pub latest_time: u64,
    /// The timestamp of the earliest edge; for determining expiry.
    pub earliest_time: u64,

    /// (input node id, pattern node id)
    pub node_id_map: Vec<(u64, u64)>,
    /// "edge_id_map[matched_id] = input_id"
    pub edge_id_map: Vec<Option<u64>>,

    /// sort this by id for uniqueness determination
    pub match_edges: Vec<MatchEdge<'p>>,
}

impl SubPatternMatch {
    /// todo: check correctness
    fn merge_edge_id(sub_pattern_match1: &Self, sub_pattern_match2: &Self) -> Vec<Option<u64>> {
        let mut edge_id_map = vec![None, sub_pattern_match1.edge_id_map.len()];
        for i in 0..sub_pattern_match1.edge_id_map.len() {
            match sub_pattern_match1.edge_id_map[i] {
                Some(T) => edge_id_map[i] = Some(T),
                None => match sub_pattern_match2.edge_id_map[i] {
                    Some(T) => edge_id_map[i] = Some(T),
                    None => (),
                },
            }
        }
        edge_id_map
    }
    /// todo: check correctness
    pub fn merge_matches(
        sub_pattern_buffer: &SubPatternBuffer,
        sub_pattern_match1: &Self,
        sub_pattern_match2: &Self,
    ) -> Option<Self> {
        /// handle ordering relation
        if !sub_pattern_buffer
            .relation
            .check_order_relation(sub_pattern_match1, sub_pattern_match2)
        {
            return None;
        }

        let merged_edges = sub_pattern_match1
            .match_edges
            .iter()
            .cloned()
            .merge_by(sub_pattern_match2.match_edges.iter().cloned(), |a, b| {
                a.input_edge.id < b.input_edge.id
            })
            .collect_vec();

        /// handle "edge uniqueness"
        let mut prev_id = -1;
        for edge in merged_edges {
            if edge.input_edge.id == prev_id {
                return None;
            }
            prev_id = edge.input_edge.id;
        }

        /// handle "shared node" and "node uniqueness"
        /// Use relation.shared_node
        /// MORE CHECKS (NOT FINISHED)
        /// todo: if shared_node[i] = true, there must exist a pair!
        let mut node_id_map = vec![];
        let mut j = 0;
        for (i, (input_node_id, pattern_node_id)) in
            sub_pattern_match1.node_id_map.iter().enumerate()
        {
            while j < sub_pattern_match2.node_id_map.len() {
                let (input_id1, pattern_id1) = &sub_pattern_match1.node_id_map[i];
                let (input_id2, pattern_id2) = &sub_pattern_match2.node_id_map[i];

                if input_id2 < input_id1 {
                    node_id_map.push(sub_pattern_match2.node_id_map[j]);
                    j += 1;
                } else if input_id2 == input_id1 {
                    /// The input node is mapped to the same pattern node, and the node is indeed should be shared.
                    if pattern_id2 == pattern_id1
                        && sub_pattern_buffer
                            .relation
                            .is_node_shared(pattern_id2)
                    {
                        break;
                    } else {
                        return None;
                    }
                }
            }
            node_id_map.push(sub_pattern_match1.node_id_map[i]);
        }
        node_id_map.extend(&sub_pattern_match2.node_id_map[j..]);

        let edge_id_map = Self::merge_edge_id(sub_pattern_match1, sub_pattern_match2);

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
            node_id_map,
            edge_id_map,
            match_edges: merged_edges,
        })
    }
}

#[derive(Debug)]
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

impl<'p> AsRef<SubPatternMatch<'p>> for EarliestFirst<'p> {
    fn as_ref(&self) -> &SubPatternMatch<'p> {
        self.0.as_ref()
    }
}
