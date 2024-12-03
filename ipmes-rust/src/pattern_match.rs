use itertools::Itertools;
use nix::libc::write;
use std::cmp::Ordering;
use std::fmt::Formatter;
use std::fmt::{self};
use std::hash::Hash;
use std::rc::Rc;

use crate::match_event::{MatchEvent, RawEvents};
use crate::process_layers::composition_layer::match_instance::InputEventId;

/// Complete Pattern Match
#[derive(Clone, Debug)]
pub struct PatternMatch {
    /// The timestamp of the last event (in `matched_events`), which is also the latest timestamp; indicating "current time".
    pub latest_time: u64,
    /// The timestamp of the earliest event; for determining expiry of this match.
    pub earliest_time: u64,

    pub event_ids: Box<[InputEventId]>,
    pub match_event_map: Box<[Option<Rc<MatchEvent>>]>,
}

impl Hash for PatternMatch {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.event_ids.hash(state);
    }
}

impl Eq for PatternMatch {}

impl PartialEq for PatternMatch {
    fn eq(&self, other: &Self) -> bool {
        self.event_ids == other.event_ids
    }
}

impl fmt::Display for PatternMatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let event_str = self.match_event_map.iter().flatten().map(|match_event| {
            match &match_event.raw_events {
                RawEvents::Single(input_event) => {
                    input_event.event_id.to_string()
                }
                RawEvents::Multiple(input_events) => {
                    format!("({})", input_events.iter().map(|e| e.event_id).join(", "))
                }
                RawEvents::Flow(_, _) => {
                    format!("({} -> {})", match_event.input_subject_id, match_event.input_object_id)
                }
            }
        }).join(", ");
        let start_t = self.earliest_time as f32 / 1000.0;
        let end_t = self.latest_time as f32 / 1000.0;
        write!(f, "<{start_t:.3}, {end_t:.3}>[{event_str}]")
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
