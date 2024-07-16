use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone)]
struct Node {
    id: u64,
    parent: u64,
    update_time: u64,
}

struct ReachSet {
    nodes: HashMap<u64, Node>,
    root: Node,
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
        }
    }

    pub fn merge_with(&mut self, other: &Self, cur_time: u64, time_bound: u64) {
        self.root.update_time = cur_time;
        for node in other.nodes.values() {
            if node.update_time < time_bound || node.id == self.root.id {
                continue;
            }

            if let Some(our_node) = self.nodes.get_mut(&node.id) {
                if our_node.update_time < node.update_time {
                    our_node.parent = node.parent;
                    our_node.update_time = node.update_time;
                }
            } else {
                self.nodes.insert(node.id, node.clone());
            }
        }

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
    }

    pub fn query(&self, id: u64) -> bool {
        id == self.root.id || self.nodes.contains_key(&id)
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
        dst_set.borrow_mut().merge_with(
            &src_set.borrow(),
            time,
            time.saturating_sub(self.window_size),
        );
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
}
