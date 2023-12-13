use std::rc::Rc;
use crate::input_event::InputEvent;
use crate::pattern::Event as PatternEvent;

#[derive(Clone, Debug)]
pub struct MatchEvent<'p> {
    pub input_event: Rc<InputEvent>,
    pub matched: &'p PatternEvent,
}

impl<'p> MatchEvent<'p> {
}