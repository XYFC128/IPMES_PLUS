use std::collections::HashSet;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

/// Transitive closure **T** of a directed graph **G** shares the same vertices as **G** but as
/// an arc from **u** to **v** if and only if there is a path from **u** to **v** in **G**. This
/// structure maintains the reachibility information incrementally and supports querying
/// reachibility of any pair of nodes in **G**.
pub struct TransitiveClosure {
    nodes: HashSet<u64>,
    index: Index,
}

impl TransitiveClosure {
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            index: Index::new(),
        }
    }

    /// Add an arc from `i` to `j`
    pub fn add(&mut self, i: u64, j: u64) {
        if self.is_reachable(i, j) {
            return;
        }

        if self.nodes.insert(i) {
            self.index.add_node(i);
        }
        if self.nodes.insert(j) {
            self.index.add_node(j);
        }

        for x in &self.nodes {
            if self.is_reachable(*x, i) && !self.is_reachable(*x, j) {
                self.index.meld(*x, j, i, j);
            }
        }
    }

    /// Returns true if there is a path from `i` to `j`
    pub fn is_reachable(&self, i: u64, j: u64) -> bool {
        self.index.query(i, j)
    }
}

struct NodeData {
    id: u64,
    children: Vec<Rc<RefCell<NodeData>>>,
}

struct Index {
    index: HashMap<(u64, u64), Rc<RefCell<NodeData>>>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: u64) {
        self.index.insert(
            (id, id),
            Rc::new(RefCell::new(NodeData {
                id,
                children: Vec::new(),
            })),
        );
    }

    /// Meld two spanning trees rooted at `x` and `j` with the arc from `u` to `v` where `u` is
    /// reachable from `x` but `v` is not.
    pub fn meld(&mut self, x: u64, j: u64, u: u64, v: u64) {
        // create a new node pointed by index (x, v)
        let new_node = Rc::new(RefCell::new(NodeData {
            id: v,
            children: Vec::new(),
        }));
        self.index.insert((x, v), Rc::clone(&new_node));

        // insert the new node in desc(x) as a child of u
        let u_x = self.index.get(&(x, u)).unwrap();
        u_x.borrow_mut().children.push(new_node);

        let v_j = Rc::clone(self.index.get(&(j, v)).unwrap());
        for child in &v_j.borrow().children {
            let w = child.borrow().id;
            if !self.index.contains_key(&(x, w)) {
                self.meld(x, j, v, w);
            }
        }
    }

    pub fn query(&self, i: u64, j: u64) -> bool {
        self.index.contains_key(&(i, j))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        let mut tc = TransitiveClosure::new();
        tc.add(2, 3);
        tc.add(1, 2);
        assert!(tc.is_reachable(1, 3));
        tc.add(4, 5);
        assert!(!tc.is_reachable(1, 5));
    }

    #[test]
    fn test_cycle() {
        let mut tc = TransitiveClosure::new();
        tc.add(1, 2);
        tc.add(2, 3);
        tc.add(3, 4);
        tc.add(3, 1);
        assert!(tc.is_reachable(1, 4));
        assert!(tc.is_reachable(3, 1));
        assert!(tc.is_reachable(3, 2));
        assert!(!tc.is_reachable(4, 1));
    }
}
