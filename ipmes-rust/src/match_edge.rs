use crate::pattern::Edge as PatternEdge;

#[derive(Clone)]
pub struct MatchEdge<'a> {
    pub id: u64,
    pub matched: &'a PatternEdge,
}