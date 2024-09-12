use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use std::cell::RefCell;
use std::rc::Rc;

type NodeRef = Rc<RefCell<Node>>;

struct Node {
    id: u64,
    update_time: u64,
    child: Vec<NodeRef>,
}

impl Node {
    fn child_newer_than(&self, time: u64) -> &[NodeRef] {
        let i = self
            .child
            .partition_point(|node_ref| node_ref.borrow().update_time < time);
        &self.child[i..]
    }
}

/// It maintains the nodes that can reach `v` for some node `v` in the graph. `E` is the type of
/// associated data on the arcs.
pub struct ReachSet {
    nodes: HashMap<u64, NodeRef>,
    root: Node,
    match_signature: bool,

    /// the updated node ids of the set since last time update_time is advanced
    updated_nodes: HashSet<u64>,
}

impl ReachSet {
    pub fn new(root_id: u64, time: u64, match_signature: bool) -> Self {
        Self {
            nodes: HashMap::new(),
            root: Node {
                id: root_id,
                update_time: time,
                child: vec![],
            },
            match_signature,
            updated_nodes: HashSet::new(),
        }
    }

    /// The root node `u` of `other` can connect to the root node `v` of this set. This operation
    /// correspond to set union operation. Given that `u` can reach `v`, all nodes that can reach
    /// `u` can reach `v` too, thus union the ReachSet of `u` to `v`.
    ///
    /// The union operation will ignore the nodes in `other` that are updated before `timebound`.
    pub fn connect_from(&mut self, other: &Self, cur_time: u64, window_bound: u64) {
        if cur_time > self.root.update_time {
            self.root.update_time = cur_time;
            self.updated_nodes.clear();
        }

        if other.match_signature {
            if let Some(new_nd) = self.update_subtree(&other.root, window_bound) {
                self.root.child.push(new_nd);
            }
        } else {
            self.merge_child(&other.root, window_bound);
        }

        self.root.update_time = cur_time;
    }

    /// Merges the children of other_root into our root's children. As the children array is sorted
    /// incrementally by their update time, this function guarantees that the order of the merged
    /// children array follows this constrain.
    fn merge_child(&mut self, other_root: &Node, window_bound: u64) {
        let other_child = other_root.child_newer_than(window_bound);
        if other_child.is_empty() {
            return;
        }

        let mut new_child = Vec::with_capacity(self.root.child.len() + other_child.len());
        let mut it_our = std::mem::take(&mut self.root.child).into_iter().peekable();

        for ch_other in other_child {
            if let Some(new_nd) = self.update_subtree(&ch_other.borrow(), window_bound) {
                let new_time = new_nd.borrow().update_time;

                // insert all our child older than new_nd
                while let Some(nd_our) = it_our.next_if(|nd| nd.borrow().update_time <= new_time) {
                    new_child.push(nd_our);
                }

                new_child.push(new_nd);
            }
        }

        new_child.extend(it_our);

        self.root.child = new_child;
    }

    fn update_subtree(&mut self, root_other: &Node, window_bound: u64) -> Option<NodeRef> {
        if root_other.id == self.root.id {
            return None;
        }

        let child_time_bound = self.get_update_time(root_other.id).unwrap_or(window_bound);
        if root_other.update_time < child_time_bound {
            return None;
        }

        let mut new_child = vec![];
        for child_ref in root_other.child_newer_than(child_time_bound) {
            let child_other = child_ref.borrow();
            if let Some(new_nd) = self.update_subtree(&child_other, window_bound) {
                new_child.push(new_nd);
            }
        }

        if new_child.is_empty() {
            return None;
        }

        self.updated_nodes.insert(root_other.id);

        if let Some(node_our_ref) = self.nodes.get(&root_other.id) {
            let update_time_our = node_our_ref.borrow().update_time;
            if update_time_our == root_other.update_time {
                node_our_ref.borrow_mut().child.extend(new_child);
                return None;
            }
        }

        Some(self.new_node(root_other.id, root_other.update_time, new_child))
    }

    fn new_node(&mut self, id: u64, update_time: u64, child: Vec<NodeRef>) -> NodeRef {
        let new_nd = Rc::new(RefCell::new(Node {
            id,
            update_time,
            child,
        }));

        self.nodes.insert(id, new_nd.clone());

        new_nd
    }

    fn get_update_time(&self, id: u64) -> Option<u64> {
        self.nodes
            .get(&id)
            .map(|node_ref| node_ref.borrow().update_time)
    }

    /// Returns the node that are updated since last update of update time.
    pub fn get_updated_nodes(&self) -> impl Iterator<Item=u64> + '_ {
        self.updated_nodes.iter().copied()
    }

    pub fn del_outdated(&mut self, time_bound: u64) {
        if self.root.update_time < time_bound {
            self.nodes.clear();
            self.root.child.clear();
        }
        self.nodes.retain(|_, nd| nd.borrow().update_time >= time_bound);
        Self::subtree_windowing(&mut self.root, time_bound);
    }

    fn subtree_windowing(root: &mut Node, time_bound: u64) {
        let too_old = |nd: &NodeRef| nd.borrow().update_time < time_bound;
        let i = root.child.partition_point(too_old);
        root.child.drain(..i);

        for ch in &root.child {
            Self::subtree_windowing(&mut ch.borrow_mut(), time_bound);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {

    }
}
