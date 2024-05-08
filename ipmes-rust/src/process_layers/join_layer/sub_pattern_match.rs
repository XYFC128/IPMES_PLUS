use crate::match_event::MatchEvent;
use crate::process_layers::composition_layer;
use crate::process_layers::join_layer::SubPatternBuffer;
use crate::universal_match_event::UniversalMatchEvent;
use log::debug;
use std::cmp::Ordering;
use std::cmp::{max, min};
use std::fmt::{Debug, Pointer};

/// Matches of sub-patterns.
#[derive(Clone)]
pub struct SubPatternMatch<'p> {
    /// The id of the matched sub-pattern.
    pub id: usize,
    /// The timestamp of the last event (in `match_events`), which is also the latest timestamp; indicating "current time".
    pub latest_time: u64,
    /// The timestamp of the earliest event; for determining expiry of this match.
    pub earliest_time: u64,

    /// Sorted array of `(input entity id, pattern entity id)`.
    ///
    /// `match_entities.len()` == number of entities in this sub-pattern match.
    pub match_entities: Box<[(u64, u64)]>,

    /// Sorted input event ids for event uniqueness determination.
    pub event_ids: Box<[u64]>,

    /// `event_id_map[matched_id] = input_event`
    ///
    /// `event_id_map.len()` == number of event in the "whole pattern".
    ///
    /// > Note: The terms **matched event** and pattern event are used interchangeably.
    pub match_event_map: Box<[Option<UniversalMatchEvent<'p>>]>,
}

pub struct DebugMatchEventMap<'p, 't>(pub &'t [Option<UniversalMatchEvent<'p>>]);
impl<'p, 't> Debug for DebugMatchEventMap<'p, 't> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(
                self.0
                    .iter()
                    .map(|opt| opt.as_ref().map(|val| &val.event_ids)),
            )
            .finish()
    }
}

impl<'p> Debug for SubPatternMatch<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubPatternMatch")
            .field("id", &self.id)
            .field("match_entities", &self.match_entities)
            .field("event_ids", &self.event_ids)
            .field(
                "match_event_map",
                &DebugMatchEventMap(&self.match_event_map),
            )
            .finish()
    }
}

/// > Note: Since pattern-edges in sub-patterns are disjoint, we need not check uniqueness.
fn merge_match_event_map<T>(event_map1: &[Option<T>], event_map2: &[Option<T>]) -> Box<[Option<T>]>
where
    T: Clone,
{
    assert_eq!(event_map1.len(), event_map2.len());
    let mut merged = Vec::with_capacity(event_map1.len());
    for (a, b) in event_map1.iter().zip(event_map2.iter()) {
        if a.is_some() {
            merged.push(a.clone());
        } else if b.is_some() {
            merged.push(b.clone());
        } else {
            merged.push(None);
        }
    }
    merged.into_boxed_slice()
}

fn try_merge_event_ids(id_list1: &[u64], id_list2: &[u64]) -> Option<Box<[u64]>> {
    let mut merged = Vec::with_capacity(id_list1.len() + id_list2.len());
    let mut p1 = id_list1.iter();
    let mut p2 = id_list2.iter();
    let mut next1 = p1.next();
    let mut next2 = p2.next();

    while let (Some(id1), Some(id2)) = (next1, next2) {
        match id1.cmp(id2) {
            Ordering::Less => {
                merged.push(*id1);
                next1 = p1.next();
            }
            Ordering::Equal => {
                debug!("event id duplicates: {}", id1);
                return None;
            }
            Ordering::Greater => {
                merged.push(*id2);
                next2 = p2.next();
            }
        }
    }

    if next1.is_none() {
        p1 = p2;
        next1 = next2;
    }

    while let Some(id) = next1 {
        merged.push(*id);
        next1 = p1.next();
    }

    Some(merged.into_boxed_slice())
}

fn check_edge_uniqueness(match_events: &[MatchEvent]) -> bool {
    let mut prev_id = u64::MAX;
    for edge in match_events {
        if edge.input_event.event_id == prev_id {
            return false;
        }
        prev_id = edge.input_event.event_id;
    }
    true
}

impl<'p> SubPatternMatch<'p> {
    pub fn build(
        sub_pattern_id: u32,
        match_instance: composition_layer::MatchInstance<'p>,
        num_pattern_event: usize,
    ) -> Option<Self> {
        let latest_time = match_instance.match_events.last()?.end_time;
        let earliest_time = match_instance.start_time;

        let match_events = match_instance.match_events.into_vec();
        let match_entities = match_instance.match_entities.clone();

        let mut event_ids: Vec<u64> = match_events
            .iter()
            .flat_map(|e| e.event_ids.iter().cloned())
            .collect();
        event_ids.sort_unstable();

        let mut match_event_map = vec![None; num_pattern_event];
        for event in match_events.into_iter() {
            let pat_id = event.matched.id;
            match_event_map[pat_id] = Some(event);
        }

        Some(Self {
            id: sub_pattern_id as usize,
            latest_time,
            earliest_time,
            event_ids: event_ids.into_boxed_slice(),
            match_entities,
            match_event_map: match_event_map.into_boxed_slice(),
        })
    }

    // todo: check correctness
    pub fn merge_matches(
        sub_pattern_buffer: &SubPatternBuffer<'p>,
        sub_pattern_match1: &Self,
        sub_pattern_match2: &Self,
    ) -> Option<Self> {
        debug!(
            "now try merging\n{:?} and\n{:?}",
            sub_pattern_match1, sub_pattern_match2,
        );

        debug!("event uniqueness checking...");

        let event_ids =
            try_merge_event_ids(&sub_pattern_match1.event_ids, &sub_pattern_match2.event_ids)?;
        let match_event_map = merge_match_event_map(
            &sub_pattern_match1.match_event_map,
            &sub_pattern_match2.match_event_map,
        );

        debug!("order relation checking...");

        // check "order relation"
        if !sub_pattern_buffer
            .relation
            .check_order_relation(&match_event_map)
        {
            return None;
        }

        debug!("shared node and node uniqueness checking");

        // handle "shared node" and "node uniqueness"
        let match_entities = sub_pattern_buffer.try_merge_entities(
            &sub_pattern_match1.match_entities,
            &sub_pattern_match2.match_entities,
        )?;

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
            event_ids,
            match_event_map,
        })
    }
}

/// Helper structure that implements `PartialEq`, `Ord`, `PartialOrd` traits for `SubPatternMatch`.
///
/// *Earliest* refers to `SubPatternMatch.earliest_time`.
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
    use super::*;
    use crate::input_event::InputEvent;
    use crate::pattern::{PatternEntity, PatternEvent, PatternEventType};
    use std::rc::Rc;

    fn dummy_input_event(event_id: u64) -> Rc<InputEvent> {
        Rc::new(InputEvent::new(0, event_id, "", 0, "", 0, ""))
    }

    #[test]
    fn test_check_edge_uniqueness_1() {
        let pattern_edge = PatternEvent {
            id: 0,
            event_type: PatternEventType::Default,
            signature: "".to_string(),
            subject: PatternEntity {
                id: 0,
                signature: "".to_string(),
            },
            object: PatternEntity {
                id: 0,
                signature: "".to_string(),
            },
        };
        let match_edges = vec![
            MatchEvent {
                input_event: dummy_input_event(1),
                matched: &pattern_edge,
            },
            MatchEvent {
                input_event: dummy_input_event(2),
                matched: &pattern_edge,
            },
            MatchEvent {
                input_event: dummy_input_event(2),
                matched: &pattern_edge,
            },
        ];

        assert!(!check_edge_uniqueness(&match_edges));
    }

    #[test]
    fn test_check_edge_uniqueness_2() {
        let pattern_edge = PatternEvent {
            id: 0,
            event_type: PatternEventType::Default,
            signature: "".to_string(),
            subject: PatternEntity {
                id: 0,
                signature: "".to_string(),
            },
            object: PatternEntity {
                id: 0,
                signature: "".to_string(),
            },
        };
        let match_edges = vec![
            MatchEvent {
                input_event: dummy_input_event(1),
                matched: &pattern_edge,
            },
            MatchEvent {
                input_event: dummy_input_event(2),
                matched: &pattern_edge,
            },
            MatchEvent {
                input_event: dummy_input_event(3),
                matched: &pattern_edge,
            },
        ];

        assert!(check_edge_uniqueness(&match_edges));
    }

    #[test]
    fn test_merge_edge_id_map_1() {
        let edge_id_map1 = vec![None, Some(3), Some(2), None, None];
        let edge_id_map2 = vec![Some(1), None, None, None, Some(7)];

        assert_eq!(
            [Some(1), Some(3), Some(2), None, Some(7)],
            *merge_match_event_map(&edge_id_map1, &edge_id_map2)
        );
    }

    #[test]
    fn test_merge_edge_id_map_2() {
        let edge_id_map1 = vec![None, Some(3), None, None, None];
        let edge_id_map2 = vec![Some(1), None, None, None, Some(7)];

        assert_eq!(
            [Some(1), Some(3), None, None, Some(7)],
            *merge_match_event_map(&edge_id_map1, &edge_id_map2)
        );
    }

    #[test]
    fn test_merge_event_id_basecase() {
        let id_list1 = [1, 3, 5];
        let id_list2 = [2, 4];
        assert_eq!(
            *try_merge_event_ids(&id_list1, &id_list2).unwrap(),
            [1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn test_merge_event_id_dup_id() {
        let id_list1 = [1, 3, 5];
        let id_list2 = [3, 4];
        assert_eq!(try_merge_event_ids(&id_list1, &id_list2), None);
    }

    #[test]
    fn test_merge_event_id_edgecases() {
        assert_eq!(*try_merge_event_ids(&[1], &[2]).unwrap(), [1, 2]);
        assert_eq!(*try_merge_event_ids(&[2], &[1]).unwrap(), [1, 2]);
        assert_eq!(*try_merge_event_ids(&[1], &[]).unwrap(), [1]);
        assert!(try_merge_event_ids(&[], &[]).unwrap().is_empty(),);
    }
}
