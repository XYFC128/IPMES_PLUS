use std::collections::{HashMap, HashSet};

use crate::pattern::PatternEventType;
use crate::process_layers::matching_layer::PartialMatchEvent;
use crate::sub_pattern::SubPattern;
use crate::universal_match_event::UniversalMatchEvent;

use super::entity_encode::EntityEncode;
use super::filter::{Filter, FilterInfo};
use super::match_instance::InstanceAction;
use super::{InstanceStorage, MatchInstance, StateData, StateInfo};

pub struct InstanceRunner<'p> {
    pub state_table: Vec<(StateInfo, FilterInfo)>,
    new_instance: Vec<(Filter, MatchInstance<'p>)>,
    pub output_buffer: Vec<(u32, MatchInstance<'p>)>,
}

impl<'p> InstanceRunner<'p> {
    pub fn new(decomposition: &[SubPattern]) -> Self {
        let mut state_table = Vec::new();
        let mut match_ord = 0;
        for sub_pattern in decomposition {
            let mut entity_table = HashMap::new();

            for (event_idx, pattern) in sub_pattern.events.iter().enumerate() {
                let shared_subject = entity_table.get(&pattern.subject.id).cloned();
                let shared_object = entity_table.get(&pattern.object.id).cloned();
                let filter_info = match (shared_subject, shared_object) {
                    (None, None) => FilterInfo::MatchOrdOnly { match_ord },
                    (None, Some(object)) => FilterInfo::Object { match_ord, object },
                    (Some(subject), None) => FilterInfo::Subject { match_ord, subject },
                    (Some(subject), Some(object)) => FilterInfo::Endpoints {
                        match_ord,
                        subject,
                        object,
                    },
                };

                let next_state = (state_table.len() + 1) as u32;
                match pattern.event_type {
                    PatternEventType::Default | PatternEventType::Flow => {
                        state_table.push((StateInfo::Default { next_state }, filter_info));
                    }
                    PatternEventType::Frequency(frequency) => {
                        state_table.push((StateInfo::InitFreq { next_state }, filter_info));
                        let next_state = next_state + 1;
                        state_table.push((
                            StateInfo::AggFreq {
                                next_state,
                                frequency,
                            },
                            filter_info,
                        ))
                    }
                }

                entity_table.insert(pattern.subject.id, EntityEncode::subject_of(event_idx));
                entity_table.insert(pattern.object.id, EntityEncode::object_of(event_idx));
                match_ord += 1;
            }

            state_table.push((
                StateInfo::Output {
                    subpattern_id: sub_pattern.id as u32,
                },
                FilterInfo::None,
            ));
        }

        Self {
            state_table,
            new_instance: Vec::new(),
            output_buffer: Vec::new(),
        }
    }

    pub fn run(&mut self, instance: &mut MatchInstance<'p>, match_event: &PartialMatchEvent<'p>) {
        if let InstanceAction::NewInstance {
            new_state_id,
            new_event,
        } = instance.accept(match_event)
        {
            if let StateInfo::Output { subpattern_id } = self.state_table[new_state_id as usize].0 {
                if let Some(output) = self.new_output_from(&instance, new_event, subpattern_id) {
                    self.output_buffer.push(output);
                }
            } else if let Some(data) = self.new_instance_from(&instance, new_state_id, new_event) {
                self.new_instance.push(data);
            }
        }
    }

    pub fn new_output_from(
        &self,
        instance: &MatchInstance<'p>,
        new_event: UniversalMatchEvent<'p>,
        subpattern_id: u32,
    ) -> Option<(u32, MatchInstance<'p>)> {
        let old_filter = self.state_table[instance.state_id as usize].1;
        let match_entities = match old_filter {
            FilterInfo::Subject {
                match_ord: _,
                subject: _,
            } => MatchInstance::dup_extend_entities_list(
                &instance.match_entities,
                new_event.subject_id,
                new_event.matched.subject.id as u64,
            )?,
            FilterInfo::Object {
                match_ord: _,
                object: _,
            } => MatchInstance::dup_extend_entities_list(
                &instance.match_entities,
                new_event.object_id,
                new_event.matched.object.id as u64,
            )?,
            _ => instance.match_entities.clone(),
        };

        let events = &instance.match_events;
        let mut new_match_events = Vec::with_capacity(events.len() + 1);
        new_match_events.clone_from_slice(events);
        new_match_events.push(new_event);

        Some((
            subpattern_id,
            MatchInstance {
                match_events: new_match_events.into_boxed_slice(),
                match_entities,
                ..instance.clone()
            },
        ))
    }

    pub fn new_instance_from(
        &self,
        old_instance: &MatchInstance<'p>,
        new_state_id: u32,
        new_event: UniversalMatchEvent<'p>,
    ) -> Option<(Filter, MatchInstance<'p>)> {
        let events = &old_instance.match_events;

        let (state_info, filter_info) = self.state_table[new_state_id as usize];

        let new_instance = match state_info {
            StateInfo::Default { next_state: _ } | StateInfo::InitFreq { next_state: _ } => {
                let mut event_ids = old_instance.event_ids.clone().into_vec();
                event_ids.extend(new_event.event_ids.iter());
                event_ids.sort();
                let event_ids = event_ids.into_boxed_slice();

                let old_filter = self.state_table[old_instance.state_id as usize].1;
                let match_entities = match old_filter {
                    FilterInfo::Subject {
                        match_ord: _,
                        subject: _,
                    } => MatchInstance::dup_extend_entities_list(
                        &old_instance.match_entities,
                        new_event.subject_id,
                        new_event.matched.subject.id as u64,
                    )?,
                    FilterInfo::Object {
                        match_ord: _,
                        object: _,
                    } => MatchInstance::dup_extend_entities_list(
                        &old_instance.match_entities,
                        new_event.object_id,
                        new_event.matched.object.id as u64,
                    )?,
                    _ => old_instance.match_entities.clone(),
                };

                let mut new_match_events = Vec::with_capacity(events.len() + 1);
                new_match_events.clone_from_slice(events);
                new_match_events.push(new_event);
                let new_match_events = new_match_events.into_boxed_slice();

                MatchInstance {
                    start_time: old_instance.start_time,
                    match_events: new_match_events,
                    match_entities,
                    state_data: state_info.try_into().unwrap(),
                    state_id: new_state_id,
                    event_ids,
                }
            }
            StateInfo::AggFreq {
                next_state,
                frequency,
            } => {
                let mut current_set = HashSet::new();
                for event_id in new_event.event_ids.iter() {
                    current_set.insert(*event_id);
                }

                MatchInstance {
                    state_data: StateData::AggFreq {
                        next_state,
                        frequency,
                        current_set,
                    },
                    state_id: next_state,
                    ..old_instance.clone()
                }
            }
            _ => panic!("should not reach here"),
        };

        let endpoints_extractor = |e: &UniversalMatchEvent| (e.subject_id, e.object_id);
        let filter = match filter_info {
            FilterInfo::None => panic!("Should not reach here"),
            FilterInfo::MatchOrdOnly { match_ord } => Filter::MatchOrdOnly { match_ord },
            FilterInfo::Subject { match_ord, subject } => Filter::Subject {
                match_ord,
                subject: subject
                    .get_entity_unchecked(&new_instance.match_events, endpoints_extractor),
            },
            FilterInfo::Object { match_ord, object } => Filter::Object {
                match_ord,
                object: object
                    .get_entity_unchecked(&new_instance.match_events, endpoints_extractor),
            },
            FilterInfo::Endpoints {
                match_ord,
                subject,
                object,
            } => Filter::Endpoints {
                match_ord,
                subject: subject
                    .get_entity_unchecked(&new_instance.match_events, endpoints_extractor),
                object: object
                    .get_entity_unchecked(&new_instance.match_events, endpoints_extractor),
            },
        };

        Some((filter, new_instance))
    }

    pub fn store_new_instances(&mut self, storage: &mut InstanceStorage<'p>) {
        let new_instance = std::mem::take(&mut self.new_instance);
        for (filter, instance) in new_instance.into_iter() {
            match filter {
                Filter::MatchOrdOnly { match_ord } => storage
                    .simple_instances
                    .entry(match_ord)
                    .or_default()
                    .push(instance),
                Filter::Subject { match_ord, subject } => storage
                    .subject_instances
                    .entry((match_ord, subject))
                    .or_default()
                    .push(instance),
                Filter::Object { match_ord, object } => storage
                    .object_instances
                    .entry((match_ord, object))
                    .or_default()
                    .push(instance),
                Filter::Endpoints {
                    match_ord,
                    subject,
                    object,
                } => storage
                    .endpoints_instances
                    .entry((match_ord, subject, object))
                    .or_default()
                    .push(instance),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pattern::Pattern;

    use super::*;

    #[test]
    fn test_new() {
        let pattern = Pattern::from_graph(
            &["v1", "v2", "v3", "v4"],
            &[
                (0, 1, "e1"), // share none
                (1, 2, "e2"), // share subject
                (3, 2, "e3"), // share object
                (2, 3, "e4"), // share both
            ],
            false,
        );

        let decomposition = [SubPattern {
            id: 0,
            events: pattern.events.iter().collect(),
        }];

        let ans = [
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
            (
                StateInfo::Default { next_state: 3 },
                FilterInfo::Object {
                    match_ord: 2,
                    object: EntityEncode::object_of(1),
                },
            ),
            (
                StateInfo::Default { next_state: 4 },
                FilterInfo::Endpoints {
                    match_ord: 3,
                    // the shared subject could be object_of(1), but 2 is better
                    // since it is closer
                    subject: EntityEncode::object_of(2), 
                    object: EntityEncode::subject_of(2),
                },
            ),
            (StateInfo::Output { subpattern_id: 0 }, FilterInfo::None),
        ];
        let runner = InstanceRunner::new(&decomposition);
        assert_eq!(runner.state_table, &ans)
    }
}
