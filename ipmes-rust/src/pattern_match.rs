use std::cmp::Ordering;
use crate::input_edge::InputEdge;
use itertools::Itertools;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::rc::Rc;

/// Complete Pattern Match
#[derive(Clone)]
pub struct PatternMatch {
    /// Matched edges of this pattern. i-th element is the input edge that matches pattern edge i
    pub matched_edges: Vec<Rc<InputEdge>>,
    pub earliest_time: u64,
    pub latest_time: u64,
}

impl fmt::Display for PatternMatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.matched_edges.iter().map(|e| e.id).join(", ")
        )
    }
}

impl Eq for PatternMatch {}

impl PartialEq for PatternMatch {
    fn eq(&self, other: &Self) -> bool {
        zip(&self.matched_edges, &other.matched_edges).all(|(a, b)| a.id.eq(&b.id))
    }
}

impl Hash for PatternMatch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for edge in &self.matched_edges {
            edge.id.hash(state);
        }
    }
}

#[derive(Clone)]
pub struct EarliestFirst<'p>(pub PatternMatch);
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
