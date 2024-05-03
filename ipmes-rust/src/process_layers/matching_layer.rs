mod matcher;
mod partial_match_event;

use crate::pattern::{Pattern, PatternEventType};
use crate::{input_event::InputEvent, sub_pattern::SubPattern};
use matcher::DefaultMatcher;
pub use partial_match_event::PartialMatchEvent;
use regex::Error as RegexError;
use std::rc::Rc;

use self::matcher::{FlowMatcher, Matcher};

pub struct MatchingLayer<'p, P> {
    prev_layer: P,
    /// Regex matchers for the corresponding pattern event
    matchers: Vec<Box<dyn Matcher<'p> + 'p>>,

    // internal states of iterator
    /// The index of signature we want to match last time `next()` is called
    matcher_state: usize,
    /// The index of input edge we want to match last time `next()` is called
    time_batch_state: usize,
    /// The current time batch (input events with the same timestamp)
    cur_time_batch: Vec<Rc<InputEvent>>,
}

impl<'p, P> MatchingLayer<'p, P> {
    pub fn new(
        prev_layer: P,
        pattern: &'p Pattern,
        decomposition: &[SubPattern<'p>],
        window_size: u64,
    ) -> Result<Self, RegexError> {
        let mut matchers = vec![];
        for sub_pattern in decomposition {
            for pattern_event in &sub_pattern.events {
                let matcher: Box<dyn Matcher<'p> + 'p> = match pattern_event.event_type {
                    PatternEventType::Flow => Box::new(FlowMatcher::new(
                        pattern_event,
                        pattern.use_regex,
                        window_size,
                    )?),
                    _ => Box::new(DefaultMatcher::new(pattern_event, pattern.use_regex)?),
                };
                matchers.push(matcher);
            }
        }

        let matcher_state = matchers.len() - 1;
        let time_batch_state = 0;

        Ok(Self {
            prev_layer,
            matchers,
            matcher_state,
            time_batch_state,
            cur_time_batch: vec![],
        })
    }

    fn get_match(&mut self) -> Option<PartialMatchEvent<'p>> {
        let matcher = &mut self.matchers[self.matcher_state];
        let input = &self.cur_time_batch[self.time_batch_state];
        matcher.get_match(input)
    }
}

impl<'p, P> Iterator for MatchingLayer<'p, P>
where
    P: Iterator<Item = Vec<Rc<InputEvent>>>,
{
    type Item = PartialMatchEvent<'p>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.time_batch_state == self.cur_time_batch.len() {
                if self.matcher_state + 1 == self.matchers.len() {
                    self.cur_time_batch = self.prev_layer.next()?;
                    self.matcher_state = 0;
                } else {
                    self.matcher_state += 1;
                }
                self.time_batch_state = 0;
            }

            if let Some(mut result) = self.get_match() {
                result.match_ord = self.matcher_state;
                return Some(result);
            }
            self.time_batch_state += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_input_edge(id: u64, signature: &str) -> Rc<InputEvent> {
        Rc::new(InputEvent {
            timestamp: 0,
            event_id: id,
            event_signature: signature.to_string(),
            subject_id: 1,
            subject_signature: "u".to_string(),
            object_id: 2,
            object_signature: "v".to_string(),
        })
    }

    #[test]
    fn test_no_regex() {
        let pattern = Pattern::from_graph(&["u", "v"], &[(0, 1, "edge[0-9]+")], false);

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern.events[0]],
        };
        let decomposition = [sub_pattern];

        let time_batch = vec![
            simple_input_edge(1, "edge[0-9]+"),
            simple_input_edge(2, "edge1234"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "ed"),
        ];

        let mut layer =
            MatchingLayer::new([time_batch].into_iter(), &pattern, &decomposition, u64::MAX)
                .unwrap();

        assert_eq!(layer.next().unwrap().input_event.event_id, 1);
        assert!(layer.next().is_none());
    }

    #[test]
    fn test_regex() {
        let pattern = Pattern::from_graph(&["u", "v"], &[(0, 1, "edge[0-9]+")], true);

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern.events[0]],
        };
        let decomposition = [sub_pattern];

        let time_batch = vec![
            simple_input_edge(1, "edge[0-9]+"),
            simple_input_edge(2, "edge1234"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "ed"),
        ];

        let mut layer =
            MatchingLayer::new([time_batch].into_iter(), &pattern, &decomposition, u64::MAX)
                .unwrap();

        assert_eq!(layer.next().unwrap().input_event.event_id, 2);
        assert_eq!(layer.next().unwrap().input_event.event_id, 3);
        assert!(layer.next().is_none());
    }

    #[test]
    fn test_reorder() {
        let pattern = Pattern::from_graph(&["u", "v"], &[(0, 1, "edge1"), (0, 1, "edge2")], false);

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern.events[0], &pattern.events[1]],
        };
        let decomposition = [sub_pattern];

        let time_batch = vec![
            simple_input_edge(1, "edge0"),
            simple_input_edge(2, "edge2"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "edge2"),
        ];

        let mut layer =
            MatchingLayer::new([time_batch].into_iter(), &pattern, &decomposition, u64::MAX)
                .unwrap();
        assert_eq!(layer.next().unwrap().input_event.event_id, 3);
        assert_eq!(layer.next().unwrap().input_event.event_id, 2);
        assert_eq!(layer.next().unwrap().input_event.event_id, 4);
        assert!(layer.next().is_none());
    }

    #[test]
    fn test_multiple_time_batch() {
        let pattern = Pattern::from_graph(&["u", "v"], &[(0, 1, "edge1"), (0, 1, "edge2")], false);

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern.events[0], &pattern.events[1]],
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
            &pattern,
            &decomposition,
            u64::MAX,
        )
        .unwrap();

        assert_eq!(layer.next().unwrap().input_event.event_id, 3);
        assert_eq!(layer.next().unwrap().input_event.event_id, 2);
        assert_eq!(layer.next().unwrap().input_event.event_id, 4);
        assert_eq!(layer.next().unwrap().input_event.event_id, 7);
        assert_eq!(layer.next().unwrap().input_event.event_id, 5);
        assert!(layer.next().is_none());
    }
}
