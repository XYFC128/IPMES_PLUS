mod entry;
mod entry_wrappers;

// #[cfg(test)]
// mod tests;

use crate::match_event::MatchEvent;
use crate::pattern::Pattern;
use crate::pattern_match::PatternMatch;
use crate::process_layers::composition_layer::PartialMatch;
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
        let place_holder = Rc::new(Entry::placeholder(pattern.num_entities));
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
            if result.match_events.len() == self.pattern.events.len() {
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
    /// 1. shared node & event
    /// 2. order relation
    /// 3. event & node uniqueness
    /// If any of the check failed, return None
    fn try_merge(&self, entry1: &Entry<'p>, entry2: &Entry<'p>) -> Option<Entry<'p>> {
        if !self.check_shared_node(entry1, entry2) {
            return None;
        }

        let merged_edges = entry1
            .match_events
            .iter()
            .cloned()
            .merge_by(entry2.match_events.iter().cloned(), |a, b| {
                a.input_event.id < b.input_event.id
            })
            .collect_vec();

        let mapping = self.get_mapping(&merged_edges)?;

        if !self.check_order_relation(&entry1, &entry2, &mapping) {
            return None;
        }

        // todo: check node uniqueness

        let merged_nodes = self.merge_nodes(&entry1.match_entities, &entry2.match_entities);
        let hash = UniqueEntry::calc_hash(&mapping);
        Some(Entry {
            earliest_time: min(entry1.earliest_time, entry2.earliest_time),
            match_events: merged_edges,
            match_entities: merged_nodes,
            hash,
        })
    }

    fn try_merge_edges(
        &self,
        a: &[MatchEvent<'p>],
        b: &[MatchEvent<'p>],
    ) -> Option<Vec<MatchEvent<'p>>> {
        let (mut p1, mut p2) = if a.len() > b.len() {
            (a.iter(), b.iter())
        } else {
            (b.iter(), a.iter())
        };

        let mut mapping = vec![None; self.pattern.events.len()];
        let mut merged = Vec::new();

        let mut next1 = p1.next();
        let mut next2 = p2.next();
        while let (Some(edge1), Some(edge2)) = (next1, next2) {
            if edge1.input_event.id < edge2.input_event.id {
                if mapping[edge1.matched.id].is_some() {
                    return None;
                }
                merged.push(edge1.clone());
                mapping[edge1.matched.id] = Some(edge1.input_event.timestamp);
                next1 = p1.next();
            } else {
                if mapping[edge2.matched.id].is_some() {
                    return None;
                }

                if edge1.input_event.id == edge2.input_event.id {
                    if edge1.matched.id != edge2.matched.id {
                        return None;
                    }
                    next1 = p1.next();
                }
                merged.push(edge2.clone());
                mapping[edge2.matched.id] = Some(edge2.input_event.timestamp);
                next2 = p2.next();
            }
        }

        while let Some(event) = next1 {
            if mapping[event.matched.id].is_some() {
                return None;
            }
            merged.push(event.clone());
            mapping[event.matched.id] = Some(event.input_event.timestamp);
            next1 = p1.next();
        }

        Some(merged)
    }

    /// Check whether input nodes in different entries that match the same pattern node are also
    /// the same input nodes.
    fn check_shared_node(&self, entry1: &Entry<'p>, entry2: &Entry<'p>) -> bool {
        zip(&entry1.match_entities, &entry2.match_entities).all(|pair| {
            if let (Some(n1), Some(n2)) = pair {
                n1 == n2
            } else {
                pair.0.is_none() && pair.1.is_none()
            }
        })
    }

    /// Get the mapping from pattern event id to match event. The mapping is stored in a vector,
    /// with the index as the id of pattern event.
    ///
    /// This function returns [None] when:
    /// 1. detects 2 input events with the same id
    /// 2. detects 2 input events matches to the same pattern event
    ///
    /// This means it is responsible for checking shared event and event uniqueness
    fn get_mapping<'a>(
        &self,
        merged_edges: &'a [MatchEvent<'p>],
    ) -> Option<Vec<Option<&'a MatchEvent<'p>>>> {
        let mut pattern_match = vec![None; self.pattern.events.len()];
        let mut prev_id = u64::MAX;
        for event in merged_edges {
            if event.input_event.id == prev_id {
                return None; // 2 input events with the same id
            }
            prev_id = event.input_event.id;

            todo!("This check is not right");
            if pattern_match[event.matched.id].is_some() {
                return None; // 2 input events matches to the same pattern event
            }
            pattern_match[event.matched.id] = Some(event);
        }

        Some(pattern_match)
    }

    fn check_order_relation(
        &self,
        entry1: &Entry<'p>,
        entry2: &Entry<'p>,
        pattern_match: &[Option<&MatchEvent<'p>>],
    ) -> bool {
        let events = if entry1.match_events.len() < entry2.match_events.len() {
            &entry1.match_events
        } else {
            &entry2.match_events
        };

        for edge1 in events {
            for prev_id in self.pattern.order.get_previous(edge1.matched.id) {
                if let Some(Some(edge2)) = pattern_match.get(prev_id) {
                    if edge2.input_event.timestamp > edge1.input_event.timestamp {
                        return false;
                    }
                }
            }

            for next_id in self.pattern.order.get_next(edge1.matched.id) {
                if let Some(Some(edge2)) = pattern_match.get(next_id) {
                    if edge2.input_event.timestamp < edge1.input_event.timestamp {
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
                let mut match_entities = vec![None; self.pattern.num_entities];
                let mut earliest_time = u64::MAX;
                for event in &sub_match.events {
                    match_entities[event.matched.subject] = Some(event.input_event.subject);
                    match_entities[event.matched.object] = Some(event.input_event.object);

                    earliest_time = min(event.input_event.timestamp, earliest_time);
                }

                if let Some(mapping) = self.get_mapping(&sub_match.events) {
                    let hash = UniqueEntry::calc_hash(&mapping);
                    let entry = Rc::new(Entry {
                        earliest_time,
                        match_events: sub_match.events,
                        match_entities,
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
