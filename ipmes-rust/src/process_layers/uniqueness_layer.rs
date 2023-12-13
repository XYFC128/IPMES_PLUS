use std::collections::{BinaryHeap, HashSet};
use itertools::Itertools;
use crate::pattern_match::PatternMatch;
use crate::pattern_match::EarliestFirst;

pub struct UniquenessLayer<P> {
    prev_layer: P,
    window_size: u64,
    pattern_match_sequence: BinaryHeap<EarliestFirst>,
    uniqueness_pool: HashSet<PatternMatch>,
    unique_matches: Vec<PatternMatch>
}

impl<P> UniquenessLayer<P> {
    pub fn new(prev_layer: P, window_size: u64) -> Self {
        Self {
            prev_layer,
            window_size,
            pattern_match_sequence: BinaryHeap::new(),
            uniqueness_pool: HashSet::new(),
            unique_matches: Vec::new()
        }
    }
    fn flush_expired(&mut self, latest_time: u64) {
        while let Some(pattern_match) = self.pattern_match_sequence.peek() {
            if latest_time.saturating_sub(self.window_size) > pattern_match.0.earliest_time {
                let item = self.pattern_match_sequence.pop().unwrap().0;
                self.uniqueness_pool.remove(&item);
                self.unique_matches.push(item);
            } else {
                break;
            }
        }
    }
}

impl<P> Iterator for UniquenessLayer<P>
where
    P: Iterator<Item = PatternMatch>,
{
    type Item = PatternMatch;
    fn next(&mut self) -> Option<Self::Item> {
        while self.unique_matches.is_empty() {
            let pattern_match = self.prev_layer.next()?;
            let latest_time = pattern_match.latest_time;
            self.flush_expired(latest_time);
            if !self.uniqueness_pool.contains(&pattern_match) {
                self.uniqueness_pool.insert(pattern_match.clone());
                self.pattern_match_sequence.push(EarliestFirst(pattern_match));
            }
        }
        self.unique_matches.pop()
    }
}