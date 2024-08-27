use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
};

#[derive(Clone)]
struct Node<E> {
    id: u64,
    parent: u64,
    update_time: u64,
    edge_data: E,
}

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

    pub fn apply_new_changes(&mut self, other: &Self, time_bound: u64) {
        if let Some(other_root) = self.nodes.get(&other.root_id) {
            if other_root.update_time != other.update_time {
                // the new changes is to late to be applied
                return;
            }
        } else {
            return; // `other.root` is not in this set
        }

        // NOTE: No need to clear updated_nodes here as the update_time is not advanced

        for node in other.nodes.values() {
            if node.update_time < time_bound || node.id == self.root_id {
                continue;
            }

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

    pub fn query(&self, id: u64) -> bool {
        id == self.root_id || self.nodes.contains_key(&id)
    }

    pub fn get_id(&self) -> u64 {
        self.root_id
    }

    pub fn get_update_time(&self) -> u64 {
        self.update_time
    }

    pub fn get_update_time_of(&self, src: u64) -> Option<u64> {
        if src == self.root_id {
            Some(self.update_time)
        } else {
            self.nodes.get(&src).map(|node| node.update_time)
        }
    }

    pub fn get_updated_nodes(&self) -> &[u64] {
        &self.updated_nodes
    }

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
/// graph.
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

    pub fn add_arc(&mut self, src: u64, dst: u64, time: u64, edge_data: E) {
        if src == dst {
            return;
        }

        self.make_sure_exist(src, time);
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

    pub fn add_batch(
        &mut self,
        batch: impl IntoIterator<Item = (u64, u64, E)>,
        time: u64,
    ) -> HashSet<u64> {
        let mut modified = HashSet::new();
        let time_bound = time.saturating_sub(self.window_size);
        for (src, dst, edge_data) in batch {
            if src == dst {
                continue;
            }
            self.make_sure_exist(src, time);
            self.make_sure_exist(dst, time);

            let src_set = self.sets.get(&src).unwrap();
            let dst_set = self.sets.get(&dst).unwrap();
            dst_set.borrow_mut().connect_from(
                &src_set.borrow(),
                time,
                time_bound,
                edge_data.clone(),
            );
            modified.insert(dst);
            for m in &modified {
                if *m == dst {
                    continue;
                }
                let set = self.sets.get(m).unwrap();
                if set.borrow().query(dst) {
                    set.borrow_mut()
                        .apply_new_changes(&dst_set.borrow(), time_bound);
                }
            }
        }

        modified
    }

    fn make_sure_exist(&mut self, id: u64, time: u64) {
        self.sets
            .entry(id)
            .or_insert(RefCell::new(ReachSet::new(id, time)));
    }

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

    pub fn visit_updated_nodes(&self, dst: u64, mut f: impl FnMut(u64)) {
        if let Some(dst_set) = self.sets.get(&dst) {
            for node in dst_set.borrow().get_updated_nodes() {
                f(*node);
            }
        }
    }

    pub fn visit_path(&self, src: u64, dst: u64, mut f: impl FnMut(E)) {
        if let Some(dst_set) = self.sets.get(&dst) {
            for edge_data in dst_set.borrow().query_path(src) {
                f(edge_data.clone());
            }
        }
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
        self.tracer.add_arc(src, dst, time, ());
    }

    pub fn add_batch(&mut self, batch: impl IntoIterator<Item = (u64, u64)>, time: u64) {
        let iter = batch.into_iter().map(|(u, v)| (u, v, ()));
        self.tracer.add_batch(iter, time);
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
        t.add_arc(1, 2, 0, "1 -> 2");
        t.add_arc(2, 3, 0, "2 -> 3");

        let set = t.sets.get(&3).unwrap().borrow();
        let mut path_iter = set.query_path(1);
        assert_eq!(path_iter.next(), Some("1 -> 2"));
        assert_eq!(path_iter.next(), Some("2 -> 3"));
        assert_eq!(path_iter.next(), None);
    }
}
