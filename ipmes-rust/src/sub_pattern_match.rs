use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::input_edge::InputEdge;
use crate::match_edge::MatchEdge;

pub struct SubPatternMatch<'p> {
    pub timestamp: u64,
    pub matched_nodes_table: HashMap<usize, u64>,
    pub matched_edges_table: HashMap<usize, Rc<InputEdge>>,
    pub match_edges: Vec<MatchEdge<'p>>,
}