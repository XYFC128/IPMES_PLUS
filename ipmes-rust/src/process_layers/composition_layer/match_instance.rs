use super::pattern_info::SharedNodeInfo;
use crate::input_event::InputEvent;
use crate::match_event::MatchEvent;
use crate::universal_match_event::UniversalMatchEvent;
use itertools::Itertools;
use regex::Match;
use std::cmp::min;
use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;

pub type InputEntityId = u64;
pub type PatternEntityId = u64;
pub type InputEventId = u64;

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
    event: &MatchEvent,
    // event: &UniversalMatchEvent,
    shared_node_info: SharedNodeInfo,
) -> Option<Box<[(InputEntityId, PatternEntityId)]>> {
    use SharedNodeInfo::*;
    match shared_node_info {
        None => {
            if event.subject_id < event.object_id {
                Some(Box::new([
                    (event.subject_id, event.subject_id),
                    (event.object_id, event.object_id),
                ]))
            } else {
                Some(Box::new([
                    (event.object_id, event.object_id),
                    (event.subject_id, event.subject_id),
                ]))
            }
        }
        Subject => dup_extend_entities_list(
            match_entities,
            event.object_id,
            event.object_id,
        ),
        Object => dup_extend_entities_list(
            match_entities,
            event.subject_id,
            event.subject_id,
        ),
        Both => Some(Box::from(match_entities)),
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

#[derive(Clone)]
#[cfg_attr(test, derive(Default))]
// pub struct MatchInstance<'p> {
pub struct MatchInstance {
    pub start_time: u64,
    pub match_events: Box<[MatchEvent]>,
    // pub match_events: Box<[UniversalMatchEvent<'p>]>,

    /// Sorted array of `(input entity id, pattern entity id)`.
    ///
    /// `match_entities.len()` == number of entities in this sub-pattern match.
    pub match_entities: Box<[(InputEntityId, PatternEntityId)]>,
    pub event_ids: Box<[InputEventId]>,
    pub state_id: u32,
}

// impl<'p> MatchInstance<'p> {
impl MatchInstance {
    pub fn dead_default() -> Self {
        Self {
            start_time: 0,
            match_events: Box::new([]),
            match_entities: Box::new([]),
            event_ids: Box::new([]),
            state_id: 0,
        }
    }

    /// clone this instance and insert the [new_event] into the new instance. [filter_info] is the
    /// filter of the original instance. This method uses this information to extract the newly
    /// added entities.
    pub fn clone_extend(
        &self,
        new_event: MatchEvent,
        // new_event: UniversalMatchEvent<'p>,
        shared_node_info: SharedNodeInfo,
    ) -> Option<Self> {
        // @TODO: Perhaps we need not extend ids explicitly?
        let event_ids = dup_extend_event_ids(&self.event_ids, &new_event.raw_events.get_ids().collect_vec())?;
        // let event_ids = dup_extend_event_ids(&self.event_ids, &new_event.event_ids)?;
        let match_entities =
            dup_extend_entities_by_event(&self.match_entities, &new_event, shared_node_info)?;
        let start_time = min(self.start_time, new_event.raw_events.get_interval().0);

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

    /// Adding the flow event to the new instance. This method skip the check of the event
    /// uniqueness against the events in the new flow event. In addition, the event ids in the new
    /// flow event will not be considered for uniqueness checking for later events.
    pub fn clone_extend_flow(
        &self,
        // new_event: UniversalMatchEvent<'p>,
        new_event: MatchEvent,
        shared_node_info: SharedNodeInfo,
    ) -> Option<Self> {
        let event_ids = self.event_ids.clone();
        let match_entities =
            dup_extend_entities_by_event(&self.match_entities, &new_event, shared_node_info)?;
        let start_time = min(self.start_time, new_event.raw_events.get_interval().0);

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

    /// Return true if the entity_id is already in this [MatchInstance]
    pub fn contains_eneity(&self, entity_id: u64) -> bool {
        self.match_entities
            .binary_search_by(|entry| entry.0.cmp(&entity_id))
            .is_ok()
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
}

// impl<'p> Debug for MatchInstance<'p> {
impl<'p> Debug for MatchInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatchInstance")
            .field("start_time", &self.start_time)
            // .field("match_events", &self.match_events)
            .field("match_entities", &self.match_entities)
            .field("state_id", &self.state_id)
            .finish()
    }
}

// pub struct FreqInstance<'p> {
pub struct FreqInstance {
    // pub instance: MatchInstance<'p>,
    pub instance: MatchInstance,
    pub start_time: u64,
    pub remain_freq: u32,
    pub cur_set: HashSet<u64>,
    // pub new_events: Vec<u64>,
    pub new_events: Vec<Rc<InputEvent>>,
}

// impl<'p> FreqInstance<'p> {
impl FreqInstance {
    // pub fn new(instance: MatchInstance<'p>, frequency: u32, time: u64) -> Self {
    pub fn new(instance: MatchInstance, frequency: u32, time: u64) -> Self {
        let cur_set = HashSet::from_iter(instance.event_ids.iter().copied());
        Self {
            instance,
            start_time: time,
            remain_freq: frequency,
            cur_set,
            new_events: vec![],
        }
    }

    /// Adds an event id into the frequency tracing set
    ///
    /// Returns `ture` if the event_id was not previously in the set
    // pub fn add_event(&mut self, event_id: u64) -> bool {
    // pub fn add_event(&mut self, event_id: u64) -> bool {
    pub fn add_event(&mut self, event: &Rc<InputEvent>) -> bool {
        if self.cur_set.insert(event.event_id) {
            self.remain_freq -= 1;
            // self.new_events.push(event_id);
            self.new_events.push(event.clone());
            true
        } else {
            false
        }
    }

    pub fn is_full(&self) -> bool {
        self.remain_freq == 0
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
            *dup_extend_entities_list(&match_entities, 104, 3).unwrap(),
            [(100, 1), (101, 0), (103, 2), (104, 3)],
        );
        // insert middle
        assert_eq!(
            *dup_extend_entities_list(&match_entities, 102, 3).unwrap(),
            [(100, 1), (101, 0), (102, 3), (103, 2)],
        );
        // event_id duplicates
        assert_eq!(dup_extend_entities_list(&match_entities, 100, 1), None);
        // event_id duplicates
        assert_eq!(dup_extend_entities_list(&match_entities, 100, 3), None);
    }
}
