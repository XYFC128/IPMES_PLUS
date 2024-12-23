use crate::input_event::InputEvent;
use crate::match_event::MatchEvent;
use crate::pattern::Event as PatternEvent;
use crate::sub_pattern::SubPattern;
use itertools::Itertools;
use log::warn;
use regex::Error as RegexError;
use regex::Regex;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::rc::Rc;

/// Internal representation of a not complete subpattern match
#[derive(Debug)]
pub struct PartialMatch<'p> {
    /// the id of the matched sub-pattern
    pub id: usize,
    /// the earliest timestamp
    pub timestamp: u64,
    /// Maps the id of entities in the pattern to the id of input entities
    ///
    /// For example, if the input entity `i` match the pattern entity `p`, then
    /// `entity_id_map[p] = Some(i)`.
    ///
    /// If the pattern entity `p` doesn't match any pattern in this partial match,
    /// then `entity_id_map[p] = None`.
    pub entity_id_map: Vec<Option<u64>>,
    pub events: Vec<MatchEvent<'p>>,
}

impl<'p> PartialMatch<'p> {
    /// Return `true` if the input entities in the partial match is unique.
    fn check_entity_uniqueness(&self) -> bool {
        let mut used = HashSet::new();
        for entity_match in &self.entity_id_map {
            if let Some(entity_id) = entity_match {
                if used.contains(entity_id) {
                    return false;
                } else {
                    used.insert(entity_id);
                }
            }
        }

        true
    }
}

/// Represent a small buffer for matching an edge
struct SubMatcher<'p> {
    /// regex matcher for the pattern signature
    signature: Regex,
    /// corresponding pattern edge
    pattern_edge: &'p PatternEvent,
    /// if this is the last buffer of a subpattern, sub_pattern_id will be the id of that subpattern
    /// , -1 otherwise
    sub_pattern_id: i64,
    /// buffer for the partial match results, dequeue is used for windowing
    buffer: VecDeque<PartialMatch<'p>>,
}

impl<'p> SubMatcher<'p> {
    pub fn new(pattern_edge: &'p PatternEvent, use_regex: bool) -> Result<Self, RegexError> {
        // the regex expression should match whole string, so we add ^ and $ to the front and
        // object of the expression.
        let match_syntax = if use_regex {
            format!("^{}$", pattern_edge.signature)
        } else {
            // if disable regex matching, simply escape meta characters in the string
            format!("^{}$", regex::escape(&pattern_edge.signature))
        };
        let signature = Regex::new(&match_syntax)?;

        Ok(Self {
            signature,
            pattern_edge,
            sub_pattern_id: -1,
            buffer: VecDeque::new(),
        })
    }
    pub fn match_against(&mut self, input_edges: &[Rc<InputEvent>]) -> Vec<PartialMatch<'p>> {
        input_edges
            .iter()
            .filter(|edge| self.signature.is_match(&edge.signature))
            .cartesian_product(self.buffer.iter())
            .filter_map(|(input_event, partial_match)| {
                self.merge(Rc::clone(input_event), partial_match)
            })
            .collect()
    }

    fn merge(
        &self,
        input_event: Rc<InputEvent>,
        partial_match: &PartialMatch<'p>,
    ) -> Option<PartialMatch<'p>> {
        if self.has_entity_collision(&input_event, partial_match)
            || Self::event_duplicates(&input_event, partial_match)
        {
            return None;
        }

        // duplicate the partial match and add the input edge into the new partial match
        let mut entity_id_map = partial_match.entity_id_map.clone();
        let mut events = partial_match.events.clone();
        entity_id_map[self.pattern_edge.subject] = Some(input_event.subject);
        entity_id_map[self.pattern_edge.object] = Some(input_event.object);

        let timestamp = min(input_event.timestamp, partial_match.timestamp);
        let match_edge = MatchEvent {
            input_event,
            matched: self.pattern_edge,
        };
        events.push(match_edge);

        Some(PartialMatch {
            id: partial_match.id,
            timestamp,
            entity_id_map,
            events,
        })
    }

    /// Return `true` if the input event's entities **do not match** the expected id in the partial match.
    ///
    /// That is, if the input event's subject (id = `x`) matches pattern entity `n0`, and `n0` in
    /// this partial match matches `y`, then `x` must equals to `y` for this input event to be merged
    /// into the partial match.
    fn has_entity_collision(
        &self,
        input_event: &InputEvent,
        partial_match: &PartialMatch<'p>,
    ) -> bool {
        if let Some(subject_match) = partial_match.entity_id_map[self.pattern_edge.subject] {
            if subject_match != input_event.subject {
                return true;
            }
        }

        if let Some(object_match) = partial_match.entity_id_map[self.pattern_edge.object] {
            if object_match != input_event.object {
                return true;
            }
        }

        false
    }

    /// Return `true` if the id of the input event already exist in the partial match.
    fn event_duplicates(input_event: &InputEvent, partial_match: &PartialMatch<'p>) -> bool {
        partial_match
            .events
            .iter()
            .find(|edge| edge.input_event.id == input_event.id)
            .is_some()
    }

    /// Clear the entries in the buffer which timestamp < time_bound
    pub fn clear_expired(&mut self, time_bound: u64) {
        while let Some(head) = self.buffer.front() {
            if head.timestamp < time_bound {
                self.buffer.pop_front();
            } else {
                break;
            }
        }
    }
}

pub struct CompositionLayer<'p, P> {
    prev_layer: P,
    window_size: u64,
    sub_matchers: Vec<SubMatcher<'p>>,
}

impl<'p, P> CompositionLayer<'p, P> {
    pub fn new(
        prev_layer: P,
        decomposition: &'p [SubPattern],
        use_regex: bool,
        window_size: u64,
    ) -> Result<Self, RegexError> {
        let mut sub_matchers = Vec::new();
        for sub_pattern in decomposition {
            // create sub-matcher for each edge
            for edge in &sub_pattern.events {
                sub_matchers.push(SubMatcher::new(edge, use_regex)?);
            }
            // get the first sub-matcher for this sub-pattern
            if let Some(first) = sub_matchers
                .iter_mut()
                .nth_back(sub_pattern.events.len() - 1)
            {
                // the entity_id_map only need to store up to the maximum node id nodes
                let max_node_id = sub_pattern
                    .events
                    .iter()
                    .map(|e| max(e.subject, e.object))
                    .max()
                    .unwrap();
                // Insert an empty partial match that is never expired. All the partial match for
                // sub pattern will be duplicated from this partial match
                first.buffer.push_back(PartialMatch {
                    id: sub_pattern.id,
                    timestamp: u64::MAX,
                    entity_id_map: vec![None; max_node_id + 1],
                    events: vec![],
                })
            }
            if let Some(last) = sub_matchers.last_mut() {
                last.sub_pattern_id = sub_pattern.id as i64;
            }
        }

        Ok(Self {
            prev_layer,
            window_size,
            sub_matchers,
        })
    }
}

impl<'p, P> Iterator for CompositionLayer<'p, P>
where
    P: Iterator<Item = Vec<Rc<InputEvent>>>,
{
    type Item = Vec<PartialMatch<'p>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut results = vec![];

        while results.is_empty() {
            let time_batch = self.prev_layer.next()?;
            let time_bound = if let Some(edge) = time_batch.first() {
                edge.timestamp.saturating_sub(self.window_size)
            } else {
                warn!("Previous layer outputs an empty time batch.");
                continue;
            };

            let mut prev_result = Vec::new();
            for matcher in &mut self.sub_matchers {
                matcher.clear_expired(time_bound);

                matcher.buffer.extend(prev_result.into_iter());
                let cur_result = matcher.match_against(&time_batch);
                if matcher.sub_pattern_id != -1 {
                    // this is the last buffer of a subpattern
                    results.extend(
                        cur_result
                            .into_iter()
                            .filter(|partial_match| partial_match.check_entity_uniqueness()),
                    );
                    prev_result = Vec::new();
                } else {
                    prev_result = cur_result;
                }
            }
        }

        Some(results)
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

    fn input_event(id: u64, signature: &str, subject: u64, object: u64) -> Rc<InputEvent> {
        Rc::new(InputEvent {
            timestamp: 0,
            signature: signature.to_string(),
            id,
            subject,
            object,
        })
    }
    #[test]
    fn test_sub_matcher_no_regex() {
        let pattern_edge = PatternEvent {
            id: 0,
            signature: "edge*".to_string(),
            subject: 0,
            object: 1,
        };

        let mut matcher = SubMatcher::new(&pattern_edge, false).unwrap();
        matcher.buffer.push_back(PartialMatch {
            id: 0,
            timestamp: u64::MAX,
            entity_id_map: vec![None; 2],
            events: vec![],
        });

        let input_edges = vec![
            simple_input_edge(1, "edge*"),
            simple_input_edge(2, "edgee"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "input_event"),
        ];

        let result = matcher.match_against(&input_edges);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].events[0].input_event.id, 1);
    }

    #[test]
    fn test_sub_matcher_regex() {
        let pattern_edge = PatternEvent {
            id: 0,
            signature: "edge*".to_string(),
            subject: 0,
            object: 1,
        };

        let mut matcher = SubMatcher::new(&pattern_edge, true).unwrap();
        matcher.buffer.push_back(PartialMatch {
            id: 0,
            timestamp: u64::MAX,
            entity_id_map: vec![None; 2],
            events: vec![],
        });

        let input_edges = vec![
            simple_input_edge(1, "edge*"),
            simple_input_edge(2, "edgee"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "input_event"),
        ];

        let result = matcher.match_against(&input_edges);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].events[0].input_event.id, 2);
    }

    #[test]
    fn test_composition_layer() {
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
            object: 2,
        };

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern_edge1, &pattern_edge2],
        };
        let decomposition = [sub_pattern];

        let time_batch = vec![
            input_event(2, "edge2", 2, 3),
            input_event(3, "foo", 4, 5),
            input_event(4, "bar", 6, 7),
            input_event(1, "edge1", 1, 2),
        ];

        let mut layer =
            CompositionLayer::new([time_batch].into_iter(), &decomposition, false, u64::MAX)
                .unwrap();
        let result = layer.next().unwrap();
        assert_eq!(result.len(), 1);
    }

    /// In this testcase, the pattern event 1 and 3 matches the same input edge 1,
    /// but (1, 2, 1) is not a valid match state, since the input edge 1 is duplicated.
    #[test]
    fn test_event_uniqueness() {
        let pattern_edge1 = PatternEvent {
            id: 0,
            signature: "a".to_string(),
            subject: 0,
            object: 1,
        };
        let pattern_edge2 = PatternEvent {
            id: 1,
            signature: "b".to_string(),
            subject: 1,
            object: 2,
        };
        let pattern_edge3 = PatternEvent {
            id: 2,
            signature: "a".to_string(),
            subject: 3,
            object: 1,
        };

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern_edge1, &pattern_edge2, &pattern_edge3],
        };
        let decomposition = [sub_pattern];

        let time_batch = vec![input_event(1, "a", 1, 2), input_event(2, "b", 2, 3)];

        let mut layer =
            CompositionLayer::new([time_batch].into_iter(), &decomposition, true, u64::MAX)
                .unwrap();
        assert!(layer.next().is_none());
    }

    #[test]
    fn test_entity_uniqueness() {
        let pattern_event1 = PatternEvent {
            id: 0,
            signature: "a".to_string(),
            subject: 0,
            object: 1,
        };
        let pattern_event2 = PatternEvent {
            id: 1,
            signature: "b".to_string(),
            subject: 1,
            object: 2,
        };

        let sub_pattern = SubPattern {
            id: 0,
            events: vec![&pattern_event1, &pattern_event2],
        };
        let decomposition = [sub_pattern];

        let time_batch = vec![
            input_event(1, "a", 1, 2),
            input_event(2, "b", 2, 1), // entity ID duplicated
        ];

        let mut layer =
            CompositionLayer::new([time_batch].into_iter(), &decomposition, true, u64::MAX)
                .unwrap();
        assert!(layer.next().is_none());
    }
}
