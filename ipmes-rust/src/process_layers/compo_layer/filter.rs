use super::entity_encode::EntityEncode;

#[derive(Clone, Copy, Debug)]
pub enum Filter {
    MatchOrdOnly {
        match_ord: usize,
    },
    Subject {
        match_ord: usize,
        subject: u64,
    },
    Object {
        match_ord: usize,
        object: u64,
    },
    Endpoints {
        match_ord: usize,
        subject: u64,
        object: u64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilterInfo {
    None,
    MatchOrdOnly {
        match_ord: usize,
    },
    Subject {
        match_ord: usize,
        subject: EntityEncode,
    },
    Object {
        match_ord: usize,
        object: EntityEncode,
    },
    Endpoints {
        match_ord: usize,
        subject: EntityEncode,
        object: EntityEncode,
    },
}
