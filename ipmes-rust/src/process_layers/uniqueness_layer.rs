use crate::pattern_match::EarliestFirst;
use crate::pattern_match::PatternMatch;
use log::debug;
use std::collections::{BinaryHeap, HashSet};

/// The layer that handles pattern match uniqueness.
pub struct UniquenessLayer<P> {
    prev_layer: P,
    /// See `flush_expired()`.
    window_size: u64,
    /// A priority queue of pattern matches, where the earliest pattern match is at the top of the queue.
    pattern_match_sequence: BinaryHeap<EarliestFirst>,
    /// A pool which is used to maintain the uniqueness of pattern matches.
    uniqueness_pool: HashSet<PatternMatch>,
    /// Unique pattern matches which are ready for the next layer.
    unique_matches: Vec<PatternMatch>,
}

impl<P> UniquenessLayer<P> {
    pub fn new(prev_layer: P, window_size: u64) -> Self {
        Self {
            prev_layer,
            window_size,
            pattern_match_sequence: BinaryHeap::new(),
            uniqueness_pool: HashSet::new(),
            unique_matches: Vec::new(),
        }
    }
    /// Flush expired pattern matches.
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
        debug!(
            "After flushing: {} pattern matches",
            self.uniqueness_pool.len()
        );
    }
}

impl<P> Iterator for UniquenessLayer<P>
where
    P: Iterator<Item = PatternMatch>,
{
    type Item = PatternMatch;
    fn next(&mut self) -> Option<Self::Item> {
        while self.unique_matches.is_empty() {
            debug!("no instance available yet");
            if let Some(pattern_match) = self.prev_layer.next() {
                let latest_time = pattern_match.latest_time;
                self.flush_expired(latest_time);
                if !self.uniqueness_pool.contains(&pattern_match) {
                    self.uniqueness_pool.insert(pattern_match.clone());
                    self.pattern_match_sequence
                        .push(EarliestFirst(pattern_match));
                }
                debug!("size of uniqueness_pool: {}", self.uniqueness_pool.len());
            } else {
                debug!("prev layer no stuff, flush all");
                self.flush_expired(u64::MAX);
                break;
            }
        }
        self.unique_matches.pop()
    }
}
