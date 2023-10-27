use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::input_edge::InputEdge;
use crate::match_edge::MatchEdge;

todo!(Don't use HashMap, use Vec (the keys only need to be stored once));
#[derive(Clone)]
pub struct SubPatternMatch<'p> {
    pub id: usize,
    /// The timestamp of the last edge (in "match_edges"), which is also the latest timestamp; indicating "current time".
    pub latest_time: u64,
    /// The timestamp of the earliest edge; for determining expiry.
    pub earliest_time: u64,

    /// todo: HashMap to  Vec (Don't "from" ord_match_layer)
    // pub matched_nodes_table: HashMap<usize, u64>,
    // pub matched_edges_table: HashMap<usize, Rc<InputEdge>>,

    pub node_list: Vec<Option<u64>>,
    pub edge_list: Vec<Option<u64>>,

    /// sort this by id for uniqueness determination
    pub match_edges: Vec<MatchEdge<'p>>,
}

impl SubPatternMatch {
    pub fn set_id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }

    pub fn merge_matches(sub_pattern_match1: &Self, sub_pattern_match2: &Self) -> Self {
        let mut sub_pattern_match = sub_pattern_match1.clone();
        sub_pattern_match.matched_nodes_table.extend(&sub_pattern_match2.matched_nodes_table);
        sub_pattern_match.matched_edges_table.extend(&sub_pattern_match2.matched_edges_table);
        sub_pattern_match.match_edges.append(&mut sub_pattern_match2.match_edges.clone());

        sub_pattern_match
    }
}

#[derive(Debug)]
pub struct EarliestFirst<'p>(pub SubPatternMatch<'p>);

impl Eq for EarliestFirst<'_> {}

impl PartialEq<Self> for EarliestFirst<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.earliest_time.eq(&other.0.earliest_time)
    }
}

impl Ord for EarliestFirst<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.earliest_time.cmp(&other.0.earliest_time).reverse()
    }
}

impl PartialOrd<Self> for EarliestFirst<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'p> AsRef<SubPatternMatch<'p>> for EarliestFirst<'p> {
    fn as_ref(&self) -> &SubPatternMatch<'p> {
        self.0.as_ref()
    }
}