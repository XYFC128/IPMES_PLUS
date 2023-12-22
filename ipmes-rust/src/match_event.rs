use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::input_event::InputEvent;
use crate::pattern::Event as PatternEvent;

#[derive(Clone)]
pub struct MatchEvent<'p> {
    pub input_event: Rc<InputEvent>,
    pub matched: &'p PatternEvent,
}

impl<'p> MatchEvent<'p> {
}

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