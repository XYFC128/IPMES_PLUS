use crate::input_edge::InputEdge;
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::rc::Rc;

/// Complete Pattern Match
pub struct PatternMatch {
    /// Matched edges of this pattern. i-th element is the input edge that matches pattern edge i
    pub matched_edges: Vec<Rc<InputEdge>>,
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
