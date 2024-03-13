use crate::input_event::InputEvent;
use crate::pattern::PatternEvent;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

/// The structure that pairs up an input event with the pattern event it matches.
#[derive(Clone)]
pub struct MatchEvent<'p> {
    /// An reference-counting pointer the an input event.
    pub input_event: Rc<InputEvent>,
    /// The matched pattern event of this input event.
    pub matched: &'p PatternEvent,
}

impl<'p> MatchEvent<'p> {}

impl Debug for MatchEvent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[({}, {}), {}, {}, {}]",
            self.input_event.id,
            self.matched.id,
            self.input_event.timestamp,
            self.input_event.subject,
            self.input_event.object,
        )
    }
}

pub trait EventAttributes {
    fn get_timestamp(&self) -> u64;
    fn get_signature(&self) -> &str;
    fn get_id(&self) -> u64;
    fn get_subject(&self) -> u64;
    fn get_object(&self) -> u64;
    fn get_matched(&self) -> &PatternEvent;
}

impl<'p> EventAttributes for MatchEvent<'p> {
    fn get_timestamp(&self) -> u64 {
        self.input_event.timestamp
    }

    fn get_signature(&self) -> &str {
        &self.input_event.signature
    }

    fn get_id(&self) -> u64 {
        self.input_event.id
    }

    fn get_subject(&self) -> u64 {
        self.input_event.subject
    }

    fn get_object(&self) -> u64 {
        self.input_event.object
    }

    fn get_matched(&self) -> &PatternEvent {
        self.matched
    }
}
