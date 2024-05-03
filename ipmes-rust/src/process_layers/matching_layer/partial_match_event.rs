use std::rc::Rc;

use crate::input_event::InputEvent;
use crate::pattern::PatternEvent;

#[derive(Debug)]
pub struct PartialMatchEvent<'p> {
    pub matched: &'p PatternEvent,
    pub input_event: Rc<InputEvent>,
    pub match_ord: usize,
    pub start_time: u64,
    pub subject_id: u64,
}
