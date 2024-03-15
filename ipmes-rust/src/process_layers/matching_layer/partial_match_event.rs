use std::rc::Rc;
use crate::{input_event::InputEvent, pattern::PatternEvent};

#[derive(Debug)]
pub struct PartialMatchEvent<'p> {
    pub matched: &'p PatternEvent,
    pub input_event: Rc<InputEvent>,
    pub match_ord: usize,
    pub match_type: MatchType,
}

#[derive(Debug)]
pub enum MatchType {
    Normal,
    Frequency,
    Flow,
}