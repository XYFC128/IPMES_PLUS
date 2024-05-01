use itertools::Itertools;

use super::StateData;
use crate::process_layers::matching_layer::PartialMatchEvent;
use crate::universal_match_event::UniversalMatchEvent;

type InputEntityId = u64;
type PatternEntityId = u64;

#[derive(Clone, Debug)]
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
    pub fn accept(&mut self, match_event: &PartialMatchEvent<'p>) -> InstanceAction<'p> {
        if self.contains_event(match_event.input_event.event_id) {
            return InstanceAction::Remain;
        }

        match &mut self.state_data {
            StateData::Default { next_state } => InstanceAction::NewInstance {
                new_state_id: *next_state,
                new_event: match_event.into(),
            },
            StateData::InitFreq { next_state } => InstanceAction::NewInstance {
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
                    InstanceAction::NewInstance {
                        new_state_id: *next_state,
                        new_event: match_event.into(),
                    }
                } else {
                    InstanceAction::Remain
                }
            }
        }
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

    /// Clone and add (`entity_id`, `pattern_id`) to the `match_entities`.
    ///
    /// Returns [None] when the `entity_id` is already in it.
    pub fn dup_extend_entities_list(
        match_entities: &[(u64, u64)],
        new_entity: u64,
        match_id: u64,
    ) -> Option<Box<[(InputEntityId, PatternEntityId)]>> {
        if match_entities.is_empty() {
            return None;
        }

        let mut new_entities = Vec::with_capacity(match_entities.len() + 1);
        let mut iter = match_entities.iter();
        for entry in iter.take_while_ref(|(ent_id, _)| *ent_id < new_entity) {
            new_entities.push(*entry);
        }

        if let Some(entry) = iter.next() {
            if entry.0 == new_entity {
                return None; // entity id duplicates
            }
            new_entities.push((new_entity, match_id));
            new_entities.push(*entry);
        }

        new_entities.extend(iter);
        Some(new_entities.into_boxed_slice())
    }
}

#[derive(Clone)]
pub enum InstanceAction<'p> {
    Remain,
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
        assert_eq!(
            *MatchInstance::dup_extend_entities_list(&match_entities, 102, 3).unwrap(),
            [(100, 1), (101, 0), (102, 3), (103, 2)],
        );
        assert_eq!(
            MatchInstance::dup_extend_entities_list(&match_entities, 100, 1),
            None
        );
        assert_eq!(
            MatchInstance::dup_extend_entities_list(&match_entities, 100, 3),
            None
        );
    }
}
