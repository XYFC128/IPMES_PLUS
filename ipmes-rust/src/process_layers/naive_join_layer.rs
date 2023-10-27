mod entry;
mod entry_wrappers;

use crate::match_edge::MatchEdge;
use crate::pattern::Pattern;
use crate::pattern_match::PatternMatch;
use crate::sub_pattern_match::SubPatternMatch;
use itertools::Itertools;
use std::cmp::min;
use std::collections::{BinaryHeap, HashSet};
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::ops::Deref;
use std::rc::Rc;
use entry::Entry;
use entry_wrappers::{EarliestFirst, UniqueEntry};

pub struct NaiveJoinLayer<'p, P> {
    prev_layer: P,
    pattern: &'p Pattern,
    unique_entries: HashSet<UniqueEntry<'p>>,
    table: BinaryHeap<EarliestFirst<'p>>,
    full_matches: Vec<PatternMatch>,
}

impl<'p, P> NaiveJoinLayer<'p, P> {
    fn new(prev_layer: P, pattern: &Pattern) -> Self {
        todo!()
    }

    fn add_entry(&mut self, entry: Rc<Entry<'p>>) {
        let unique_entry = UniqueEntry(Rc::clone(&entry));
        if self.unique_entries.contains(&unique_entry) {
            return;
        }

        let merge_result = self
            .table
            .iter()
            .filter_map(|other| self.try_merge(&entry, other.as_ref()))
            .collect_vec();
        for result in merge_result.into_iter() {
            let unique_result = UniqueEntry(Rc::clone(&result));
            if self.unique_entries.contains(&unique_result) {
                continue;
            }

            self.table.push(EarliestFirst(result));
            self.unique_entries.insert(unique_result);
        }
    }

    /// Merge entry1 and entry2. Return a new merged entry or [None]
    /// if they cannot be merged or the merge result is already in the pool.
    ///
    /// So this function will check for:
    /// 1. shared node & edge
    /// 2. order relation
    /// 3. edge & node uniqueness
    /// If any of the check failed, return None
    fn try_merge(&self, entry1: &Entry<'p>, entry2: &Entry<'p>) -> Option<Rc<Entry<'p>>> {
        if !self.check_shared_node(entry1, entry2) {
            return None;
        }

        let merged_edges = entry1
            .match_edges
            .iter()
            .cloned()
            .merge_by(entry2.match_edges.iter().cloned(), |a, b| {
                a.input_edge.id < b.input_edge.id
            })
            .collect_vec();

        let pattern_match = self.get_mapping(&merged_edges)?;

        if !self.check_order_relation(&entry1, &entry2, &pattern_match) {
            return None;
        }

        // todo: check node uniqueness

        let merged_nodes = self.merge_nodes(&entry1.match_nodes, &entry2.match_nodes);
        let hash = UniqueEntry::calc_hash(&pattern_match);
        Some(Rc::new(Entry {
            earliest_time: min(entry1.earliest_time, entry2.earliest_time),
            match_edges: merged_edges,
            match_nodes: merged_nodes,
            hash,
        }))
    }

    /// Check whether input nodes in different entries that match the same pattern node are also
    /// the same input nodes.
    fn check_shared_node(&self, entry1: &Entry<'p>, entry2: &Entry<'p>) -> bool {
        zip(&entry1.match_nodes, &entry2.match_nodes).all(|pair| {
            if let (Some(n1), Some(n2)) = pair {
                n1 == n2
            } else {
                pair.0.is_none() && pair.1.is_none()
            }
        })
    }

    /// Get the mapping from pattern edge id to match edge. The mapping is stored in a vector,
    /// with the index as the id of pattern edge.
    ///
    /// This function returns [None] when:
    /// 1. detects 2 input edges with the same id
    /// 2. detects 2 input edges matches to the same pattern edge
    ///
    /// This means it is responsible for checking shared edge and edge uniqueness
    fn get_mapping<'a>(&self, merged_edges: &'a [MatchEdge<'p>]) -> Option<Vec<Option<&'a MatchEdge<'p>>>> {
        let mut pattern_match = vec![None; self.pattern.edges.len()];
        let mut prev_id = u64::MAX;
        for edge in merged_edges {
            if edge.input_edge.id == prev_id {
                return None; // 2 input edges with the same id
            }
            prev_id = edge.input_edge.id;

            if pattern_match[edge.matched.id].is_some() {
                return None; // 2 input edges matches to the same pattern edge
            }
            pattern_match[edge.matched.id] = Some(edge);
        }

        Some(pattern_match)
    }

    fn check_order_relation(
        &self,
        entry1: &Entry<'p>,
        entry2: &Entry<'p>,
        pattern_match: &[Option<&MatchEdge<'p>>],
    ) -> bool {
        let edges = if entry1.match_edges.len() < entry2.match_edges.len() {
            &entry1.match_edges
        } else {
            &entry2.match_edges
        };

        for edge1 in edges {
            for prev_id in self.pattern.order.get_previous(edge1.matched.id) {
                if let Some(Some(edge2)) = pattern_match.get(prev_id) {
                    if edge2.input_edge.timestamp > edge1.input_edge.timestamp {
                        return false;
                    }
                }
            }

            for next_id in self.pattern.order.get_next(edge1.matched.id) {
                if let Some(Some(edge2)) = pattern_match.get(next_id) {
                    if edge2.input_edge.timestamp < edge1.input_edge.timestamp {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Merges the given two node mappings. Their length should be the same for the expansion table
    /// to work correctly.
    fn merge_nodes(&self, nodes1: &[Option<u64>], nodes2: &[Option<u64>]) -> Vec<Option<u64>> {
        zip(nodes1, nodes2)
            .map(|pair| {
                if pair.0.is_some() {
                    pair.0.clone()
                } else {
                    pair.1.clone()
                }
            })
            .collect_vec()
    }
}

impl<'p, P> Iterator for NaiveJoinLayer<'p, P>
where
    P: Iterator<Item = Vec<SubPatternMatch<'p>>>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Self::Item> {
        while self.full_matches.is_empty() {
            let sub_pattern_matches = self.prev_layer.next()?;
            for sub_match in sub_pattern_matches {
                todo!()
                // let match_nodes = vec![None; self.pattern.num_nodes];
                // for edge in &sub_match.match_edges {
                //
                // }
                // let entry = Rc::new(Entry::from(sub_match));
                // self.add_entry(entry);
            }
        }
        self.full_matches.pop()
    }
}
