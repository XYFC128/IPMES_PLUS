use crate::input_event::InputEvent;
use crate::match_event::MatchEvent;
use crate::sub_pattern::SubPattern;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::collections::VecDeque;

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
    /// Whether this is the last buffer of some subpattern
    is_last: bool,
    /// buffer for the partial match results, dequeue is used for windowing
    buffer: VecDeque<PartialMatch<'p>>,
}

impl<'p> SubMatcher<'p> {
    pub fn new() -> Self {
        Self {
            is_last: false,
            buffer: VecDeque::new(),
        }
    }

    pub fn match_against(&mut self, match_event: &MatchEvent<'p>) -> Vec<PartialMatch<'p>> {
        self.buffer
            .iter()
            .filter_map(|partial_match| self.merge(match_event, partial_match))
            .collect()
    }

    fn merge(
        &self,
        match_event: &MatchEvent<'p>,
        partial_match: &PartialMatch<'p>,
    ) -> Option<PartialMatch<'p>> {
        if self.has_entity_collision(match_event, partial_match)
            || Self::event_duplicates(&match_event.input_event, partial_match)
        {
            return None;
        }

        // duplicate the partial match and add the input edge into the new partial match
        let mut entity_id_map = partial_match.entity_id_map.clone();
        let mut events = partial_match.events.clone();
        entity_id_map[match_event.matched.subject.id] = Some(match_event.input_event.subject_id);
        entity_id_map[match_event.matched.object.id] = Some(match_event.input_event.object_id);

        let timestamp = min(match_event.input_event.timestamp, partial_match.timestamp);
        events.push(match_event.clone());

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
        match_event: &MatchEvent<'p>,
        partial_match: &PartialMatch<'p>,
    ) -> bool {
        if let Some(subject_match) = partial_match.entity_id_map[match_event.matched.subject.id] {
            if subject_match != match_event.input_event.subject_id {
                return true;
            }
        }

        if let Some(object_match) = partial_match.entity_id_map[match_event.matched.object.id] {
            if object_match != match_event.input_event.object_id {
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
            .find(|edge| edge.input_event.event_id == input_event.event_id)
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
    pub fn new(prev_layer: P, decomposition: &'p [SubPattern], window_size: u64) -> Self {
        let mut sub_matchers = Vec::new();
        for sub_pattern in decomposition {
            // create sub-matcher for each edge
            for _ in &sub_pattern.events {
                sub_matchers.push(SubMatcher::new());
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
                    .map(|e| max(e.subject.id, e.object.id))
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
            // get the last sub-matcher for this sub-pattern
            if let Some(last) = sub_matchers.last_mut() {
                last.is_last = true;
            }
        }

        Self {
            prev_layer,
            window_size,
            sub_matchers,
        }
    }
}

impl<'p, P> Iterator for CompositionLayer<'p, P>
where
    P: Iterator<Item = (MatchEvent<'p>, usize)>,
{
    type Item = Vec<PartialMatch<'p>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (match_event, match_order) = self.prev_layer.next()?;

            let sub_matcher = &mut self.sub_matchers[match_order];

            let time_bound = match_event
                .input_event
                .timestamp
                .saturating_sub(self.window_size);
            sub_matcher.clear_expired(time_bound);

            let mut result = sub_matcher.match_against(&match_event);
            if result.is_empty() {
                continue;
            }

            if sub_matcher.is_last {
                result.retain(|partial_match| partial_match.check_entity_uniqueness());
                if result.is_empty() {
                    continue;
                }
                return Some(result);
            } else {
                let next_matcher = &mut self.sub_matchers[match_order + 1];
                next_matcher.buffer.extend(result.into_iter());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::{PatternEntity, PatternEvent, PatternEventType};
    use std::rc::Rc;

    /// Create match event for testing purpose
    fn match_event<'p>(
        id: u64,
        signature: &str,
        subject: u64,
        object: u64,
        matched: &'p PatternEvent,
    ) -> MatchEvent<'p> {
        let input_event = Rc::new(InputEvent {
            timestamp: 0,
            event_id: id,
            event_signature: signature.to_string(),
            subject_id: subject,
            subject_signature: String::new(),
            object_id: object,
            object_signature: String::new(),
        });

        MatchEvent {
            input_event,
            matched,
        }
    }

    #[test]
    fn test_linear_subpattern() {
        let patterns = [
            PatternEvent {
                id: 0,
                event_type: PatternEventType::Default,
                signature: "edge1".to_string(),
                subject: PatternEntity {
                    id: 0,
                    signature: "".to_string(),
                },
                object: PatternEntity {
                    id: 1,
                    signature: "".to_string(),
                },
            },
            PatternEvent {
                id: 1,
                event_type: PatternEventType::Default,
                signature: "edge2".to_string(),
                subject: PatternEntity {
                    id: 1,
                    signature: "".to_string(),
                },
                object: PatternEntity {
                    id: 2,
                    signature: "".to_string(),
                },
            },
        ];

        let sub_pattern = SubPattern {
            id: 0,
            events: patterns.iter().collect(),
        };
        let decomposition = [sub_pattern];

        let match_events = vec![
            (match_event(1, "edge1", 1, 2, &patterns[0]), 0),
            (match_event(2, "edge2", 2, 3, &patterns[1]), 1),
        ];

        let mut layer = CompositionLayer::new(match_events.into_iter(), &decomposition, u64::MAX);
        let result = layer.next().unwrap();
        assert_eq!(result.len(), 1);
    }

    /// In this testcase, the pattern event `p1` and `p3` matches the same input edge `i1`,
    /// but `(i1, i2, i1)` is not a valid match state, since the input edge `i1` is duplicated.
    #[test]
    fn test_event_uniqueness() {
        let patterns = [
            PatternEvent {
                id: 0,
                event_type: PatternEventType::Default,
                signature: "a".to_string(),
                subject: PatternEntity {
                    id: 0,
                    signature: "".to_string(),
                },
                object: PatternEntity {
                    id: 1,
                    signature: "".to_string(),
                },
            },
            PatternEvent {
                id: 1,
                event_type: PatternEventType::Default,
                signature: "b".to_string(),
                subject: PatternEntity {
                    id: 1,
                    signature: "".to_string(),
                },
                object: PatternEntity {
                    id: 2,
                    signature: "".to_string(),
                },
            },
            PatternEvent {
                id: 2,
                event_type: PatternEventType::Default,
                signature: "a".to_string(),
                subject: PatternEntity {
                    id: 3,
                    signature: "".to_string(),
                },
                object: PatternEntity {
                    id: 1,
                    signature: "".to_string(),
                },
            },
        ];

        let sub_pattern = SubPattern {
            id: 0,
            events: patterns.iter().collect(),
        };
        let decomposition = [sub_pattern];

        let match_events = vec![
            (match_event(1, "a", 1, 2, &patterns[0]), 0),
            (match_event(2, "b", 2, 3, &patterns[1]), 1),
            (match_event(1, "a", 1, 2, &patterns[2]), 2),
        ];

        let mut layer = CompositionLayer::new(match_events.into_iter(), &decomposition, u64::MAX);
        assert!(layer.next().is_none());
    }

    #[test]
    fn test_entity_uniqueness() {
        let patterns = [
            PatternEvent {
                id: 0,
                event_type: PatternEventType::Default,
                signature: "edge1".to_string(),
                subject: PatternEntity {
                    id: 0,
                    signature: "".to_string(),
                },
                object: PatternEntity {
                    id: 1,
                    signature: "".to_string(),
                },
            },
            PatternEvent {
                id: 1,
                event_type: PatternEventType::Default,
                signature: "edge2".to_string(),
                subject: PatternEntity {
                    id: 1,
                    signature: "".to_string(),
                },
                object: PatternEntity {
                    id: 2,
                    signature: "".to_string(),
                },
            },
        ];

        let sub_pattern = SubPattern {
            id: 0,
            events: patterns.iter().collect(),
        };
        let decomposition = [sub_pattern];

        let match_events = vec![
            (match_event(1, "edge1", 1, 2, &patterns[0]), 0),
            (match_event(2, "edge2", 2, 1, &patterns[1]), 1), // entity ID 1 duplicated
        ];

        let mut layer = CompositionLayer::new(match_events.into_iter(), &decomposition, u64::MAX);
        assert!(layer.next().is_none());
    }
}
