use std::collections::HashSet;
use std::collections::VecDeque;
use std::rc::Rc;

use regex::Error as RegexError;
use regex::Regex;

use crate::input_event::InputEvent;
use crate::pattern::PatternEvent;

use super::PartialMatchEvent;

pub trait Matcher<'p> {
    /// If the input event match the matchers requirement, returns ([PartialMatchEvent], `more`). Otherwise, return [None].
    ///
    /// If the second return value `more` is true, the caller should call this method again with
    /// the same input until the returned `more` becomes false or return [None].
    fn get_match(&mut self, input: &Rc<InputEvent>) -> Option<(PartialMatchEvent<'p>, bool)>;
}

/// Helper function to construct regex object matching the whole input.
fn construct_regex(pattern: &str, escape_regex: bool) -> Result<Regex, RegexError> {
    let match_syntax = if escape_regex {
        format!("^{}$", regex::escape(pattern))
    } else {
        format!("^{}$", pattern)
    };
    Regex::new(&match_syntax)
}

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

struct ReachableSet {
    start_id: u64,
    start_time: u64,
    reachable_set: HashSet<u64>,
}

pub struct FlowMatcher<'p> {
    matched: &'p PatternEvent,
    window_size: u64,
    subject_signature: Regex,
    object_signature: Regex,
    reachable_sets: VecDeque<ReachableSet>,
    is_root: HashSet<u64>,

    next_state: usize,
    subject_match: bool,
    object_match: bool,
}

impl<'p> FlowMatcher<'p> {
    pub fn new(
        matched: &'p PatternEvent,
        use_regex: bool,
        window_size: u64,
    ) -> Result<Self, RegexError> {
        Ok(Self {
            matched,
            window_size,
            subject_signature: construct_regex(&matched.subject.signature, !use_regex)?,
            object_signature: construct_regex(&matched.object.signature, !use_regex)?,
            reachable_sets: VecDeque::new(),
            is_root: HashSet::new(),
            next_state: 0,
            subject_match: false,
            object_match: false,
        })
    }

    fn windowing(&mut self, latest_time: u64) {
        let window_bound = latest_time.saturating_sub(self.window_size);
        while let Some(front) = self.reachable_sets.front() {
            if front.start_time >= window_bound {
                break;
            }
            self.is_root.remove(&front.start_id);
            self.reachable_sets.pop_front();
        }
    }

    fn new_reachable_set(&mut self, input: &InputEvent) {
        let mut reachable_set = HashSet::new();
        reachable_set.insert(input.subject_id);
        reachable_set.insert(input.object_id);
        self.reachable_sets.push_back(ReachableSet {
            start_id: input.subject_id,
            start_time: input.timestamp,
            reachable_set,
        });
        self.is_root.insert(input.subject_id);
    }
}

impl<'p> Matcher<'p> for FlowMatcher<'p> {
    fn get_match(&mut self, input: &Rc<InputEvent>) -> Option<(PartialMatchEvent<'p>, bool)> {
        if self.next_state == 0 {
            // this is an new input event
            self.subject_match = self
                .subject_signature
                .is_match(input.get_subject_signature());
            self.object_match = self.object_signature.is_match(input.get_object_signature());

            self.windowing(input.timestamp);
        }

        while self.next_state < self.reachable_sets.len() {
            let reach = &mut self.reachable_sets[self.next_state];
            let head_in_set = reach.reachable_set.contains(&input.subject_id);
            self.next_state += 1;

            if head_in_set {
                reach.reachable_set.insert(input.object_id);
                if self.object_match {
                    return Some((
                        PartialMatchEvent {
                            matched: self.matched,
                            match_ord: 0,
                            start_time: reach.start_time,
                            subject_id: reach.start_id,
                            input_event: Rc::clone(input),
                        },
                        true,
                    ));
                }
            }
        }

        // already visit all reachable sets in this matcher

        let not_root = !self.is_root.contains(&input.subject_id);
        if self.subject_match && not_root {
            self.new_reachable_set(input);
        }

        self.next_state = 0;
        if self.object_match && self.subject_match {
            Some((
                PartialMatchEvent {
                    matched: self.matched,
                    match_ord: 0,
                    start_time: input.timestamp,
                    subject_id: input.subject_id,
                    input_event: Rc::clone(input),
                },
                false,
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pattern::{PatternEntity, PatternEventType};

    use super::*;

    fn setup_flow_matcher(pattern: &PatternEvent) -> FlowMatcher<'_> {
        FlowMatcher {
            matched: pattern,
            window_size: 10,
            subject_signature: construct_regex("u", false).unwrap(),
            object_signature: construct_regex("v", false).unwrap(),
            reachable_sets: VecDeque::new(),
            is_root: HashSet::new(),
            next_state: 0,
            subject_match: false,
            object_match: false,
        }
    }

    #[test]
    fn test_simple_flow() {
        let pattern = PatternEvent {
            id: 0,
            event_type: PatternEventType::Flow,
            signature: "".to_string(),
            subject: PatternEntity {
                id: 0,
                signature: "".to_string(),
            },
            object: PatternEntity {
                id: 1,
                signature: "".to_string(),
            },
        };

        let input1 = Rc::new(InputEvent::new(1, 0, "", 0, "u", 1, "x"));
        let input2 = Rc::new(InputEvent::new(1, 0, "", 1, "x", 2, "x"));
        let input3 = Rc::new(InputEvent::new(3, 0, "", 2, "x", 3, "v"));

        let mut matcher = setup_flow_matcher(&pattern);
        assert!(matcher.get_match(&input1).is_none());
        assert!(matcher.get_match(&input2).is_none());
        assert!(matcher.get_match(&input3).is_some());
    }

    #[test]
    fn test_single_event_flow() {
        let pattern = PatternEvent {
            id: 0,
            event_type: PatternEventType::Flow,
            signature: "".to_string(),
            subject: PatternEntity {
                id: 0,
                signature: "".to_string(),
            },
            object: PatternEntity {
                id: 1,
                signature: "".to_string(),
            },
        };

        let input1 = Rc::new(InputEvent::new(1, 0, "", 0, "u", 1, "v"));

        let mut matcher = setup_flow_matcher(&pattern);
        assert!(matcher.get_match(&input1).is_some());
    }
}
