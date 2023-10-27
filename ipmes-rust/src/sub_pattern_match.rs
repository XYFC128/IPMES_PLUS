use crate::input_edge::InputEdge;
use crate::match_edge::MatchEdge;
use std::collections::HashMap;
use std::rc::Rc;

pub struct SubPatternMatch<'p> {
    pub timestamp: u64,
    pub matched_nodes_table: HashMap<usize, u64>,
    pub matched_edges_table: HashMap<usize, Rc<InputEdge>>,
    pub match_edges: Vec<MatchEdge<'p>>,
}
