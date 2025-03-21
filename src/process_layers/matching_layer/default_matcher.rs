use super::matcher::{construct_regex, Matcher};
use super::PartialMatchEvent;
use crate::input_event::InputEvent;
use crate::pattern::PatternEvent;
use regex::Error as RegexError;
use regex::Regex;
use std::rc::Rc;

pub struct DefaultMatcher<'p> {
    regex_matcher: Regex,
    matched: &'p PatternEvent,
}

impl<'p> DefaultMatcher<'p> {
    pub fn new(pattern: &'p PatternEvent, use_regex: bool) -> Result<Self, RegexError> {
        let regex_pattern = format!(
            "{}\0{}\0{}",
            pattern.signature, pattern.subject.signature, pattern.object.signature
        );
        Ok(Self {
            regex_matcher: construct_regex(&regex_pattern, !use_regex)?,
            matched: pattern,
        })
    }

    /// Return true if and only if signatures of input event and its endpoints matches the given pattern.
    pub fn is_match(&self, input: &InputEvent) -> bool {
        self.regex_matcher.is_match(input.get_signatures())
    }
}

impl<'p> Matcher<'p> for DefaultMatcher<'p> {
    fn get_match(&mut self, input: &Rc<InputEvent>) -> Option<(PartialMatchEvent<'p>, bool)> {
        if self.is_match(input) {
            Some((
                PartialMatchEvent {
                    matched: self.matched,
                    match_ord: 0,
                    subject_id: input.subject_id,
                    start_time: input.timestamp,
                    input_event: Rc::clone(input),
                },
                false,
            ))
        } else {
            None
        }
    }
}
