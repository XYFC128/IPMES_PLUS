use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::pattern::PatternEventType;
use crate::process_layers::matching_layer::PartialMatchEvent;
use crate::sub_pattern::SubPattern;

use super::entity_encode::EntityEncode;
use super::filter::{Filter, FilterInfo};
use super::match_instance::InstanceAction;
use super::{InstanceStorage, MatchInstance, StateData, StateInfo, UniversalMatchEvent};

pub struct InstanceRunner<'p> {
    pub state_table: Vec<(StateInfo, FilterInfo)>,
    new_instance: Vec<(Filter, MatchInstance<'p>)>,
    pub output_buffer: Vec<(u32, Box<[UniversalMatchEvent<'p>]>)>,
}

impl<'p> InstanceRunner<'p> {
    pub fn new(decomposition: &[SubPattern]) -> Self {
        let mut state_table = Vec::new();
        let mut match_ord = 0;
        for sub_pattern in decomposition {
            let mut entity_table = HashMap::new();

            for (event_idx, pattern) in sub_pattern.events.iter().enumerate() {
                let shared_subject = entity_table.get(&pattern.subject).cloned();
                let shared_object = entity_table.get(&pattern.object).cloned();
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
                    PatternEventType::Default => {
                        state_table.push((StateInfo::Normal { next_state }, filter_info));
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
                    PatternEventType::Flow => {
                        let agg_state = next_state;
                        let next_state = next_state + 1;
                        state_table.push((
                            StateInfo::InitFlow {
                                next_state,
                                agg_state,
                            },
                            filter_info,
                        ));
                        state_table.push((StateInfo::AggFlow { next_state }, filter_info))
                    }
                }

                entity_table.insert(pattern.subject, EntityEncode::subject_of(event_idx));
                entity_table.insert(pattern.object, EntityEncode::object_of(event_idx));
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
                self.output_buffer
                    .push(self.new_output_from(instance, new_event, subpattern_id));
            } else {
                self.new_instance
                    .push(self.new_instance_from(&instance, new_state_id, new_event));
            }
        }
    }

    pub fn new_output_from(
        &self,
        instance: &mut MatchInstance<'p>,
        new_event: UniversalMatchEvent<'p>,
        subpattern_id: u32,
    ) -> (u32, Box<[UniversalMatchEvent<'p>]>) {
        let events = &instance.match_events;
        let mut new_match_events = Vec::with_capacity(events.len() + 1);
        new_match_events.clone_from(events);
        new_match_events.push(new_event);

        (subpattern_id, new_match_events.into_boxed_slice())
    }

    pub fn new_instance_from(
        &self,
        old_instance: &MatchInstance<'p>,
        new_state_id: u32,
        new_event: UniversalMatchEvent<'p>,
    ) -> (Filter, MatchInstance<'p>) {
        let events = &old_instance.match_events;

        let (state_info, filter_info) = self.state_table[new_state_id as usize];

        let new_instance = match state_info {
            StateInfo::Normal { next_state: _ }
            | StateInfo::InitFlow {
                next_state: _,
                agg_state: _,
            }
            | StateInfo::InitFreq { next_state: _ } => {
                let mut new_match_events = Vec::with_capacity(events.len() + 1);
                new_match_events.clone_from(events);
                new_match_events.push(new_event);

                MatchInstance {
                    start_time: old_instance.start_time,
                    match_events: new_match_events,
                    state_data: state_info.try_into().unwrap(),
                }
            }
            StateInfo::AggFlow { next_state } => {
                let mut reachable = HashSet::new();
                reachable.insert(new_event.subject_id);
                reachable.insert(new_event.object_id);

                MatchInstance {
                    state_data: StateData::AggFlow {
                        next_state,
                        reachable,
                    },
                    ..old_instance.clone()
                }
            }
            StateInfo::AggFreq {
                next_state,
                frequency,
            } => {
                let mut current_set = HashSet::new();
                for input in new_event.input_events.iter() {
                    current_set.insert(Rc::clone(input).into());
                }

                MatchInstance {
                    state_data: StateData::AggFreq {
                        next_state,
                        frequency,
                        current_set,
                    },
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

        (filter, new_instance)
    }

    pub fn store_new_instances(&mut self, storage: &mut InstanceStorage<'p>) {
        let new_instance = std::mem::replace(&mut self.new_instance, vec![]);
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
    use crate::{pattern::Pattern, sub_pattern::decompose};

    use super::*;

    #[test]
    fn test() {
        let pattern = Pattern::from_graph(
            &["v1", "v2", "v3", "v4"],
            &[(0, 1, "e1"), (1, 2, "e2"), (1, 3, "e3")],
            false,
        );
        let decomposition = decompose(&pattern);
        println!("{:#?}", decomposition);
    }
}
