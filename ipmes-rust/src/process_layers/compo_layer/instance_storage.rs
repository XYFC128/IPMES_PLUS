use crate::process_layers::matching_layer::PartialMatchEvent;
use ahash::{HashMap, HashMapExt};
use std::hash::Hash;

use super::{filter::FilterInfo, MatchInstance, StateData, StateInfo};

pub struct InstanceStorage<'p> {
    /// The match instances that can go to next state once an input event matches the specified
    /// match order, no shared entity is required. This only happens on the first event of a
    /// sub-pattern. Since sub-patterns are disjoint and each match order correspond to only one
    /// pattern event, there will be at most one such match instance for each match order.
    pub simple_instances: HashMap<usize, MatchInstance<'p>>,
    pub subject_instances: HashMap<(usize, u64), Vec<MatchInstance<'p>>>,
    pub object_instances: HashMap<(usize, u64), Vec<MatchInstance<'p>>>,
    pub endpoints_instances: HashMap<(usize, u64, u64), Vec<MatchInstance<'p>>>,
}

impl<'p> InstanceStorage<'p> {
    pub fn init_from_state_table(state_table: &[(StateInfo, FilterInfo)]) -> Self {
        let mut simple_instances: HashMap<usize, MatchInstance<'p>> = HashMap::new();
        for (state_id, (state_info, filter_info)) in state_table.iter().enumerate() {
            if let FilterInfo::MatchOrdOnly { match_ord } = filter_info {
                simple_instances.insert(
                    *match_ord,
                    MatchInstance {
                        start_time: u64::MAX,
                        match_events: Box::new([]),
                        match_entities: Box::new([]),
                        state_data: (*state_info).into(),
                        state_id: state_id as u32,
                        event_ids: Box::new([]),
                    },
                );
            }
        }

        Self {
            simple_instances,
            subject_instances: HashMap::new(),
            object_instances: HashMap::new(),
            endpoints_instances: HashMap::new(),
        }
    }

    pub fn query<F>(
        &mut self,
        match_event: &PartialMatchEvent<'p>,
        window_bound: u64,
        mut callback: F,
    ) where
        F: FnMut(&mut MatchInstance<'p>),
    {
        let match_ord = match_event.match_ord;
        let subject_id = match_event.subject_id;
        let object_id = match_event.input_event.object_id;

        if let Some(instance) = self.simple_instances.get_mut(&match_ord) {
            callback(instance);
        }

        Self::apply_filter(
            &mut self.subject_instances,
            (match_ord, subject_id),
            window_bound,
            &mut callback,
        );

        Self::apply_filter(
            &mut self.object_instances,
            (match_ord, object_id),
            window_bound,
            &mut callback,
        );

        Self::apply_filter(
            &mut self.endpoints_instances,
            (match_ord, subject_id, object_id),
            window_bound,
            &mut callback,
        );
    }

    fn apply_filter<K, F>(
        storage: &mut HashMap<K, Vec<MatchInstance<'p>>>,
        filter: K,
        window_bound: u64,
        mut callback: F,
    ) where
        K: Eq + Hash,
        F: FnMut(&mut MatchInstance<'p>),
    {
        let is_valid = |inst: &MatchInstance| {
            inst.start_time >= window_bound && !matches!(inst.state_data, StateData::Dead)
        };

        if let Some(instances) = storage.get_mut(&filter) {
            instances.retain(is_valid);
            if instances.is_empty() {
                storage.remove(&filter);
            } else {
                for instance in instances {
                    callback(instance);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::process_layers::compo_layer::entity_encode::EntityEncode;

    use super::*;

    #[test]
    fn test_init() {
        let state_table = [
            (
                StateInfo::Default { next_state: 1 },
                FilterInfo::MatchOrdOnly { match_ord: 0 },
            ),
            (
                StateInfo::Default { next_state: 2 },
                FilterInfo::Subject {
                    match_ord: 1,
                    subject: EntityEncode::object_of(0),
                },
            ),
            (StateInfo::Output { subpattern_id: 0 }, FilterInfo::None),
            (
                StateInfo::Default { next_state: 4 },
                FilterInfo::MatchOrdOnly { match_ord: 2 },
            ),
            (StateInfo::Output { subpattern_id: 0 }, FilterInfo::None),
        ];

        let storage = InstanceStorage::init_from_state_table(&state_table);

        assert_eq!(storage.simple_instances.len(), 2);
        let placeholder = &storage.simple_instances[&0];
        assert_eq!(placeholder.state_id, 0);

        let placeholder = &storage.simple_instances[&2];
        assert_eq!(placeholder.state_id, 3);

        assert_eq!(storage.subject_instances.len(), 0);
        assert_eq!(storage.object_instances.len(), 0);
        assert_eq!(storage.endpoints_instances.len(), 0);
    }
}
