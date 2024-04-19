use super::StateData;
use crate::process_layers::matching_layer::PartialMatchEvent;
use crate::universal_match_event::UniversalMatchEvent;

#[derive(Clone)]
pub struct MatchInstance<'p> {
    pub start_time: u64,
    pub match_events: Vec<UniversalMatchEvent<'p>>,
    pub state_data: StateData,
}

impl<'p> MatchInstance<'p> {
    pub fn accept(&mut self, match_event: &PartialMatchEvent<'p>) -> InstanceAction<'p> {
        if self.contains_event(match_event) {
            return InstanceAction::Remain;
        }

        match &mut self.state_data {
            StateData::Normal { next_state } => InstanceAction::NewInstance {
                new_state_id: *next_state,
                new_event: match_event.into(),
            },
            StateData::InitFlow {
                next_state,
                agg_state,
            } => todo!(),
            StateData::AggFlow {
                next_state,
                reachable,
            } => todo!(),
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
    fn contains_event(&self, match_event: &PartialMatchEvent) -> bool {
        // FIXME: This is an extremely slow operation due to high cache miss!
        for event in &self.match_events {
            for event_id in event.event_ids.iter() {
                if *event_id == match_event.input_event.event_id {
                    return true;
                }
            }
        }
        false
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
