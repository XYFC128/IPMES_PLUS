use crate::input_event::InputEvent;
use itertools::Itertools;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::rc::Rc;

/// Complete Pattern Match
#[derive(Clone, Debug)]
pub struct PatternMatch {
    /// Matched edges of this pattern. `match_events[i]` is the input edge that matches pattern edge `i`.
    pub matched_events: Vec<Rc<InputEvent>>,
    /// The timestamp of the last event (in `matched_events`), which is also the latest timestamp; indicating "current time".
    pub latest_time: u64,
    /// The timestamp of the earliest event; for determining expiry of this match.
    pub earliest_time: u64,
}

impl fmt::Display for PatternMatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.matched_events.iter().map(|e| e.event_id).join(", ")
        )
    }
}

impl Eq for PatternMatch {}

impl PartialEq for PatternMatch {
    fn eq(&self, other: &Self) -> bool {
        zip(&self.matched_events, &other.matched_events).all(|(a, b)| a.event_id.eq(&b.event_id))
    }
}

impl Hash for PatternMatch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for edge in &self.matched_events {
            edge.event_id.hash(state);
        }
    }
}

/// Helper structure that implements `PartialEq`, `Ord`, `PartialOrd` traits for `PatternMatch`.
///
/// *Earliest* refers to `PatternMatch.earliest_time`.
#[derive(Clone)]
pub struct EarliestFirst(pub PatternMatch);
impl Eq for EarliestFirst {}

impl PartialEq<Self> for EarliestFirst {
    fn eq(&self, other: &Self) -> bool {
        self.0.earliest_time.eq(&other.0.earliest_time)
    }
}

impl Ord for EarliestFirst {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.earliest_time.cmp(&other.0.earliest_time).reverse()
    }
}

impl PartialOrd<Self> for EarliestFirst {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
