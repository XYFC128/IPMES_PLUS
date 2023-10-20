use std::collections::{BinaryHeap, HashSet};
use std::rc::Rc;
use crate::match_edge::MatchEdge;
use crate::pattern_match::PatternMatch;
use crate::sub_pattern_match::SubPatternMatch;


struct Entry<'p> {
    earliest_time: u64,
    match_edges: Vec<MatchEdge<'p>>,
    num_nodes: usize,
}

impl<'p> From<Entry<'p>> for PatternMatch {
    fn from(value: Entry<'p>) -> Self {
        todo!()
    }
}

pub struct NaiveJoinLayer<'p, P>
{
    prev_layer: P,
    table: HashSet<Rc<Entry<'p>>>,
    order: BinaryHeap<Rc<Entry<'p>>>,
    full_matches: Vec<PatternMatch>,
}

impl<'p, P> NaiveJoinLayer<'p, P> {

}

impl<'p, P> Iterator for NaiveJoinLayer<'p, P>
    where
        P: Iterator<Item = Vec<SubPatternMatch<'p>>>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Self::Item> {
        while self.full_matches.is_empty() {
            let sub_pattern_matches = self.prev_layer.next()?;

        }
        self.full_matches.pop()
    }
}