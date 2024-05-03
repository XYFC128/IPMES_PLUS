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
                            FilterInfo::Endpoints {
                                match_ord,
                                subject: EntityEncode::subject_of(event_idx),
                                object: EntityEncode::object_of(event_idx),
                            },
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
        match instance.accept(match_event) {
            InstanceAction::NewInstance {
                new_state_id,
                new_event,
            } => {
                let old_filter = self.state_table[instance.state_id as usize].1;
                if let Some(new_instance) = instance.clone_extend(new_event, &old_filter) {
                    self.place_new_instance(new_instance, new_state_id);
                }
            }
            InstanceAction::MoveInstance { new_state_id } => {
                let new_instance = std::mem::replace(instance, MatchInstance::dead_default());
                self.place_new_instance(new_instance, new_state_id);
            }
            InstanceAction::Remain => {}
        }
    }

    fn place_new_instance(&mut self, mut new_instance: MatchInstance<'p>, new_state_id: u32) {
        let (state_info, filter_info) = self.state_table[new_state_id as usize];
        let state_data = if let StateInfo::AggFreq {
            next_state,
            frequency,
        } = state_info
        {
            let mut current_set = HashSet::new();
            if let Some(new_event) = new_instance.match_events.last() {
                for event_id in new_event.event_ids.iter() {
                    current_set.insert(*event_id);
                }
            }
            StateData::AggFreq {
                next_state,
                frequency,
                current_set,
            }
        } else {
            state_info.into()
        };
        new_instance.state_data = state_data;
        new_instance.state_id = new_state_id;

        if let StateInfo::Output { subpattern_id } = state_info {
            self.output_buffer.push((subpattern_id, new_instance));
        } else {
            let filter = Self::extract_filter(&new_instance, filter_info);
            self.new_instance.push((filter, new_instance));
        }
    }

    fn extract_filter(instance: &MatchInstance, filter_info: FilterInfo) -> Filter {
        let endpoints_extractor = |e: &UniversalMatchEvent| (e.subject_id, e.object_id);
        match filter_info {
            FilterInfo::None => unreachable!(),
            FilterInfo::MatchOrdOnly { match_ord } => Filter::MatchOrdOnly { match_ord },
            FilterInfo::Subject { match_ord, subject } => Filter::Subject {
                match_ord,
                subject: subject.get_entity_unchecked(&instance.match_events, endpoints_extractor),
            },
            FilterInfo::Object { match_ord, object } => Filter::Object {
                match_ord,
                object: object.get_entity_unchecked(&instance.match_events, endpoints_extractor),
            },
            FilterInfo::Endpoints {
                match_ord,
                subject,
                object,
            } => Filter::Endpoints {
                match_ord,
                subject: subject.get_entity_unchecked(&instance.match_events, endpoints_extractor),
                object: object.get_entity_unchecked(&instance.match_events, endpoints_extractor),
            },
        }
    }

    pub fn store_new_instances(&mut self, storage: &mut InstanceStorage<'p>) {
        let new_instance = std::mem::take(&mut self.new_instance);
        for (filter, instance) in new_instance.into_iter() {
            match filter {
                Filter::MatchOrdOnly { match_ord } => {
                    if let Some(instance) = storage.simple_instances.insert(match_ord, instance) {
                        log::warn!("Duplicated simple instance. Old instance: {:?}", instance);
                    }
                }
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

    #[test]
    fn test_frequency_state_generation() {
        let mut pattern = Pattern::from_graph(
            &["v1", "v2", "v3"],
            &[
                (0, 1, "e1"), // share none
                (1, 2, "e2"), // share subject
            ],
            false,
        );
        pattern.events[1].event_type = PatternEventType::Frequency(7);

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
                StateInfo::InitFreq { next_state: 2 },
                FilterInfo::Subject {
                    match_ord: 1,
                    subject: EntityEncode::object_of(0),
                },
            ),
            (
                StateInfo::AggFreq {
                    next_state: 3,
                    frequency: 7,
                },
                FilterInfo::Endpoints {
                    match_ord: 1,
                    subject: EntityEncode::subject_of(1),
                    object: EntityEncode::object_of(1),
                },
            ),
            (StateInfo::Output { subpattern_id: 0 }, FilterInfo::None),
        ];
        let runner = InstanceRunner::new(&decomposition);
        assert_eq!(runner.state_table, &ans)
    }
}
