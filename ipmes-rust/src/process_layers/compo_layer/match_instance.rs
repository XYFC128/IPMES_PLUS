use std::collections::HashSet;

use super::StateData;
use crate::process_layers::matching_layer::PartialMatchEvent;
use crate::universal_match_event::UniversalMatchEvent;

#[derive(Clone)]
pub struct MatchInstance<'p> {
    pub start_time: u64,
    pub match_events: Vec<UniversalMatchEvent<'p>>,
    pub event_ids: HashSet<u64>,
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
    fn contains_event(&self, input_event_id: u64) -> bool {
        self.event_ids.contains(&input_event_id)
    }

    pub fn check_entity_uniqueness(&self) -> bool {
        todo!()
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
