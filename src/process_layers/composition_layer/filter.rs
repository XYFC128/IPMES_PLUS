use super::entity_encode::EntityEncode;

/// Information for shared-node relation, with ordinary entity ids.
#[derive(Clone, Copy, Debug)]
pub enum Filter {
    MatchIdxOnly {
        match_idx: usize,
    },
    Subject {
        match_idx: usize,
        subject: u64,
    },
    Object {
        match_idx: usize,
        object: u64,
    },
    Endpoints {
        match_idx: usize,
        subject: u64,
        object: u64,
    },
}

/// Information for shared-node relation, with entities encoded as `EntityEncode`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilterInfo {
    None,
    MatchIdxOnly {
        match_idx: usize,
    },
    Subject {
        match_idx: usize,
        subject: EntityEncode,
    },
    Object {
        match_idx: usize,
        object: EntityEncode,
    },
    Endpoints {
        match_idx: usize,
        subject: EntityEncode,
        object: EntityEncode,
    },
}
