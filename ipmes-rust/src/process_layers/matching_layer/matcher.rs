use super::PartialMatchEvent;
use crate::input_event::InputEvent;
use regex::Error as RegexError;
use regex::Regex;
use std::rc::Rc;

pub trait Matcher<'p> {
    /// If the input event match the matchers requirement, returns ([PartialMatchEvent], `more`). Otherwise, return [None].
    ///
    /// If the second return value `more` is true, the caller should call this method again with
    /// the same input until the returned `more` becomes false or return [None].
    fn get_match(&mut self, input: &Rc<InputEvent>) -> Option<(PartialMatchEvent<'p>, bool)>;
}

/// Helper function to construct regex object matching the whole input.
pub fn construct_regex(pattern: &str, escape_regex: bool) -> Result<Regex, RegexError> {
    let match_syntax = if escape_regex {
        format!("^{}$", regex::escape(pattern))
    } else {
        format!("^{}$", pattern)
    };
    Regex::new(&match_syntax)
}
