mod entry;
mod entry_wrappers;

#[cfg(test)]
mod tests;

use crate::match_edge::MatchEdge;
use crate::pattern::Pattern;
use crate::pattern_match::PatternMatch;
use crate::process_layers::ord_match_layer::PartialMatch;
use crate::sub_pattern_match::SubPatternMatch;
use entry::Entry;
use entry_wrappers::{EarliestFirst, UniqueEntry};
use itertools::Itertools;
use std::cmp::min;
use std::collections::{BinaryHeap, HashSet};
use std::iter::zip;
use std::rc::Rc;

pub struct NaiveJoinLayer<'p, P> {
    prev_layer: P,
    pattern: &'p Pattern,
    unique_entries: HashSet<UniqueEntry<'p>>,
    table: BinaryHeap<EarliestFirst<'p>>,
    full_matches: Vec<PatternMatch>,
    time_window: u64,
}

impl<'p, P> NaiveJoinLayer<'p, P> {
    fn new(prev_layer: P, pattern: &'p Pattern, time_window: u64) -> Self {
        let place_holder = Rc::new(Entry::placeholder(pattern.num_nodes));
        let mut unique_entries = HashSet::new();
        unique_entries.insert(UniqueEntry(Rc::clone(&place_holder)));
        let mut table = BinaryHeap::new();
        table.push(EarliestFirst(place_holder));

        Self {
            prev_layer,
            pattern,
            unique_entries,
            table,
            full_matches: Vec::new(),
            time_window,
        }
    }

    /// Remove the entries where their timestamp is earlier than `time_bound`.
    fn clear_expired(&mut self, time_bound: u64) {
        while let Some(first) = self.table.peek() {
            if first.0.earliest_time >= time_bound {
                break;
            }

            let first = self.table.pop().unwrap();
            self.unique_entries.remove(&UniqueEntry(first.0));
        }
    }

    /// Add entry to the pool and perform join
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
            // check if all pattern edges are matched
            if result.match_edges.len() == self.pattern.edges.len() {
                self.full_matches.push(result.into());
                continue;
            }

            let result = Rc::new(result);
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
    fn try_merge(&self, entry1: &Entry<'p>, entry2: &Entry<'p>) -> Option<Entry<'p>> {
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

        let mapping = self.get_mapping(&merged_edges)?;

        if !self.check_order_relation(&entry1, &entry2, &mapping) {
            return None;
        }

        // todo: check node uniqueness

        let merged_nodes = self.merge_nodes(&entry1.match_nodes, &entry2.match_nodes);
        let hash = UniqueEntry::calc_hash(&mapping);
        Some(Entry {
            earliest_time: min(entry1.earliest_time, entry2.earliest_time),
            match_edges: merged_edges,
            match_nodes: merged_nodes,
            hash,
        })
    }

    fn try_merge_edges(
        &self,
        a: &[MatchEdge<'p>],
        b: &[MatchEdge<'p>],
    ) -> Option<Vec<MatchEdge<'p>>> {
        let (mut p1, mut p2) = if a.len() > b.len() {
            (a.iter(), b.iter())
        } else {
            (b.iter(), a.iter())
        };

        let mut mapping = vec![None; self.pattern.edges.len()];
        let mut merged = Vec::new();

        let mut next1 = p1.next();
        let mut next2 = p2.next();
        while let (Some(edge1), Some(edge2)) = (next1, next2) {
            if edge1.input_edge.id < edge2.input_edge.id {
                if mapping[edge1.matched.id].is_some() {
                    return None;
                }
                merged.push(edge1.clone());
                mapping[edge1.matched.id] = Some(edge1.input_edge.timestamp);
                next1 = p1.next();
            } else {
                if mapping[edge2.matched.id].is_some() {
                    return None;
                }

                if edge1.input_edge.id == edge2.input_edge.id {
                    if edge1.matched.id != edge2.matched.id {
                        return None;
                    }
                    next1 = p1.next();
                }
                merged.push(edge2.clone());
                mapping[edge2.matched.id] = Some(edge2.input_edge.timestamp);
                next2 = p2.next();
            }
        }

        while let Some(edge) = next1 {
            if mapping[edge.matched.id].is_some() {
                return None;
            }
            merged.push(edge.clone());
            mapping[edge.matched.id] = Some(edge.input_edge.timestamp);
            next1 = p1.next();
        }

        Some(merged)
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
    fn get_mapping<'a>(
        &self,
        merged_edges: &'a [MatchEdge<'p>],
    ) -> Option<Vec<Option<&'a MatchEdge<'p>>>> {
        let mut pattern_match = vec![None; self.pattern.edges.len()];
        let mut prev_id = u64::MAX;
        for edge in merged_edges {
            if edge.input_edge.id == prev_id {
                return None; // 2 input edges with the same id
            }
            prev_id = edge.input_edge.id;

            todo!("This check is not right");
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
    P: Iterator<Item = Vec<PartialMatch<'p>>>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Self::Item> {
        while self.full_matches.is_empty() {
            let sub_pattern_matches = self.prev_layer.next()?;

            if let Some(sub_match) = sub_pattern_matches.last() {
                self.clear_expired(sub_match.timestamp.saturating_sub(self.time_window));
            } else {
                continue;
            }

            for sub_match in sub_pattern_matches {
                let mut match_nodes = vec![None; self.pattern.num_nodes];
                let mut earliest_time = u64::MAX;
                for edge in &sub_match.edges {
                    match_nodes[edge.matched.start] = Some(edge.input_edge.start);
                    match_nodes[edge.matched.end] = Some(edge.input_edge.end);

                    earliest_time = min(edge.input_edge.timestamp, earliest_time);
                }

                if let Some(mapping) = self.get_mapping(&sub_match.edges) {
                    let hash = UniqueEntry::calc_hash(&mapping);
                    let entry = Rc::new(Entry {
                        earliest_time,
                        match_edges: sub_match.edges,
                        match_nodes,
                        hash,
                    });
                    self.add_entry(entry);
                } else {
                    // dirty subpattern match
                    continue;
                }
            }
        }
        self.full_matches.pop()
    }
}
