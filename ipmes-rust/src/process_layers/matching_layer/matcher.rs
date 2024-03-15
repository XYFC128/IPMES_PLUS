use regex::Error as RegexError;
use regex::Regex;

use crate::input_event::InputEvent;
use crate::pattern::PatternEntity;
use crate::pattern::PatternEvent;
use crate::pattern::PatternEventType;

pub struct Matcher {
    event_signature: Regex,
    subject_signature: Regex,
    object_signature: Regex,
}

fn construct_regex(pattern: &str, escape_regex: bool) -> Result<Regex, RegexError> {
    let match_syntax = if escape_regex {
        format!("^{}$", regex::escape(pattern))
    } else {
        format!("^{}$", pattern)
    };
    Regex::new(&match_syntax)
}

impl Matcher {
    pub fn new(pattern: &PatternEvent, subject: &PatternEntity, object: &PatternEntity, use_regex: bool) -> Result<Self, RegexError> {
        if pattern.event_type == PatternEventType::Flow {
            Ok(Self {
                event_signature: Regex::new(".*")?,
                subject_signature: Regex::new(".*")?,
                object_signature: Regex::new(".*")?,
            })
        } else {
            Ok(Self {
                event_signature: construct_regex(&pattern.signature, !use_regex)?,
                subject_signature: construct_regex(&subject.signature, !use_regex)?,
                object_signature: construct_regex(&object.signature, !use_regex)?,
            })
        }
    }
    /// Return true if and only if signatures of input event and its endpoints matches the given pattern.
    pub fn is_match(&self, input: &InputEvent) -> bool {
        let event_match = self.event_signature.is_match(&input.event_signature);
        let subject_match = self.subject_signature.is_match(&input.subject_signature);
        let object_match = self.object_signature.is_match(&input.object_signature);

        event_match && subject_match && object_match
    }
}