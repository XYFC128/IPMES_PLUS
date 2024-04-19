use crate::{input_event::InputEvent, pattern::PatternEvent};
use std::rc::Rc;

#[derive(Debug)]
pub struct PartialMatchEvent<'p> {
    pub matched: &'p PatternEvent,
    pub input_event: Rc<InputEvent>,
    pub match_ord: usize,
}
