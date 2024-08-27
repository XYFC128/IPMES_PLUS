use std::slice::Iter;

use super::filter::FilterInfo;
use super::pattern_info::SharedNodeInfo;
use super::StateInfo;
use crate::pattern::{PatternEventType, SubPattern};
use crate::process_layers::composition_layer::entity_encode::EntityEncode;
use ahash::{HashMap, HashMapExt};
use log::debug;

pub struct StateTable {
    pub table: Vec<(StateInfo, FilterInfo)>,
    pub shared_node_info: Vec<SharedNodeInfo>,
}

impl StateTable {
    pub fn new(decomposition: &[SubPattern]) -> Self {
        let mut table = vec![];
        let mut shared_node_info = vec![];
        let mut match_idx = 0;
        for sub_pattern in decomposition {
            let mut entity_table = HashMap::new();

            for (event_idx, pattern) in sub_pattern.events.iter().enumerate() {
                let shared_subject = entity_table.get(&pattern.subject.id).cloned();
                let shared_object = entity_table.get(&pattern.object.id).cloned();
                let filter_info = match (shared_subject, shared_object) {
                    (None, None) => FilterInfo::MatchIdxOnly { match_idx },
                    (None, Some(object)) => FilterInfo::Object { match_idx, object },
                    (Some(subject), None) => FilterInfo::Subject { match_idx, subject },
                    (Some(subject), Some(object)) => FilterInfo::Endpoints {
                        match_idx,
                        subject,
                        object,
                    },
                };
                shared_node_info.push(filter_info.into());

                let next_state = (table.len() + 1) as u32;
                match pattern.event_type {
                    PatternEventType::Default | PatternEventType::Flow => {
                        table.push((StateInfo::Default { next_state }, filter_info));
                    }
                    PatternEventType::Frequency(frequency) => {
                        table.push((StateInfo::InitFreq { next_state }, filter_info));
                        let next_state = next_state + 1;
                        table.push((
                            StateInfo::AggFreq {
                                next_state,
                                frequency,
                            },
                            FilterInfo::Endpoints {
                                match_idx,
                                subject: EntityEncode::subject_of(event_idx),
                                object: EntityEncode::object_of(event_idx),
                            },
                        ))
                    }
                }

                entity_table.insert(pattern.subject.id, EntityEncode::subject_of(event_idx));
                entity_table.insert(pattern.object.id, EntityEncode::object_of(event_idx));
                match_idx += 1;
            }

            table.push((
                StateInfo::Output {
                    subpattern_id: sub_pattern.id as u32,
                },
                FilterInfo::None,
            ));
        }

        debug!("state table:\n{:#?}", table);

        Self {
            table,
            shared_node_info,
        }
    }

    pub fn iter(&self) -> Iter<'_, (StateInfo, FilterInfo)> {
        self.table.iter()
    }

    pub fn get(&self, state_id: u32) -> (StateInfo, FilterInfo) {
        self.table[state_id as usize]
    }

    pub fn get_state_info(&self, state_id: u32) -> &StateInfo {
        &self.table[state_id as usize].0
    }

    pub fn get_filter_info(&self, state_id: u32) -> &FilterInfo {
        &self.table[state_id as usize].1
    }

    pub fn get_shared_node_info(&self, match_idx: usize) -> SharedNodeInfo {
        self.shared_node_info[match_idx]
    }

    pub fn get_next_state(&self, state_id: u32) -> u32 {
        match self.table[state_id as usize].0 {
            StateInfo::Default { next_state } => next_state,
            StateInfo::Output { subpattern_id: _ } => todo!(),
            StateInfo::InitFreq { next_state } => next_state,
            StateInfo::AggFreq {
                next_state,
                frequency: _,
            } => next_state,
            StateInfo::AggFlow { next_state } => next_state,
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
                FilterInfo::MatchIdxOnly { match_idx: 0 },
            ),
            (
                StateInfo::Default { next_state: 2 },
                FilterInfo::Subject {
                    match_idx: 1,
                    subject: EntityEncode::object_of(0),
                },
            ),
            (
                StateInfo::Default { next_state: 3 },
                FilterInfo::Object {
                    match_idx: 2,
                    object: EntityEncode::object_of(1),
                },
            ),
            (
                StateInfo::Default { next_state: 4 },
                FilterInfo::Endpoints {
                    match_idx: 3,
                    // the shared subject could be object_of(1), but 2 is better
                    // since it is closer
                    subject: EntityEncode::object_of(2),
                    object: EntityEncode::subject_of(2),
                },
            ),
            (StateInfo::Output { subpattern_id: 0 }, FilterInfo::None),
        ];
        let table = StateTable::new(&decomposition);
        assert_eq!(table.table, &ans)
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
                FilterInfo::MatchIdxOnly { match_idx: 0 },
            ),
            (
                StateInfo::InitFreq { next_state: 2 },
                FilterInfo::Subject {
                    match_idx: 1,
                    subject: EntityEncode::object_of(0),
                },
            ),
            (
                StateInfo::AggFreq {
                    next_state: 3,
                    frequency: 7,
                },
                FilterInfo::Endpoints {
                    match_idx: 1,
                    subject: EntityEncode::subject_of(1),
                    object: EntityEncode::object_of(1),
                },
            ),
            (StateInfo::Output { subpattern_id: 0 }, FilterInfo::None),
        ];
        let table = StateTable::new(&decomposition);
        assert_eq!(table.table, &ans)
    }
}
