use crate::input_event::InputEvent;
use crate::pattern::PatternEvent;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::match_event::RawEvents::{Flow, Multiple, Single};

/// The structure that pairs up an input event with the pattern event it matches.
// #[derive(Clone)]
// pub struct MatchEvent<'p> {
//     /// An reference-counting pointer the an input event.
//     pub input_event: Rc<InputEvent>,
//     /// The matched pattern event of this input event.
//     pub matched: &'p PatternEvent,
// }

#[derive(Clone)]
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

#[derive(Clone)]
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

// impl<'p> MatchEvent<'p> {}

// impl Debug for MatchEvent<'_> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "[({}, {}), {}, {}, {}]",
//             self.input_event.event_id,
//             self.matched.id,
//             self.input_event.timestamp,
//             self.input_event.subject_id,
//             self.input_event.object_id,
//         )
//     }
// }

// pub trait EventAttributes {
//     fn get_timestamp(&self) -> u64;
//     fn get_signature(&self) -> &str;
//     fn get_id(&self) -> u64;
//     fn get_subject(&self) -> u64;
//     fn get_object(&self) -> u64;
//     fn get_matched(&self) -> &PatternEvent;
// }

// impl<'p> EventAttributes for MatchEvent<'p> {
//     fn get_timestamp(&self) -> u64 {
//         self.input_event.timestamp
//     }

//     fn get_signature(&self) -> &str {
//         self.input_event.get_event_signature()
//     }

//     fn get_id(&self) -> u64 {
//         self.input_event.event_id
//     }

//     fn get_subject(&self) -> u64 {
//         self.input_event.subject_id
//     }

//     fn get_object(&self) -> u64 {
//         self.input_event.object_id
//     }

//     fn get_matched(&self) -> &PatternEvent {
//         self.matched
//     }
// }
