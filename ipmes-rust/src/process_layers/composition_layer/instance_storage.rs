use crate::universal_match_event::UniversalMatchEvent;
use ahash::{HashMap, HashMapExt};
use std::borrow::Borrow;
use std::hash::Hash;
use std::slice::IterMut;

use super::filter::Filter;
use super::match_instance::FreqInstance;
use super::state_table::StateTable;
use super::{
    filter::FilterInfo, pattern_info::SharedNodeInfo, MatchInstance, StateInfo,
};

pub struct StorageRequest {
    pub match_idx: usize,
    pub subject_id: u64,
    pub object_id: u64,
    pub shared_node_info: SharedNodeInfo,
}

pub enum StorageResponseMut<'a, T> {
    Empty,
    Single(Option<&'a mut T>),
    Multi(IterMut<'a, T>),
}

impl<'a, T> Iterator for StorageResponseMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            StorageResponseMut::Empty => None,
            StorageResponseMut::Single(val) => val.take(),
            StorageResponseMut::Multi(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            StorageResponseMut::Empty => (0, Some(0)),
            StorageResponseMut::Single(val) => {
                if val.is_some() {
                    (1, Some(1))
                } else {
                    (0, Some(0))
                }
            }
            StorageResponseMut::Multi(iter) => iter.size_hint(),
        }
    }
}

impl<'a, T> ExactSizeIterator for StorageResponseMut<'a, T> {}

pub struct InstanceStorage<'p> {
    /// The match instances that can go to next state once an input event matches the specified
    /// match order, no shared entity is required. This only happens on the first event of a
    /// sub-pattern. Since sub-patterns are disjoint and each match order correspond to only one
    /// pattern event, there will be at most one such match instance for each match order.
    pub simple_instances: HashMap<usize, MatchInstance<'p>>,
    pub subject_instances: HashMap<(usize, u64), Vec<MatchInstance<'p>>>,
    pub object_instances: HashMap<(usize, u64), Vec<MatchInstance<'p>>>,
    pub endpoints_instances: HashMap<(usize, u64, u64), Vec<MatchInstance<'p>>>,

    pub freq_instance: HashMap<(usize, u64, u64), Vec<FreqInstance<'p>>>,

    pub output_instances: Vec<(u32, MatchInstance<'p>)>,
}

impl<'p> InstanceStorage<'p> {
    pub fn init_from_state_table(state_table: &StateTable) -> Self {
        let filter_infos = state_table.table.iter().map(|(_, filter_info)| filter_info);
        let simple_instances = Self::init_simple_instances(filter_infos);
        Self {
            simple_instances,
            subject_instances: HashMap::new(),
            object_instances: HashMap::new(),
            endpoints_instances: HashMap::new(),
            freq_instance: HashMap::new(),
            output_instances: Vec::new(),
        }
    }

    fn init_simple_instances<T>(
        filter_infos: impl Iterator<Item = T>,
    ) -> HashMap<usize, MatchInstance<'p>>
    where
        T: Borrow<FilterInfo>,
    {
        let mut simple_instances: HashMap<usize, MatchInstance<'p>> = HashMap::new();
        for (state_id, filter_info) in filter_infos.enumerate() {
            let filter_info = filter_info.borrow();
            if let FilterInfo::MatchIdxOnly {
                match_idx: match_ord,
            } = filter_info
            {
                simple_instances.insert(
                    *match_ord,
                    MatchInstance {
                        start_time: u64::MAX,
                        match_events: Box::new([]),
                        match_entities: Box::new([]),
                        state_id: state_id as u32,
                        event_ids: Box::new([]),
                    },
                );
            }
        }
        simple_instances
    }

    pub fn query_with_windowing<'a>(
        &'a mut self,
        request: &StorageRequest,
        window_bound: u64,
    ) -> StorageResponseMut<'a, MatchInstance<'p>> {
        let match_idx = request.match_idx;
        let subject_id = request.subject_id;
        let object_id = request.object_id;

        let is_valid = |inst: &MatchInstance<'p>| inst.start_time >= window_bound;

        match request.shared_node_info {
            SharedNodeInfo::None => {
                StorageResponseMut::Single(self.simple_instances.get_mut(&match_idx))
            }

            SharedNodeInfo::Subject => Self::apply_filter_mut(
                &mut self.subject_instances,
                (match_idx, subject_id),
                is_valid,
            ),

            SharedNodeInfo::Object => {
                Self::apply_filter_mut(&mut self.object_instances, (match_idx, object_id), is_valid)
            }

            SharedNodeInfo::Both => Self::apply_filter_mut(
                &mut self.endpoints_instances,
                (match_idx, subject_id, object_id),
                is_valid,
            ),
        }
    }

    pub fn query_freq_instances<'a>(
        &'a mut self,
        request: &StorageRequest,
        window_bound: u64,
    ) -> StorageResponseMut<'a, FreqInstance<'p>> {
        let is_valid =
            |inst: &FreqInstance<'p>| !inst.is_full() && inst.instance.start_time >= window_bound;
        Self::apply_filter_mut(
            &mut self.freq_instance,
            (request.match_idx, request.subject_id, request.object_id),
            is_valid,
        )
    }

    fn apply_filter_mut<K, V>(
        storage: &mut HashMap<K, Vec<V>>,
        filter: K,
        is_valid: impl Fn(&V) -> bool,
    ) -> StorageResponseMut<'_, V>
    where
        K: Eq + Hash,
    {
        if let Some(instances) = storage.get_mut(&filter) {
            instances.retain(is_valid);
            StorageResponseMut::Multi(instances.iter_mut())
        } else {
            StorageResponseMut::Empty
        }
    }

    pub fn store_new_instances(
        &mut self,
        new_instances: impl Iterator<Item = MatchInstance<'p>>,
        state_table: &StateTable,
    ) {
        for new_instance in new_instances {
            let (state_info, filter_info) = state_table.get(new_instance.state_id);
            if let StateInfo::Output { subpattern_id } = state_info {
                self.output_instances.push((subpattern_id, new_instance));
                continue;
            }
            match Self::extract_filter(&new_instance, &filter_info) {
                Some(Filter::Subject { match_idx, subject }) => {
                    self.subject_instances
                        .entry((match_idx, subject))
                        .or_default()
                        .push(new_instance);
                }
                Some(Filter::Object { match_idx, object }) => {
                    self.object_instances
                        .entry((match_idx, object))
                        .or_default()
                        .push(new_instance);
                }
                Some(Filter::Endpoints {
                    match_idx,
                    subject,
                    object,
                }) => {
                    self.endpoints_instances
                        .entry((match_idx, subject, object))
                        .or_default()
                        .push(new_instance);
                }
                _ => continue,
            }
        }
    }

    pub fn store_freq_instances(
        &mut self,
        new_instances: impl Iterator<Item = ((usize, u64, u64), FreqInstance<'p>)>,
    ) {
        for (filter, instance) in new_instances {
            self.freq_instance.entry(filter).or_default().push(instance);
        }
    }

    fn extract_filter(instance: &MatchInstance, filter_info: &FilterInfo) -> Option<Filter> {
        let endpoints_extractor = |event: &UniversalMatchEvent| (event.subject_id, event.object_id);
        let filter = match filter_info {
            FilterInfo::None => return None,
            FilterInfo::MatchIdxOnly { match_idx } => Filter::MatchIdxOnly {
                match_idx: *match_idx,
            },
            FilterInfo::Subject { match_idx, subject } => {
                let subject = subject.get_entity(&instance.match_events, endpoints_extractor)?;
                Filter::Subject {
                    match_idx: *match_idx,
                    subject,
                }
            }
            FilterInfo::Object { match_idx, object } => {
                let object = object.get_entity(&instance.match_events, endpoints_extractor)?;
                Filter::Object {
                    match_idx: *match_idx,
                    object,
                }
            }
            FilterInfo::Endpoints {
                match_idx,
                subject,
                object,
            } => {
                let subject = subject.get_entity(&instance.match_events, endpoints_extractor)?;
                let object = object.get_entity(&instance.match_events, endpoints_extractor)?;
                Filter::Endpoints {
                    match_idx: *match_idx,
                    subject,
                    object,
                }
            }
        };
        Some(filter)
    }
}

#[cfg(test)]
mod tests {
    use crate::process_layers::composition_layer::entity_encode::EntityEncode;

    use super::*;

    #[test]
    fn test_init_simple_instances() {
        let state_table = [
            FilterInfo::MatchIdxOnly { match_idx: 0 },
            FilterInfo::Subject {
                match_idx: 1,
                subject: EntityEncode::object_of(0),
            },
            FilterInfo::None,
            FilterInfo::MatchIdxOnly { match_idx: 2 },
            FilterInfo::None,
        ];

        let simple_instances = InstanceStorage::init_simple_instances(state_table.iter());
        assert_eq!(simple_instances.len(), 2);

        let placeholder = &simple_instances[&0];
        assert_eq!(placeholder.state_id, 0);

        let placeholder = &simple_instances[&2];
        assert_eq!(placeholder.state_id, 3);
    }
}
