use std::rc::Rc;
use crate::input_edge::InputEdge;

/// Complete Pattern Match
pub struct PatternMatch {
    /// Matched edges of this pattern. i-th element is the input edge that matches pattern edge i
    pub matched_edges: Vec<Rc<InputEdge>>
}