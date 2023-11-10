use crate::match_edge::MatchEdge;
use crate::process_layers::join_layer::SubPatternBuffer;
use itertools::Itertools;
use std::cmp::Ordering;
use std::cmp::{max, min};
use std::collections::HashMap;

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
    // A workplace for "used" in "try_merge_nodes" and for holding timestamps of input edges in "merge_edge_id()".
    // pub workplace: Vec<u64>,
}

impl<'p> SubPatternMatch<'p> {
    /// todo: check correctness
    pub fn merge_matches(
        sub_pattern_buffer: &mut SubPatternBuffer,
        sub_pattern_match1: &Self,
        sub_pattern_match2: &Self,
    ) -> Option<Self> {
        /// merge "match_edges" with checking "edge uniqueness"
        let match_edges = Self::try_merge_match_edges(
            &sub_pattern_match1.match_edges,
            &sub_pattern_match2.match_edges,
            // sub_pattern_match1.edge_id_map.len(),
            &mut sub_pattern_buffer.timestamps
        )?;

        /// check "order relation"
        if !sub_pattern_buffer.relation.check_order_relation(
            &sub_pattern_match1.edge_id_map,
            &sub_pattern_match2.edge_id_map,
            &sub_pattern_buffer.timestamps,
        ) {
            return None;
        }

        /// handle "shared node" and "node uniqueness"
        let mut match_nodes = Self::try_merge_nodes(
            &sub_pattern_match1.match_nodes,
            &sub_pattern_match1.match_nodes,
            &mut sub_pattern_buffer.used_nodes
        )?;

        /// merge "edge_id_map"
        let edge_id_map = Self::merge_edge_id_map(sub_pattern_match1, sub_pattern_match2);

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

    /// "merge match_edge" and "check edge uniqueness"
    /// Analogous to "try_merge_nodes"
    /// todo: Check correctness. And it may be modified such that "timestamps" need not be returned.
    fn try_merge_match_edges(
        a: &[MatchEdge<'p>],
        b: &[MatchEdge<'p>],
        // max_num_edges: usize,
        timestamps: &mut HashMap<u64, u64>
    ) -> Option<Vec<MatchEdge<'p>>> {
        let (mut p1, mut p2) = if a.len() > b.len() {
            (a.iter(), b.iter())
        } else {
            (b.iter(), a.iter())
        };

        // '0' means timestamp not recorded
        // let mut timestamps = vec![0; max_num_edges];
        let mut merged = Vec::with_capacity(a.len() + b.len());

        let mut next1 = p1.next();
        let mut next2 = p2.next();
        while let (Some(edge1), Some(edge2)) = (next1, next2) {
            if timestamps.get(&edge1.input_edge.id) == None || timestamps.get(&edge2.input_edge.id)  == None {
                return None;
            }

            if edge1.input_edge.id < edge2.input_edge.id {
                merged.push(edge1.clone());
                timestamps.insert(edge1.input_edge.id, edge1.input_edge.timestamp);
                next1 = p1.next();
            } else if edge1.input_edge.id > edge2.input_edge.id {
                merged.push(edge2.clone());
                timestamps.insert(edge2.input_edge.id, edge2.input_edge.timestamp);
                next2 = p2.next();
            } else {
                if edge1.matched.id != edge2.matched.id {
                    return None;
                }
                merged.push(edge1.clone());
                timestamps.insert(edge1.input_edge.id, edge1.input_edge.timestamp);
                next1 = p1.next();
                next2 = p2.next();
            }
        }
        for edge in p1 {
            if timestamps.get(&edge.input_edge.id) == None {
                return None;
            }
            timestamps[edge.input_edge.id as usize] = edge.input_edge.timestamp;
            merged.push(edge.clone());
        }

        Some(merged)
    }

    /// todo: Write tests to check correctness
    /// Since pattern-edges in sub-patterns are disjoint, we need not check uniqueness.
    fn merge_edge_id_map(sub_pattern_match1: &Self, sub_pattern_match2: &Self) -> Vec<Option<u64>> {
        let mut edge_id_map = vec![None; sub_pattern_match1.edge_id_map.len()];
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

    /// Try to merge match nodes, and handle "shared node" and "node uniqueness" in the process.
    /// If the mentioned checks didn't pass, return None.
    ///
    /// a and b are slices over (input node id, pattern node id)
    /// todo: Write tests to check correctness
    fn try_merge_nodes(
        a: &[(u64, u64)],
        b: &[(u64, u64)],
        // max_num_nodes: usize,
        used: &mut Vec<bool>
    ) -> Option<Vec<(u64, u64)>> {
        let (mut p1, mut p2) = if a.len() > b.len() {
            (a.iter(), b.iter())
        } else {
            (b.iter(), a.iter())
        };

        // let mut used = vec![false; max_num_nodes];
        let mut merged = Vec::with_capacity(a.len() + b.len());

        let mut next1 = p1.next();
        let mut next2 = p2.next();
        while let (Some(node1), Some(node2)) = (next1, next2) {
            if used[node1.1 as usize] || used[node2.1 as usize] {
                return None;
            }

            if node1.0 < node2.0 {
                merged.push(node1.clone());
                used[node1.1 as usize] = true;
                next1 = p1.next();
            } else if node1.0 > node2.0 {
                merged.push(node2.clone());
                used[node2.1 as usize] = true;
                next2 = p2.next();
            } else {
                if node1.1 != node2.1 {
                    return None;
                }
                merged.push(node1.clone());
                used[node1.1 as usize] = true;
                next1 = p1.next();
                next2 = p2.next();
            }
        }
        for node in p1 {
            if used[node.1 as usize] {
                return None;
            }
            used[node.1 as usize] = true;
            merged.push(node.clone());
        }

        Some(merged)
    }
}

// #[derive(Debug)]
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

// impl<'p> AsRef<SubPatternMatch<'p>> for EarliestFirst<'p> {
//     fn as_ref(&self) -> &SubPatternMatch<'p> {
//         self.0.as_ref()
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {

    }
}