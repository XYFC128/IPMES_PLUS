use crate::match_event::MatchEvent;

/// Cache the extra space needed during merge process to prevent frequent allocation
pub struct MergeCache<'p> {
    /// make sure the largest pattern edge id is "pattern.edges.len() - 1"
    pub used_nodes: Vec<bool>,

    pub timestamps: Vec<u64>,

    pub merged_edge: Vec<MatchEvent<'p>>,

    pub merged_nodes: Vec<(u64, u64)>,
}

impl MergeCache {
    pub fn clear_all(&mut self) {
        self.clear_edges();
        self.clear_nodes();
    }

    /// clear caches related to merging edges
    pub fn clear_edge_related(&mut self) {
        self.timestamps.fill(0);
        self.merged_edge.clear();
    }

    /// clear caches related to merging nodes
    pub fn clear_node_related(&mut self) {
        self.used_nodes.fill(false);
        self.merged_nodes.clear();
    }
}