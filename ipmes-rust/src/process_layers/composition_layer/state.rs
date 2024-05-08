use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateInfo {
    Default { next_state: u32 },
    Output { subpattern_id: u32 },
    InitFreq { next_state: u32 },
    AggFreq { next_state: u32, frequency: u32 },
}

#[derive(Clone, Debug)]
pub enum StateData {
    Default {
        next_state: u32,
    },
    AggFreq {
        next_state: u32,
        frequency: u32,
        current_set: HashSet<u64>,
    },
    Output {
        subpattern_id: u32,
    },
    Dead,
}

#[cfg(test)]
impl Default for StateData {
    fn default() -> Self {
        Self::Default { next_state: 0 }
    }
}

impl From<StateInfo> for StateData {
    fn from(value: StateInfo) -> Self {
        match value {
            StateInfo::Default { next_state } => StateData::Default { next_state },
            StateInfo::Output { subpattern_id } => StateData::Output { subpattern_id },
            StateInfo::InitFreq { next_state } => StateData::Default { next_state },
            StateInfo::AggFreq {
                next_state,
                frequency,
            } => StateData::AggFreq {
                next_state,
                frequency,
                current_set: HashSet::new(),
            },
        }
    }
}
