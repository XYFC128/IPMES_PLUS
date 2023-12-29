use crate::match_event::MatchEvent;
use crate::process_layers::join_layer::SubPatternBuffer;
use std::cmp::Ordering;
use std::cmp::{max, min};
use log::debug;

#[derive(Clone, Debug)]
pub struct SubPatternMatch<'p> {
    pub id: usize,
    /// The timestamp of the last edge (in "match_events"), which is also the latest timestamp; indicating "current time".
    pub latest_time: u64,
    /// The timestamp of the earliest edge; for determining expiry.
    pub earliest_time: u64,

    /// (input node id, pattern node id)
    /// match_entities.len() == number of nodes in this sun-pattern match
    pub match_entities: Vec<(u64, u64)>,

    /// "event_id_map[matched_id] = input_id"
    /// The term "matched edge" and "pattern edge" is used interchangeably.
    /// event_id_map.len() == number of edges in the "whole pattern"
    pub event_id_map: Vec<Option<u64>>,

    /// sort this by 'input edge id' for uniqueness determination
    pub match_events: Vec<MatchEvent<'p>>,
}

/// Since pattern-edges in sub-patterns are disjoint, we need not check uniqueness.
fn merge_event_id_map(
    event_id_map1: &Vec<Option<u64>>,
    event_id_map2: &Vec<Option<u64>>,
) -> Vec<Option<u64>> {
    let mut event_id_map = vec![None; event_id_map1.len()];
    for i in 0..event_id_map1.len() {
        match event_id_map1[i] {
            Some(t) => event_id_map[i] = Some(t),
            None => match event_id_map2[i] {
                Some(t) => event_id_map[i] = Some(t),
                None => (),
            },
        }
    }
    event_id_map
}

fn check_edge_uniqueness(match_events: &Vec<MatchEvent>) -> bool {
    let mut prev_id = u64::MAX;
    for edge in match_events {
        if edge.input_event.id == prev_id {
            return false;
        }
        prev_id = edge.input_event.id;
    }
    true
}

impl<'p> SubPatternMatch<'p> {
    /// todo: check correctness
    pub fn merge_matches(
        sub_pattern_buffer: &SubPatternBuffer<'p>,
        sub_pattern_match1: &Self,
        sub_pattern_match2: &Self,
    ) -> Option<Self> {
        debug!("try merge_match_events...");

        // merge "match_events" (WITHOUT checking "edge uniqueness")
        let (match_events, timestamps) = sub_pattern_buffer.try_merge_match_events(
            &sub_pattern_match1.match_events,
            &sub_pattern_match2.match_events,
        )?;

        debug!("edge uniqueness checking...");

        // handle "edge uniqueness"
        if !check_edge_uniqueness(&match_events) {
            return None;
        }

        debug!("order relation checking...");

        // check "order relation"
        if !sub_pattern_buffer.relation.check_order_relation(
            &timestamps,
        ) {
            return None;
        }

        debug!("shared node and node uniqueness checking");

        // handle "shared node" and "node uniqueness"
        let match_entities = sub_pattern_buffer.try_merge_entities(
            &sub_pattern_match1.match_entities,
            &sub_pattern_match2.match_entities,
        )?;

        // merge "event_id_map"
        let event_id_map = merge_event_id_map(
            &sub_pattern_match1.event_id_map,
            &sub_pattern_match2.event_id_map,
        );

        Some(SubPatternMatch {
            // 'id' is meaningless here
            id: 0,
            latest_time: max(
                sub_pattern_match1.latest_time,
                sub_pattern_match2.latest_time,
            ),
            earliest_time: min(
                sub_pattern_match1.earliest_time,
                sub_pattern_match2.earliest_time,
            ),
            match_entities,
            event_id_map,
            match_events,
        })
    }
}

#[derive(Clone, Debug)]
pub struct EarliestFirst<'p>(pub SubPatternMatch<'p>);

impl Eq for EarliestFirst<'_> {}

impl PartialEq<Self> for EarliestFirst<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.earliest_time.eq(&other.0.earliest_time)
    }
}

impl Ord for EarliestFirst<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.earliest_time.cmp(&other.0.earliest_time).reverse()
    }
}

impl PartialOrd<Self> for EarliestFirst<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use crate::input_event::InputEvent;
    use crate::pattern::Event;
    use super::*;

    #[test]
    /// Fail
    fn test_check_edge_uniqueness_1() {
        let pattern_edge = Event {
            id: 0,
            signature: "".to_string(),
            subject: 0,
            object: 0,
        };
        let match_edges = vec![
            MatchEvent {
                input_event: Rc::new(InputEvent {
                    timestamp: 0,
                    signature: "".to_string(),
                    id: 1,
                    subject: 0,
                    object: 0,
                }),
                matched: &pattern_edge,
            },
            MatchEvent {
                input_event: Rc::new(InputEvent {
                    timestamp: 0,
                    signature: "".to_string(),
                    id: 2,
                    subject: 0,
                    object: 0,
                }),
                matched: &pattern_edge
            },
            MatchEvent {
                input_event: Rc::new(InputEvent {
                    timestamp: 0,
                    signature: "".to_string(),
                    id: 2,
                    subject: 0,
                    object: 0,
                }),
                matched: &pattern_edge
            }
        ];

        assert_eq!(check_edge_uniqueness(&match_edges), false);
    }

    #[test]
    /// Pass
    fn test_check_edge_uniqueness_2() {
        let pattern_edge = Event {
            id: 0,
            signature: "".to_string(),
            subject: 0,
            object: 0,
        };
        let match_edges = vec![
            MatchEvent {
                input_event: Rc::new(InputEvent {
                    timestamp: 0,
                    signature: "".to_string(),
                    id: 1,
                    subject: 0,
                    object: 0,
                }),
                matched: &pattern_edge,
            },
            MatchEvent {
                input_event: Rc::new(InputEvent {
                    timestamp: 0,
                    signature: "".to_string(),
                    id: 2,
                    subject: 0,
                    object: 0,
                }),
                matched: &pattern_edge
            },
            MatchEvent {
                input_event: Rc::new(InputEvent {
                    timestamp: 0,
                    signature: "".to_string(),
                    id: 3,
                    subject: 0,
                    object: 0,
                }),
                matched: &pattern_edge
            }
        ];

        assert_eq!(check_edge_uniqueness(&match_edges), true);
    }

    #[test]
    fn test_merge_edge_id_map_1() {
        let edge_id_map1 = vec![
            None,
            Some(3),
            Some(2),
            None,
            None
        ];

        let edge_id_map2 = vec![
            Some(1),
            None,
            None,
            None,
            Some(7)
        ];

        let ans = vec![
            Some(1),
            Some(3),
            Some(2),
            None,
            Some(7)
        ];

        assert_eq!(ans, merge_event_id_map(&edge_id_map1, &edge_id_map2));
    }

    #[test]
    fn test_merge_edge_id_map_2() {
        let edge_id_map1 = vec![
            None,
            Some(3),
            None,
            None,
            None
        ];

        let edge_id_map2 = vec![
            Some(1),
            None,
            None,
            None,
            Some(7)
        ];

        let ans = vec![
            Some(1),
            Some(3),
            None,
            None,
            Some(7)
        ];

        assert_eq!(ans, merge_event_id_map(&edge_id_map1, &edge_id_map2));
    }
}