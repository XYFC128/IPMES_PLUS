use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::input_edge::InputEdge;
use crate::match_edge::MatchEdge;

pub struct SubPatternMatch<'p> {
    pub timestamp: u64,
    pub nodes: HashSet<usize>,
    // "MatchEdge" contains a pair: (edge_id_of_input_edge, pattern_edge)
    // pub edges: Vec<MatchEdge<'p>>,

    // (pattern edge id, InputEdge)
    pub matched_edges_table: HashMap<usize, Rc<InputEdge>>,
}