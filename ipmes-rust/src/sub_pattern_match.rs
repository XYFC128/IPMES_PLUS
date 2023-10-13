use crate::match_edge::MatchEdge;

pub struct SubPatternMatch<'p> {
    pub nodes: Vec<usize>,
    // "MatchEdge" contains a pair: (edge_id_of_input_edge, pattern_edge)
    pub edges: Vec<MatchEdge<'p>>,
}