use regex::Regex;

use std::rc::Rc;
use super::event_matcher::EventMatcher;
use crate::input_event::InputEvent;
use crate::match_event::MatchEvent;
use crate::pattern::Event as PatternEvent;

pub struct NormalEventMatcher<'p> {
    pattern: &'p PatternEvent,
    signature: Regex,
}

impl<'p> EventMatcher<'p> for NormalEventMatcher<'p> {
    fn match_all<F>(&mut self, time_batch: &[Rc<InputEvent>], mut callback: F)
    where
        F: FnMut(MatchEvent<'p>),
    {
        for input in time_batch {
            if self.signature.is_match(&input.signature) {
                callback(MatchEvent {
                    input_event: Rc::clone(input),
                    matched: self.pattern,
                });
            }
        }
    }
}
