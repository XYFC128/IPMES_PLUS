use std::collections::HashSet;

use thiserror::Error;

#[derive(Clone, Copy)]
pub enum StateInfo {
    Default { next_state: u32 },
    Output { subpattern_id: u32 },
    InitFreq { next_state: u32 },
    AggFreq { next_state: u32, frequency: u32 },
}

type EntityId = u64;

#[derive(Clone)]
pub enum StateData {
    Default {
        next_state: u32,
    },
    InitFreq {
        next_state: u32,
    },
    AggFreq {
        next_state: u32,
        frequency: u32,
        current_set: HashSet<u64>,
    },
}

#[derive(Error, Debug)]
pub enum StateDataConstructionError {
    #[error("output state should not be constructed into an instance")]
    FromOutputState,
}

impl TryFrom<StateInfo> for StateData {
    type Error = StateDataConstructionError;

    fn try_from(value: StateInfo) -> Result<Self, Self::Error> {
        match value {
            StateInfo::Default { next_state } => Ok(StateData::Default { next_state }),
            StateInfo::Output { subpattern_id } => Err(StateDataConstructionError::FromOutputState),
            StateInfo::InitFreq { next_state } => Ok(StateData::InitFreq { next_state }),
            StateInfo::AggFreq {
                next_state,
                frequency,
            } => Ok(StateData::AggFreq {
                next_state,
                frequency,
                current_set: HashSet::new(),
            }),
        }
    }
}
