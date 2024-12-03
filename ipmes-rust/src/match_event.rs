use crate::input_event::InputEvent;
use crate::pattern::PatternEvent;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::match_event::RawEvents::{Flow, Multiple, Single};

#[derive(Clone, Debug)]
pub struct MatchEvent {
    /// The pattern event id that `self.raw_events` are matched to.
    pub match_id: u32,
    /// The subject id of the **input event** (`raw_events`)
    pub input_subject_id: u64,
    /// The object id of the **input event** (`raw_events`)
    pub input_object_id: u64,
    /// The subject id of the matched pattern event
    pub pattern_subject_id: u64,
    /// The object id of the matched pattern event
    pub pattern_object_id: u64,
    /// Input events
    pub raw_events: RawEvents,
}

#[derive(Clone, Debug)]
pub enum RawEvents {
    Single(Rc<InputEvent>),
    /// Correspond to `Frequency` match type
    Multiple(Box<[Rc<InputEvent>]>),
    Flow(u64, u64), // start_time, end_time
}

impl RawEvents {
    pub fn get_ids<'p>(
        &'p self,
    ) -> Box<dyn Iterator<Item = u64> + 'p> {
        match self {
            Single(event) => Box::new(Some(event.event_id).into_iter()),

            Multiple(events) => Box::new(events.iter().map(|e| e.event_id)),

            Flow(_, _) => Box::new(None.into_iter()),
        }
    }

    pub fn get_interval(&self) -> (u64, u64) {
        match self {
            Single(event) => (event.timestamp, event.timestamp),

            Multiple(events) => {
                let first = events.first().unwrap();
                let last = events.last().unwrap();
                (first.timestamp, last.timestamp)
            }

            Flow(start_time, end_time) => (*start_time, *end_time),
        }
    }
}
