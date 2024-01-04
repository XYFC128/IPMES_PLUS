use std::rc::Rc;

use crate::{input_event::InputEvent, match_event::MatchEvent};

pub trait EventMatcher<'p> {
    /// Match all input edge in a time batch. If a match is found, the MatchEvent is
    /// created and passed to callback.
    fn match_all<F>(&mut self, time_batch: &[Rc<InputEvent>], callback: F)
    where
        F: FnMut(MatchEvent<'p>);
}
