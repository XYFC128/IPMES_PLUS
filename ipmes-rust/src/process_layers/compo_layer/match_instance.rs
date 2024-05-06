use std::cmp::min;
use std::fmt::Debug;

use itertools::Itertools;
use log::debug;

use super::filter::FilterInfo;
use super::StateData;
use crate::process_layers::matching_layer::PartialMatchEvent;
use crate::universal_match_event::UniversalMatchEvent;

type InputEntityId = u64;
type PatternEntityId = u64;

#[derive(Clone)]
#[cfg_attr(test, derive(Default))]
pub struct MatchInstance<'p> {
    pub start_time: u64,
    pub match_events: Box<[UniversalMatchEvent<'p>]>,

    /// Sorted array of `(input entity id, pattern entity id)`.
    ///
    /// `match_entities.len()` == number of entities in this sub-pattern match.
    pub match_entities: Box<[(InputEntityId, PatternEntityId)]>,
    pub event_ids: Box<[InputEntityId]>,
    pub state_id: u32,
    pub state_data: StateData,
}

impl<'p> MatchInstance<'p> {
    pub fn dead_default() -> Self {
        Self {
            start_time: 0,
            match_events: Box::new([]),
            match_entities: Box::new([]),
            event_ids: Box::new([]),
            state_id: 0,
            state_data: StateData::Dead,
        }
    }

    pub fn accept(&mut self, match_event: &PartialMatchEvent<'p>) -> InstanceAction<'p> {
        if self.contains_event(match_event.input_event.event_id) {
            return InstanceAction::Remain;
        }

        match &mut self.state_data {
            StateData::Default { next_state } => InstanceAction::NewInstance {
                new_state_id: *next_state,
                new_event: match_event.into(),
            },
            StateData::AggFreq {
                next_state,
                frequency,
                current_set,
            } => {
                current_set.insert(match_event.input_event.event_id);
                if current_set.len() >= *frequency as usize {
                    let new_state_id = *next_state;

                    // modify the last match event
                    if let Some(last_event) = self.match_events.last_mut() {
                        last_event.end_time = match_event.input_event.timestamp;

                        let mut ids: Vec<u64> = current_set.drain().collect();
                        ids.sort_unstable();
                        last_event.event_ids = ids.into_boxed_slice();
                    }

                    InstanceAction::MoveInstance { new_state_id }
                } else {
                    InstanceAction::Remain
                }
            }
            _ => InstanceAction::Remain,
        }
    }

    pub fn clone_extend(
        &self,
        new_event: UniversalMatchEvent<'p>,
        filter_info: &FilterInfo,
    ) -> Option<Self> {
        let event_ids = Self::dup_extend_event_ids(&self.event_ids, &new_event.event_ids)?;
        let match_entities =
            Self::dup_extend_entities_by_event(&self.match_entities, &new_event, filter_info)?;
        let start_time = min(self.start_time, new_event.start_time);

        let mut new_match_events = Vec::with_capacity(self.match_events.len() + 1);
        new_match_events.extend_from_slice(&self.match_events);
        new_match_events.push(new_event);

        Some(Self {
            start_time,
            match_events: new_match_events.into_boxed_slice(),
            match_entities,
            event_ids,
            ..self.clone()
        })
    }

    /// Return true if the match_event is already in this [MatchInstance]
    pub fn contains_event(&self, input_event_id: u64) -> bool {
        self.event_ids.binary_search(&input_event_id).is_ok()
    }

    /// Return true if add this (`entity_id`, `pattern_id`) pair to this match instance will result in entity collision.
    /// Entity collision occurs when the same input entity matches different pattern entities
    pub fn conflict_with_entity(&self, entity_id: u64, pattern_id: u64) -> bool {
        if let Ok(index) = self
            .match_entities
            .binary_search_by(|entry| entry.0.cmp(&entity_id))
        {
            if self.match_entities[index].1 != pattern_id {
                return true;
            }
        }

        false
    }

    fn dup_extend_event_ids(event_ids: &[u64], new_ids: &[u64]) -> Option<Box<[u64]>> {
        let mut new_event_ids = Vec::with_capacity(event_ids.len() + new_ids.len());
        new_event_ids.extend_from_slice(event_ids);
        new_event_ids.extend_from_slice(new_ids);
        new_event_ids.sort_unstable();
        for (a, b) in new_event_ids.iter().tuple_windows() {
            if *a == *b {
                return None;
            }
        }

        Some(new_event_ids.into_boxed_slice())
    }

    fn dup_extend_entities_by_event(
        match_entities: &[(InputEntityId, PatternEntityId)],
        event: &UniversalMatchEvent,
        filter_info: &FilterInfo,
    ) -> Option<Box<[(InputEntityId, PatternEntityId)]>> {
        match filter_info {
            FilterInfo::Subject {
                match_ord: _,
                subject: _,
            } => Self::dup_extend_entities_list(
                match_entities,
                event.object_id,
                event.matched.object.id as u64,
            ),
            FilterInfo::Object {
                match_ord: _,
                object: _,
            } => Self::dup_extend_entities_list(
                match_entities,
                event.subject_id,
                event.matched.subject.id as u64,
            ),
            FilterInfo::MatchOrdOnly { match_ord: _ } => {
                if event.subject_id < event.object_id {
                    Some(Box::new([
                        (event.subject_id, event.matched.subject.id as u64),
                        (event.object_id, event.matched.object.id as u64),
                    ]))
                } else {
                    Some(Box::new([
                        (event.object_id, event.matched.object.id as u64),
                        (event.subject_id, event.matched.subject.id as u64),
                    ]))
                }
            }
            _ => Some(Box::from(match_entities)),
        }
    }

    /// Clone and add (`entity_id`, `pattern_id`) to the `match_entities`.
    ///
    /// Returns [None] when the `entity_id` is already in it.
    fn dup_extend_entities_list(
        match_entities: &[(InputEntityId, PatternEntityId)],
        entity_id: u64,
        pattern_id: u64,
    ) -> Option<Box<[(InputEntityId, PatternEntityId)]>> {
        if match_entities.is_empty() {
            return None;
        }

        let mut new_entities = Vec::with_capacity(match_entities.len() + 1);
        let mut iter = match_entities.iter().peekable();
        for entry in iter.take_while_ref(|(ent_id, _)| *ent_id < entity_id) {
            new_entities.push(*entry);
        }

        if let Some(entry) = iter.peek() {
            if entry.0 == entity_id {
                return None; // entity id duplicates
            }
        }

        new_entities.push((entity_id, pattern_id));
        new_entities.extend(iter);

        Some(new_entities.into_boxed_slice())
    }
}

impl<'p> Debug for MatchInstance<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatchInstance")
            .field("start_time", &self.start_time)
            .field("match_events", &self.match_events)
            .field("match_entities", &self.match_entities)
            .field("state_id", &self.state_id)
            .field("state_data", &self.state_data)
            .finish()
    }
}

#[derive(Clone)]
pub enum InstanceAction<'p> {
    Remain,
    MoveInstance {
        new_state_id: u32,
    },
    NewInstance {
        new_state_id: u32,
        new_event: UniversalMatchEvent<'p>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_event() {
        let instance = MatchInstance {
            event_ids: Box::new([1, 3, 7, 13, 50, 100]),
            ..Default::default()
        };

        assert!(instance.contains_event(1));
        assert!(instance.contains_event(100));
        assert!(!instance.contains_event(0));
        assert!(!instance.contains_event(10));
    }

    #[test]
    fn test_conflict_with_entity() {
        let instance = MatchInstance {
            match_entities: Box::new([(100, 0), (101, 1), (103, 2)]),
            ..Default::default()
        };

        assert!(!instance.conflict_with_entity(100, 0)); // no conflict
        assert!(instance.conflict_with_entity(100, 2)); // 100 -> {0, 2}
    }

    #[test]
    fn test_dup_extend_entities_list() {
        let match_entities: Box<[(u64, u64)]> = Box::new([(100, 1), (101, 0), (103, 2)]);
        
        // insert last
        assert_eq!(
            *MatchInstance::dup_extend_entities_list(&match_entities, 104, 3).unwrap(),
            [(100, 1), (101, 0), (103, 2), (104, 3)],
        );
        // insert middle
        assert_eq!(
            *MatchInstance::dup_extend_entities_list(&match_entities, 102, 3).unwrap(),
            [(100, 1), (101, 0), (102, 3), (103, 2)],
        );
        // event_id duplicates
        assert_eq!(
            MatchInstance::dup_extend_entities_list(&match_entities, 100, 1),
            None
        );
        // event_id duplicates
        assert_eq!(
            MatchInstance::dup_extend_entities_list(&match_entities, 100, 3),
            None
        );
    }
}
