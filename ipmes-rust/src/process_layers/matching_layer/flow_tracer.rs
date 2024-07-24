use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct Node {
    id: u64,
    parent: u64,
    update_time: u64,
}

struct ReachSet {
    nodes: HashMap<u64, Node>,
    root: Node,

    /// the updated node ids of the set since last modified
    updated_nodes: Vec<u64>,
}

impl ReachSet {
    pub fn new(id: u64, time: u64) -> Self {
        Self {
            nodes: HashMap::new(),
            root: Node {
                id,
                parent: id,
                update_time: time,
            },
            updated_nodes: Vec::new(),
        }
    }

    pub fn connect_from(&mut self, other: &Self, cur_time: u64, time_bound: u64) {
        if other.root.id == self.root.id {
            return;
        }

        self.root.update_time = cur_time;
        self.updated_nodes.clear();

        if let Some(other_root) = self.nodes.get_mut(&other.root.id) {
            other_root.parent = self.root.id;
            other_root.update_time = cur_time;
        } else {
            self.nodes.insert(
                other.root.id,
                Node {
                    id: other.root.id,
                    parent: self.root.id,
                    update_time: cur_time,
                },
            );
        }

        if other.get_update_time() < time_bound {
            return;
        }

        for node in other.nodes.values() {
            if node.update_time < time_bound || node.id == self.root.id {
                continue;
            }

            self.update_or_insert(node);
        }
    }

    pub fn apply_new_changes(&mut self, other: &Self, time_bound: u64) {
        if let Some(other_root) = self.nodes.get(&other.root.id) {
            if other_root.update_time != other.root.update_time {
                // the new changes is to late to be applied
                return;
            }
        } else {
            return; // `other.root` is not in this set
        }

        self.updated_nodes.clear();

        for node in other.nodes.values() {
            if node.update_time < time_bound || node.id == self.root.id {
                continue;
            }

            self.update_or_insert(node);
        }
    }

    fn update_or_insert(&mut self, node: &Node) {
        if let Some(our_node) = self.nodes.get_mut(&node.id) {
            if our_node.update_time < node.update_time {
                our_node.parent = node.parent;
                our_node.update_time = node.update_time;
            }
        } else {
            self.nodes.insert(node.id, node.clone());
        }
        self.updated_nodes.push(node.id);
    }

    pub fn query(&self, id: u64) -> bool {
        id == self.root.id || self.nodes.contains_key(&id)
    }

    pub fn get_id(&self) -> u64 {
        self.root.id
    }

    pub fn get_update_time(&self) -> u64 {
        self.root.update_time
    }

    pub fn visit_updated_nodes(&self, mut f: impl FnMut(u64)) {
        for id in &self.updated_nodes {
            f(*id);
        }
    }

    pub fn query_path(&self, src: u64, path_buf: &mut Vec<u64>) {
        if !self.query(src) {
            return;
        }

        path_buf.clear();
        let mut cur = src;
        while cur != self.root.id {
            let cur_node = self.nodes.get(&cur).expect("tree structer is well maintained");
            path_buf.push(cur);
            cur = cur_node.parent;
        }
        path_buf.push(self.root.id);
    }

    pub fn query_path_iter(&self, src: u64) -> PathIter {
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

pub struct PathIter<'a> {
    reach_set: &'a ReachSet,
    cur_node: Option<&'a Node>,
}

impl<'a> Iterator for PathIter<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur_node) = self.cur_node {
            let cur_id = cur_node.id;

            if cur_id == self.reach_set.root.id {
                self.cur_node = None;
            } else if cur_node.parent == self.reach_set.root.id {
                self.cur_node = Some(&self.reach_set.root);
            } else {
                self.cur_node = self.reach_set.nodes.get(&cur_node.parent);
            }

            Some(cur_id)
        } else {
            None
        }
    }
}

pub struct FlowTracer {
    sets: HashMap<u64, RefCell<ReachSet>>,
    window_size: u64,
}

impl FlowTracer {
    pub fn new(window_size: u64) -> Self {
        Self {
            sets: HashMap::new(),
            window_size,
        }
    }

    pub fn add_arc(&mut self, src: u64, dst: u64, time: u64) {
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
        );
    }

    pub fn add_batch(&mut self, batch: impl IntoIterator<Item = (u64, u64)>, time: u64) {
        let mut modified = HashSet::new();
        let time_bound = time.saturating_sub(self.window_size);
        for (src, dst) in batch {
            if src == dst {
                continue;
            }
            self.make_sure_exist(src, time);
            self.make_sure_exist(dst, time);

            let src_set = self.sets.get(&src).unwrap();
            let dst_set = self.sets.get(&dst).unwrap();
            dst_set
                .borrow_mut()
                .connect_from(&src_set.borrow(), time, time_bound);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut t = FlowTracer::new(10);
        t.add_arc(1, 2, 0);
        t.add_arc(2, 3, 0);
        assert!(t.query(1, 3));
        t.add_arc(4, 2, 0);
        assert!(t.query(4, 2));
        assert!(!t.query(4, 3));
    }

    #[test]
    fn test_windowing() {
        let mut t = FlowTracer::new(10);
        t.add_arc(1, 2, 0);
        t.add_arc(2, 3, 11);
        assert!(!t.query(1, 3));
        t.add_arc(3, 4, 21);
        assert!(t.query(2, 4));
    }

    #[test]
    fn test_cycle() {
        let mut t = FlowTracer::new(10);
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
        let mut t = FlowTracer::new(10);
        t.add_arc(1, 2, 0);
        t.add_batch([(4, 5), (3, 4), (2, 3), (6, 7), (7, 8)], 1);

        assert!(t.query(1, 5));
        assert!(t.query(6, 8));
    }

    #[test]
    fn add_batch_cycle() {
        let mut t = FlowTracer::new(10);
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
        let mut t = FlowTracer::new(10);
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
        t.add_arc(1, 2, 0);
        t.add_arc(2, 3, 0);
        
        let set = t.sets.get(&3).unwrap().borrow();
        let mut path_iter = set.query_path_iter(1);
        assert_eq!(path_iter.next(), Some(1));
        assert_eq!(path_iter.next(), Some(2));
        assert_eq!(path_iter.next(), Some(3));
        assert_eq!(path_iter.next(), None);
    }
}
