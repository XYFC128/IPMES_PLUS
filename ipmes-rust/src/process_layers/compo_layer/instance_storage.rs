use crate::process_layers::matching_layer::PartialMatchEvent;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use super::{filter::FilterInfo, MatchInstance, StateInfo};

pub struct InstanceStorage<'p> {
    pub simple_instances: HashMap<usize, Vec<MatchInstance<'p>>>,
    pub subject_instances: HashMap<(usize, u64), Vec<MatchInstance<'p>>>,
    pub object_instances: HashMap<(usize, u64), Vec<MatchInstance<'p>>>,
    pub endpoints_instances: HashMap<(usize, u64, u64), Vec<MatchInstance<'p>>>,
}

impl<'p> InstanceStorage<'p> {
    pub fn init_from_state_table(state_table: &[(StateInfo, FilterInfo)]) -> Self {
        let mut simple_instances: HashMap<usize, Vec<MatchInstance<'p>>> = HashMap::new();
        for (state_info, filter_info) in state_table {
            if let FilterInfo::MatchOrdOnly { match_ord } = filter_info {
                simple_instances
                    .entry(*match_ord)
                    .or_default()
                    .push(MatchInstance {
                        start_time: u64::MAX,
                        match_events: Vec::new(),
                        state_data: (*state_info).try_into().unwrap(),
                        event_ids: HashSet::new(),
                    });
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

        Self::apply_filter(
            &mut self.simple_instances,
            match_ord,
            window_bound,
            &mut callback,
        );

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
        let is_in_window = |inst: &MatchInstance| inst.start_time >= window_bound;

        if let Some(instances) = storage.get_mut(&filter) {
            instances.retain(is_in_window);
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
