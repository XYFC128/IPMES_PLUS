use crate::match_edge::MatchEdge;
use crate::pattern_match::PatternMatch;
use crate::process_layers::naive_join_layer::entry_wrappers::UniqueEntry;
use itertools::Itertools;
use std::cmp::min;

#[derive(Debug)]
pub struct Entry<'p> {
    pub earliest_time: u64,
    /// sorted by input_edge.id
    pub match_edges: Vec<MatchEdge<'p>>,
    pub match_nodes: Vec<Option<u64>>,
    pub hash: u64,
}

impl<'p> Entry<'p> {
    /// Create an entry with infinite timestamp and empty match edges.
    ///
    /// Infinite timestamp avoid this entry being cleaned in windowing process
    pub fn placeholder(num_nodes: usize) -> Self {
        let match_nodes = vec![None; num_nodes];
        Self {
            earliest_time: u64::MAX,
            match_edges: Vec::new(),
            match_nodes,
            hash: 0,
        }
    }

    /// Create match result from match_edges. The input matched edges are assumed to be legal.
    ///
    /// **Important**: This function is inefficient and unsafe, and should only be used for
    /// testing purpose.
    pub fn from_match_edge<L>(match_edges: L, num_nodes: usize, num_edges: usize) -> Self
    where
        L: Iterator<Item = MatchEdge<'p>>,
    {
        let mut match_edges = match_edges.collect_vec();
        match_edges.sort_by(|a, b| a.input_edge.id.cmp(&b.input_edge.id));

        let mut match_nodes = vec![None; num_nodes];
        let mut mapping = vec![None; num_edges];
        let mut earliest_time = u64::MAX;
        for edge in &match_edges {
            match_nodes[edge.matched.start] = Some(edge.input_edge.start);
            match_nodes[edge.matched.end] = Some(edge.input_edge.end);

            mapping[edge.matched.id] = Some(edge);

            earliest_time = min(edge.input_edge.timestamp, earliest_time);
        }

        let hash = UniqueEntry::calc_hash(&mapping);

        Self {
            earliest_time,
            match_edges,
            match_nodes,
            hash,
        }
    }
}

impl<'p> From<Entry<'p>> for PatternMatch {
    fn from(value: Entry<'_>) -> Self {
        let mut match_edges = value.match_edges;
        match_edges.sort_by(|a, b| a.matched.id.cmp(&b.matched.id));

        let matched_edges = match_edges
            .into_iter()
            .map(|edge| edge.input_edge)
            .collect_vec();

        Self { matched_edges }
    }
}
