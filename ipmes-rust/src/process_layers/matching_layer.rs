use std::rc::Rc;

use crate::{
    input_event::InputEvent, match_event::MatchEvent, pattern::Event as PatternEvent,
    sub_pattern::SubPattern,
};
use regex::Error as RegexError;
use regex::Regex;

pub struct MatchingLayer<'p, P> {
    prev_layer: P,
    /// All pattern events in the total order
    pattern_events: Vec<&'p PatternEvent>,
    /// Regex matchers for the corresponding pattern event
    signatures: Vec<Regex>,

    // internal states of iterator
    /// The index of signature we wnat to match last time `next()` is called
    signature_state: usize,
    /// The index of input edge we wnat to match last time `next()` is called
    time_batch_state: usize,
    /// The current time batch (input events with the same timestamp)
    cur_time_batch: Vec<Rc<InputEvent>>,
}

impl<'p, P> MatchingLayer<'p, P> {
    pub fn new(
        prev_layer: P,
        decomposition: &'p [SubPattern],
        use_regex: bool,
    ) -> Result<Self, RegexError> {
        let mut pattern_events: Vec<&PatternEvent> = vec![];
        let mut signatures = vec![];
        for sub_pattern in decomposition {
            for pattern_event in &sub_pattern.events {
                pattern_events.push(pattern_event);

                // the regex expression should match whole string, so we add ^ and $ to the front and
                // object of the expression.
                let match_syntax = if use_regex {
                    format!("^{}$", pattern_event.signature)
                } else {
                    // if disable regex matching, simply escape meta characters in the string
                    format!("^{}$", regex::escape(&pattern_event.signature))
                };
                let signature = Regex::new(&match_syntax)?;
                signatures.push(signature);
            }
        }

        let signature_state = signatures.len() - 1;
        let time_batch_state = 0;

        Ok(Self {
            prev_layer,
            pattern_events,
            signatures,
            signature_state,
            time_batch_state,
            cur_time_batch: vec![],
        })
    }

    fn is_match_state(&self) -> bool {
        self.signatures[self.signature_state]
            .is_match(&self.cur_time_batch[self.time_batch_state].signature)
    }
}

impl<'p, P> Iterator for MatchingLayer<'p, P>
where
    P: Iterator<Item = Vec<Rc<InputEvent>>>,
{
    type Item = (MatchEvent<'p>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.time_batch_state + 1 >= self.cur_time_batch.len() {
                if self.signature_state + 1 >= self.signatures.len() {
                    self.cur_time_batch = self.prev_layer.next()?;
                    self.signature_state = 0;
                } else {
                    self.signature_state += 1;
                }
                self.time_batch_state = 0;
            } else {
                self.time_batch_state += 1;
            }

            if self.is_match_state() {
                return Some((
                    MatchEvent {
                        input_event: Rc::clone(&self.cur_time_batch[self.time_batch_state]),
                        matched: self.pattern_events[self.signature_state],
                    },
                    self.signature_state,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_input_edge(id: u64, signature: &str) -> Rc<InputEvent> {
        Rc::new(InputEvent {
            timestamp: 0,
            signature: signature.to_string(),
            id,
            subject: 1,
            object: 2,
        })
    }

    #[test]
    fn test_no_regex() {
        let pattern_edge1 = PatternEvent {
            id: 0,
            signature: "edge[0-9]+".to_string(),
            subject: 0,
            object: 1,
        };

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern_edge1],
        };
        let decomposition = [sub_pattern];

        let time_batch = vec![
            simple_input_edge(1, "edge[0-9]+"),
            simple_input_edge(2, "edge1234"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "ed"),
        ];

        let mut layer =
            MatchingLayer::new([time_batch].into_iter(), &decomposition, false).unwrap();

        assert_eq!(layer.next().unwrap().0.input_event.id, 1);
        assert!(layer.next().is_none());
    }

    #[test]
    fn test_regex() {
        let pattern_edge1 = PatternEvent {
            id: 0,
            signature: "edge[0-9]+".to_string(),
            subject: 0,
            object: 1,
        };

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern_edge1],
        };
        let decomposition = [sub_pattern];

        let time_batch = vec![
            simple_input_edge(1, "edge[0-9]+"),
            simple_input_edge(2, "edge1234"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "ed"),
        ];

        let mut layer = MatchingLayer::new([time_batch].into_iter(), &decomposition, true).unwrap();

        assert_eq!(layer.next().unwrap().0.input_event.id, 2);
        assert_eq!(layer.next().unwrap().0.input_event.id, 3);
        assert!(layer.next().is_none());
    }

    #[test]
    fn test_reorder() {
        let pattern_edge1 = PatternEvent {
            id: 0,
            signature: "edge1".to_string(),
            subject: 0,
            object: 1,
        };

        let pattern_edge2 = PatternEvent {
            id: 1,
            signature: "edge2".to_string(),
            subject: 1,
            object: 0,
        };

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern_edge1, &pattern_edge2],
        };
        let decomposition = [sub_pattern];

        let time_batch = vec![
            simple_input_edge(1, "edge0"),
            simple_input_edge(2, "edge2"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "edge2"),
        ];

        let mut layer =
            MatchingLayer::new([time_batch].into_iter(), &decomposition, false).unwrap();
        assert_eq!(layer.next().unwrap().0.input_event.id, 3);
        assert_eq!(layer.next().unwrap().0.input_event.id, 2);
        assert_eq!(layer.next().unwrap().0.input_event.id, 4);
        assert!(layer.next().is_none());
    }

    #[test]
    fn test_multiple_time_batch() {
        let pattern_edge1 = PatternEvent {
            id: 0,
            signature: "edge1".to_string(),
            subject: 0,
            object: 1,
        };

        let pattern_edge2 = PatternEvent {
            id: 1,
            signature: "edge2".to_string(),
            subject: 1,
            object: 0,
        };

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern_edge1, &pattern_edge2],
        };
        let decomposition = [sub_pattern];

        let time_batch1 = vec![
            simple_input_edge(1, "edge0"),
            simple_input_edge(2, "edge2"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "edge2"),
        ];

        let time_batch2 = vec![
            simple_input_edge(5, "edge2"),
            simple_input_edge(6, "edge0"),
            simple_input_edge(7, "edge1"),
        ];

        let mut layer = MatchingLayer::new(
            [time_batch1, time_batch2].into_iter(),
            &decomposition,
            false,
        )
        .unwrap();
    
        assert_eq!(layer.next().unwrap().0.input_event.id, 3);
        assert_eq!(layer.next().unwrap().0.input_event.id, 2);
        assert_eq!(layer.next().unwrap().0.input_event.id, 4);
        assert_eq!(layer.next().unwrap().0.input_event.id, 7);
        assert_eq!(layer.next().unwrap().0.input_event.id, 5);
        assert!(layer.next().is_none());
    }
}
