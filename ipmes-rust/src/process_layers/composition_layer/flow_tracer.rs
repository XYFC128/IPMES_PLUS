use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use std::{
    cell::{Ref, RefCell},
    collections::hash_map::Entry,
};

#[derive(Clone)]
struct Node<E> {
    id: u64,
    parent: u64,
    update_time: u64,
    edge_data: E,
}

/// It maintains the nodes that can reach `v` for some node `v` in the graph. `E` is the type of
/// associated data on the arcs.
pub struct ReachSet<E> {
    nodes: HashMap<u64, Node<E>>,
    root_id: u64,
    update_time: u64,

    /// the updated node ids of the set since last time update_time is advanced
    updated_nodes: Vec<u64>,
}

impl<E> ReachSet<E>
where
    E: Clone,
{
    pub fn new(id: u64, time: u64) -> Self {
        Self {
            nodes: HashMap::new(),
            root_id: id,
            update_time: time,
            updated_nodes: Vec::new(),
        }
    }

    /// The root node `u` of `other` can connect to the root node `v` of this set. This operation
    /// correspond to set union operation. Given that `u` can reach `v`, all nodes that can reach
    /// `u` can reach `v` too, thus union the ReachSet of `u` to `v`.
    ///
    /// The union operation will ignore the nodes in `other` that are updated before `timebound`.
    ///
    /// The `edge_data` is the data associated with the arc `u -> v`.
    pub fn connect_from(&mut self, other: &Self, cur_time: u64, time_bound: u64, edge_data: E) {
        if other.root_id == self.root_id {
            return;
        }

        if cur_time > self.update_time {
            self.update_time = cur_time;
            self.updated_nodes.clear();
        }

        if let Some(other_root) = self.nodes.get_mut(&other.root_id) {
            if cur_time > other_root.update_time || other_root.parent != self.root_id {
                other_root.parent = self.root_id;
                other_root.update_time = cur_time;
                self.updated_nodes.push(other.root_id);
            }
        } else {
            self.nodes.insert(
                other.root_id,
                Node {
                    id: other.root_id,
                    parent: self.root_id,
                    update_time: cur_time,
                    edge_data,
                },
            );
            self.updated_nodes.push(other.root_id);
        }

        if other.get_update_time() < time_bound {
            return;
        }

        for node in other.nodes.values() {
            if node.update_time < time_bound || node.id == self.root_id {
                continue;
            }

            self.update_or_insert(node);
        }
    }

    /// Add new elements of `other` into this set. The root node `u` of `other` must already in
    /// this set, and the update time of `other` must not be newer than that of `u` in this set.
    pub fn apply_new_changes(&mut self, other: &Self) {
        if let Some(other_root) = self.nodes.get(&other.root_id) {
            if other_root.update_time != other.update_time {
                // the new changes is to late to be applied
                return;
            }
        } else {
            return; // `other.root` is not in this set
        }

        // NOTE: No need to clear updated_nodes here as the update_time is not advanced

        for node_id in &other.updated_nodes {
            if *node_id == self.root_id {
                continue;
            }
            let node = other.nodes.get(node_id).unwrap();
            self.update_or_insert(node);
        }
    }

    fn update_or_insert(&mut self, node: &Node<E>) {
        if let Some(our_node) = self.nodes.get_mut(&node.id) {
            if our_node.update_time < node.update_time {
                our_node.parent = node.parent;
                our_node.update_time = node.update_time;
                self.updated_nodes.push(node.id);
            }
        } else {
            self.nodes.insert(node.id, node.clone());
            self.updated_nodes.push(node.id);
        }
    }

    /// Returns `true` if some node of `id` can reach the root node of this set.
    pub fn query(&self, id: u64) -> bool {
        id == self.root_id || self.nodes.contains_key(&id)
    }

    /// Returns the root node's id
    pub fn get_id(&self) -> u64 {
        self.root_id
    }

    /// Returns the update time of this set.
    pub fn get_update_time(&self) -> u64 {
        self.update_time
    }

    /// Returns the update time of the node `src` in this set.
    /// Returns `None` if `src` is not in this set.
    pub fn get_update_time_of(&self, src: u64) -> Option<u64> {
        if src == self.root_id {
            Some(self.update_time)
        } else {
            self.nodes.get(&src).map(|node| node.update_time)
        }
    }

    /// Returns the node that are updated since last update of update time.
    pub fn get_updated_nodes(&self) -> &[u64] {
        &self.updated_nodes
    }

    /// Get the path from `src` to the root node of this set.
    ///
    /// Returns an iterator of the edge_data associated to the arcs along the path from `u` to the
    /// root of this set. If no path is found, the iterator will yeild nothing.
    pub fn query_path(&self, src: u64) -> PathIter<E> {
        if let Some(node) = self.nodes.get(&src) {
            PathIter {
                reach_set: self,
                cur_node: Some(node),
            }
        } else {
            PathIter {
                reach_set: self,
                cur_node: None,
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn del_outdated(&mut self, time_bound: u64) {
        if self.update_time < time_bound {
            self.nodes.clear();
        } else {
            self.nodes.retain(|_, nd| nd.update_time >= time_bound);
        }
    }
}

pub struct PathIter<'a, E> {
    reach_set: &'a ReachSet<E>,
    cur_node: Option<&'a Node<E>>,
}

impl<'a, E> Iterator for PathIter<'a, E>
where
    E: Clone,
{
    type Item = E;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur_node) = self.cur_node {
            let output = cur_node.edge_data.clone();

            if cur_node.parent == self.reach_set.root_id {
                self.cur_node = None;
            } else {
                self.cur_node = self.reach_set.nodes.get(&cur_node.parent);
            }

            Some(output)
        } else {
            None
        }
    }
}

/// Incrementally trace the flow. [`E`] is the type of the data associated with the arcs in the
/// graph. The current implementation requires `E` to be cheaply cloneable. If `E` doesn't meet
/// the requirement, wrap it with `Rc` or `Arc`.
///
/// The flow is a path on a directed graph where the timestamp of each arc on the path is newer
/// than that of its previous arc.
pub struct FlowTracer<E> {
    sets: HashMap<u64, RefCell<ReachSet<E>>>,
    window_size: u64,
}

impl<E> FlowTracer<E>
where
    E: Clone,
{
    pub fn new(window_size: u64) -> Self {
        Self {
            sets: HashMap::new(),
            window_size,
        }
    }

    /// add an arc connecting two nodes.
    ///
    /// Parameters:
    /// - `src`: the source node id
    /// - `dst`: the destination node id
    /// - `time`: the timestamp of the arc
    /// - `edge_data`: the data associated with the arc
    /// - `is_orphan`: whether the source node is orphan. An orphan is a node that doesn't match
    ///   any signature.
    pub fn add_arc(&mut self, src: u64, dst: u64, time: u64, edge_data: E, is_orphan: bool) {
        if src == dst {
            return;
        }

        if let Entry::Vacant(entry) = self.sets.entry(src) {
            if is_orphan {
                return;
            }
            entry.insert(RefCell::new(ReachSet::new(src, time)));
        }
        self.make_sure_exist(dst, time);

        let src_set = self.sets.get(&src).unwrap();
        let dst_set = self.sets.get(&dst).unwrap();
        dst_set.borrow_mut().connect_from(
            &src_set.borrow(),
            time,
            time.saturating_sub(self.window_size),
            edge_data,
        );
    }

    /// add multiple arcs with the same timestamp, indicating those arcs are added simultaneously.
    ///
    /// Parameters:
    /// - `batch`: an iterator yeilds `(src_id, dst_id, edge_data)`
    /// - `time`: current time, all arcs in the batch has this timestamp
    /// - `is_orphan`: a function returns whether the given node is orphan. An orphan is a node
    ///   that doesn't match any signature and not reachable by any arc in this batch.
    ///
    /// Returns a set of nodes that have new sources reachable to them after adding this batch.
    pub fn add_batch(
        &mut self,
        batch: impl IntoIterator<Item = (u64, u64, E)>,
        time: u64,
        is_orphan: impl Fn(u64) -> bool,
    ) -> HashSet<u64> {
        let mut modified = HashSet::new();
        let time_bound = time.saturating_sub(self.window_size);
        for (src, dst, edge_data) in batch {
            if src == dst {
                continue;
            }

            if let Entry::Vacant(entry) = self.sets.entry(src) {
                if is_orphan(src) {
                    continue;
                }
                entry.insert(RefCell::new(ReachSet::new(src, time)));
            }

            self.make_sure_exist(dst, time);

            let src_set = self.sets.get(&src).unwrap();
            let dst_set = self.sets.get(&dst).unwrap();
            dst_set.borrow_mut().connect_from(
                &src_set.borrow(),
                time,
                time_bound,
                edge_data.clone(),
            );
            for m in &modified {
                if *m == dst {
                    continue;
                }
                let set = self.sets.get(m).unwrap();
                if set.borrow().query(dst) {
                    set.borrow_mut()
                        .apply_new_changes(&dst_set.borrow());
                }
            }
            modified.insert(dst);
        }

        modified
    }

    fn make_sure_exist(&mut self, id: u64, time: u64) {
        self.sets
            .entry(id)
            .or_insert(RefCell::new(ReachSet::new(id, time)));
    }

    /// Returns `true` if there is a flow from `src` to `dst` in the graph.
    pub fn query(&self, src: u64, dst: u64) -> bool {
        if src == dst {
            true
        } else if let Some(set) = self.sets.get(&dst) {
            set.borrow().query(src)
        } else {
            false
        }
    }

    pub fn get_reachset(&self, id: u64) -> Option<Ref<'_, ReachSet<E>>> {
        self.sets.get(&id).map(|set| set.borrow())
    }

    pub fn get_update_time(&self, src: u64, dst: u64) -> Option<u64> {
        let dst_set = self.sets.get(&dst)?;
        dst_set.borrow().get_update_time_of(src)
    }

    pub fn visit_path(&self, src: u64, dst: u64, mut f: impl FnMut(E)) {
        if let Some(dst_set) = self.sets.get(&dst) {
            for edge_data in dst_set.borrow().query_path(src) {
                f(edge_data.clone());
            }
        }
    }

    pub fn del_outdated(&mut self, time_bound: u64) {
        self.sets.retain(|_, set| {
            set.borrow_mut().del_outdated(time_bound);
            !set.borrow().is_empty()
        });
    }
}

/// A wrapper for [FlowTracer] ignoring the associated edge data
pub struct NodeFlowTracer {
    tracer: FlowTracer<()>,
}

impl NodeFlowTracer {
    pub fn new(window_size: u64) -> Self {
        Self {
            tracer: FlowTracer::new(window_size),
        }
    }

    pub fn add_arc(&mut self, src: u64, dst: u64, time: u64) {
        self.tracer.add_arc(src, dst, time, (), false);
    }

    pub fn add_batch(&mut self, batch: impl IntoIterator<Item = (u64, u64)>, time: u64) {
        let iter = batch.into_iter().map(|(u, v)| (u, v, ()));
        self.tracer.add_batch(iter, time, |_| false);
    }

    pub fn query(&self, src: u64, dst: u64) -> bool {
        self.tracer.query(src, dst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut t = NodeFlowTracer::new(10);
        t.add_arc(1, 2, 0);
        t.add_arc(2, 3, 0);
        assert!(t.query(1, 3));
        t.add_arc(4, 2, 0);
        assert!(t.query(4, 2));
        assert!(!t.query(4, 3));
    }

    #[test]
    fn test_windowing() {
        let mut t = NodeFlowTracer::new(10);
        t.add_arc(1, 2, 0);
        t.add_arc(2, 3, 11);
        assert!(!t.query(1, 3));
        t.add_arc(3, 4, 21);
        assert!(t.query(2, 4));
    }

    #[test]
    fn test_cycle() {
        let mut t = NodeFlowTracer::new(10);
        t.add_arc(1, 2, 0);
        t.add_arc(2, 3, 0);
        t.add_arc(3, 1, 0);
        assert!(t.query(1, 3));
        assert!(t.query(2, 1));
        assert!(t.query(3, 1));
        assert!(!t.query(3, 2)); // because 1 -> 2 is before 3 -> 1
    }

    #[test]
    fn test_add_batch() {
        let mut t = NodeFlowTracer::new(10);
        t.add_arc(1, 2, 0);
        t.add_batch([(4, 5), (3, 4), (2, 3), (6, 7), (7, 8)], 1);

        assert!(t.query(1, 5));
        assert!(t.query(6, 8));
    }

    #[test]
    fn add_batch_cycle() {
        let mut t = NodeFlowTracer::new(10);
        t.add_arc(1, 2, 0);
        t.add_batch([(3, 1), (2, 3)], 1);

        assert!(t.query(1, 3));
        assert!(t.query(2, 3));
        assert!(t.query(2, 1));
        assert!(t.query(3, 1));
        assert!(!t.query(3, 2)); // because 1 -> 2 is before 3 -> 1
    }

    #[test]
    fn mix_add_batch_and_add_arc() {
        let mut t = NodeFlowTracer::new(10);
        t.add_arc(1, 2, 0);
        t.add_arc(3, 1, 0);
        t.add_batch([(2, 3), (4, 2), (5, 1)], 1);

        assert!(t.query(1, 3)); // 1, 2, 3
        assert!(!t.query(3, 2)); // because 1 -> 2 is before 3 -> 1
        assert!(t.query(4, 3)); // 4, 2, 3

        t.add_arc(1, 2, 0);
        assert!(t.query(3, 2)); // 3, 1, 2
    }

    #[test]
    fn test_query_path_iter() {
        let mut t = FlowTracer::new(10);
        t.add_arc(1, 2, 0, "1 -> 2", false);
        t.add_arc(2, 3, 0, "2 -> 3", false);

        let set = t.sets.get(&3).unwrap().borrow();
        let mut path_iter = set.query_path(1);
        assert_eq!(path_iter.next(), Some("1 -> 2"));
        assert_eq!(path_iter.next(), Some("2 -> 3"));
        assert_eq!(path_iter.next(), None);
    }
}
