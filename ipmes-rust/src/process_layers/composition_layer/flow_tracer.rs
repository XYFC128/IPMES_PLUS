use std::borrow::Borrow;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use petgraph::algo::tarjan_scc;
use petgraph::graphmap::DiGraphMap;
use petgraph::Direction;
use slab::Slab;

/// It maintains the nodes that can reach `v` for some node `v` in the graph. `E` is the type of
/// associated data on the arcs.
#[derive(Clone)]
pub struct ReachSet {
    /// Map node_id -> update_tim
    node_update_time: HashMap<u64, u64>,
}

impl ReachSet {
    pub fn new(id: u64, time: u64, is_match: bool) -> Self {
        let mut node_update_time = HashMap::new();
        if is_match {
            node_update_time.insert(id, time);
        }

        Self { node_update_time }
    }

    /// Merge multiple sets to a single set.
    pub fn merge<I, V>(sets: I, time_bound: u64) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Borrow<Self>,
    {
        let mut node_update_time = HashMap::<u64, u64>::new();
        for set in sets {
            for (id, time) in &set.borrow().node_update_time {
                if *time < time_bound {
                    continue;
                } else if let Some(prev_time) = node_update_time.get_mut(id) {
                    if *prev_time < *time {
                        *prev_time = *time;
                    }
                } else {
                    node_update_time.insert(*id, *time);
                }
            }
        }

        Self { node_update_time }
    }

    /// Update the time of the given `id` if it is in this set. Otherwise, do nothing.
    pub fn refresh_node(&mut self, id: u64, time: u64) {
        self.node_update_time.entry(id).and_modify(|t| *t = time);
    }

    /// The root node `u` of `other` can connect to the root node `v` of this set. This operation
    /// correspond to set union operation. Given that `u` can reach `v`, all nodes that can reach
    /// `u` can reach `v` too, thus union the ReachSet of `u` to `v`.
    ///
    /// The union operation will ignore the nodes in `other` that are updated before `timebound`.
    ///
    /// The `edge_data` is the data associated with the arc `u -> v`.
    pub fn unioned_by(&mut self, other: &Self, time_bound: u64) -> Vec<u64> {
        let mut updated_nodes = Vec::new();

        for (id, time) in &other.node_update_time {
            if self.update_or_insert(*id, *time, time_bound) {
                updated_nodes.push(*id);
            }
        }

        updated_nodes
    }

    pub fn difference(&self, other: &Self) -> Vec<u64> {
        let mut diff = vec![];
        for (id, time) in &self.node_update_time {
            if other
                .node_update_time
                .get(id)
                .is_some_and(|other_time| *other_time >= *time)
            {
                continue;
            }
            diff.push(*id);
        }
        diff
    }

    pub fn update_or_insert(&mut self, id: u64, update_time: u64, time_bound: u64) -> bool {
        if update_time < time_bound {
            return false;
        } else if let Some(our_update_time) = self.node_update_time.get_mut(&id) {
            if *our_update_time < update_time {
                *our_update_time = update_time;
                return true;
            }
        } else {
            self.node_update_time.insert(id, update_time);
            return true;
        }
        false
    }

    /// Returns the update time of the node `src` in this set.
    /// Returns `None` if `src` is not in this set.
    pub fn get_update_time_of(&self, src: u64) -> Option<u64> {
        self.node_update_time.get(&src).copied()
    }

    pub fn iter(&self) -> impl IntoIterator<Item = u64> + '_ {
        self.node_update_time.keys().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.node_update_time.is_empty()
    }

    pub fn del_outdated(&mut self, time_bound: u64) {
        self.node_update_time.retain(|_, t| *t >= time_bound);
    }
}

/// Incrementally trace the flow. [`E`] is the type of the data associated with the arcs in the
/// graph. The current implementation requires `E` to be cheaply cloneable. If `E` doesn't meet
/// the requirement, wrap it with `Rc` or `Arc`.
///
/// The flow is a path on a directed graph where the timestamp of each arc on the path is newer
/// than that of its previous arc.
pub struct FlowTracer {
    reach_sets: HashMap<u64, ReachSet>,
    window_size: u64,
}

impl FlowTracer {
    pub fn new(window_size: u64) -> Self {
        Self {
            reach_sets: HashMap::new(),
            window_size,
        }
    }

    /// add an arc connecting two nodes.
    ///
    /// Parameters:
    /// - `src`: the source node id
    /// - `dst`: the destination node id
    /// - `time`: the timestamp of the arc
    /// - `is_matach`: a function returns whether the given node matches any signature
    ///
    /// Returns the updated nodes of dst set.
    pub fn add_arc(&mut self, src: u64, dst: u64, time: u64, is_match: impl Fn(u64) -> bool) -> Vec<u64> {
        if src == dst {
            return vec![];
        }

        let src_match = is_match(src);
        let dst_match = is_match(dst);

        let time_bound = time.saturating_sub(self.window_size);
        let mut dst_set = self.reach_sets.remove(&dst).unwrap_or(ReachSet::new(dst, time, dst_match));
        dst_set.refresh_node(dst, time);

        let diff = if let Some(src_set) = self.reach_sets.get_mut(&src) {
            src_set.refresh_node(src, time);
            dst_set.unioned_by(src_set, time_bound)
        } else if src_match {
            dst_set.update_or_insert(src, time, time_bound);
            vec![src]
        } else {
            vec![]
        };
        
        self.reach_sets.insert(dst, dst_set);
        diff
    }

    /// add multiple arcs with the same timestamp, indicating those arcs are added simultaneously.
    ///
    /// Parameters:
    /// - `batch`: an iterator yeilds `(src_id, dst_id)`
    /// - `time`: current time, all arcs in the batch has this timestamp
    /// - `is_matach`: a function returns whether the given node matches any signature
    ///
    /// Returns a mapping from a node id to the set of its new reachable sources.
    fn add_batch(
        &mut self,
        batch: impl IntoIterator<Item = (u64, u64)>,
        time: u64,
        is_match: impl Fn(u64) -> bool,
    ) -> HashMap<u64, HashSet<u64>> {
        let time_bound = time.saturating_sub(self.window_size);
        let batch_graph = DiGraphMap::<u64, ()>::from_edges(batch);

        let num_node = batch_graph.node_count();
        let mut new_sets = Slab::<ReachSet>::with_capacity(num_node);
        let mut id2key = HashMap::<u64, usize>::with_capacity(num_node);
        let mut updated_nodes = HashMap::<u64, HashSet<u64>>::with_capacity(num_node);

        // Contract scc
        let sccs = tarjan_scc(&batch_graph);
        for scc in &sccs {
            if let Some((first, rest)) = scc.split_first() {
                if !rest.is_empty() {
                    let set_iter = scc.iter().filter_map(|id| self.reach_sets.get(id));
                    let new_key = new_sets.insert(ReachSet::merge(set_iter, time_bound));
                    for id in scc {
                        if is_match(*id) {
                            new_sets[new_key].update_or_insert(*id, time, 0);
                        }
                    }
                    for id in scc {
                        if let Some(old_set) = self.reach_sets.get(id) {
                            let diff = new_sets[new_key].difference(old_set);
                            updated_nodes.entry(*id).or_default().extend(diff);
                        } else {
                            let diff = new_sets[new_key].iter();
                            updated_nodes.entry(*id).or_default().extend(diff);
                        }
                        id2key.insert(*id, new_key);
                    }
                } else {
                    let mut set = self.reach_sets.remove(first).unwrap_or(ReachSet::new(
                        *first,
                        time,
                        is_match(*first),
                    ));
                    set.refresh_node(*first, time);
                    let new_key = new_sets.insert(set);
                    id2key.insert(*first, new_key);
                }
            }
        }

        // Union along DAG. As sccs are returned in reversed topoligical order, we traverse it
        // reversely to get the correct order.
        let mut last_union = vec![-1; new_sets.capacity()];
        for scc in sccs.iter().rev() {
            for src_id in scc {
                let src_key = id2key.get(src_id).unwrap();
                for dst_id in batch_graph.neighbors_directed(*src_id, Direction::Outgoing) {
                    let dst_key = id2key.get(&dst_id).unwrap();
                    // this will not be true if `src_key` == `dst_key`
                    if let Some((src_set, dst_set)) = new_sets.get2_mut(*src_key, *dst_key) {
                        // prevent unioning the same pair of sets multiple times
                        if last_union[*dst_key] == *src_key as i32 {
                            continue;
                        } else {
                            last_union[*dst_key] = *src_key as i32;
                        }

                        let diff = dst_set.unioned_by(src_set, time_bound);
                        dst_set.refresh_node(dst_id, time);
                        updated_nodes.entry(dst_id).or_default().extend(diff);
                    }
                }
            }
        }

        // Put the new sets back. 
        // Clone the ReachSet to the nodes in the same SCC. This is required as they must be
        // updated seperately later on.
        for scc in &sccs {
            if let Some((first, rest)) = scc.split_first() {
                let key = id2key.get(first).unwrap();
                let set = new_sets.remove(*key);
                for other_id in rest {
                    let new_set = set.clone();
                    self.reach_sets.insert(*other_id, new_set);
                }
                self.reach_sets.insert(*first, set);
            }
        }

        updated_nodes
    }

    pub fn del_outdated(&mut self, time_bound: u64) {
        self.reach_sets.retain(|_, set| {
            set.del_outdated(time_bound);
            !set.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::sorted;

    #[test]
    fn test_petgraph() {
        use petgraph::algo::tarjan_scc;
        use petgraph::graphmap::DiGraphMap;

        let g = DiGraphMap::<i32, ()>::from_edges([(1, 2), (2, 3), (3, 4), (4, 2)]);
        let scc = tarjan_scc(&g);
        println!("{:#?}", scc);
    }

    fn set_eq<A, B, V>(a: A, b: B) -> bool
    where
        A: IntoIterator<Item=V>,
        B: IntoIterator<Item=V>,
        V: Ord,
    {
        sorted(a).eq(sorted(b))
    }

    #[test]
    fn test_basic() {
        let mut t = FlowTracer::new(10);
        let is_match = |_| true;
        assert!(set_eq(t.add_arc(1, 2, 0, is_match), [1]));
        assert!(set_eq(t.add_arc(2, 3, 1, is_match), [1, 2]));
        assert!(set_eq(t.add_arc(4, 2, 2, is_match), [4]));
    }

    #[test]
    fn test_windowing() {
        let mut t = FlowTracer::new(10);
        let is_match = |_| true;
        assert!(set_eq(t.add_arc(1, 2, 0, is_match), [1]));
        assert!(set_eq(t.add_arc(2, 3, 11, is_match), [2]));
        assert!(set_eq(t.add_arc(3, 4, 21, is_match), [2, 3]));
    }

    #[test]
    fn test_cycle() {
        let mut t = FlowTracer::new(10);
        let is_match = |_| true;
        assert!(set_eq(t.add_arc(1, 2, 0, is_match), [1]));
        assert!(set_eq(t.add_arc(2, 3, 1, is_match), [1, 2]));
        assert!(set_eq(t.add_arc(3, 1, 2, is_match), [2, 3]));
    }

    fn map<I, K, V>(iter: I) -> HashMap<K, V>
    where 
        I: IntoIterator<Item=(K, V)>,
        K: std::hash::Hash + Eq,
    {
        let mut res = HashMap::new();
        for (k, v) in iter {
            res.insert(k, v);
        }
        res
    }

    fn set<I, V>(iter: I) -> HashSet<V>
    where 
        I: IntoIterator<Item=V>,
        V: std::hash::Hash + Eq,
    {
        let mut res = HashSet::new();
        for v in iter {
            res.insert(v);
        }
        res
    }

    #[test]
    fn test_add_batch() {
        let mut t = FlowTracer::new(10);
        let is_match = |_| true;
        assert!(set_eq(t.add_arc(1, 2, 0, is_match), [1]));
        let res = t.add_batch([(4, 5), (3, 4), (2, 3), (6, 7), (7, 8)], 1, is_match);
        let ans = map([
            (3, set([1, 2])),
            (4, set([1, 2, 3])),
            (5, set([1, 2, 3, 4])),
            (7, set([6])),
            (8, set([6, 7])),
        ]);
        assert_eq!(res, ans);
    }

    #[test]
    fn add_batch_cycle() {
        let mut t = FlowTracer::new(10);
        let is_match = |_| true;
        assert!(set_eq(t.add_arc(1, 2, 0, is_match), [1]));
        let res = t.add_batch([(3, 1), (2, 3)], 1, is_match);
        let ans = map([
            (1, set([2, 3])),
            (3, set([1, 2])),
        ]);
        assert_eq!(res, ans);
    }

    #[test]
    fn mix_add_batch_and_add_arc() {
        let mut t = FlowTracer::new(10);
        let is_match = |_| true;
        assert!(set_eq(t.add_arc(1, 2, 0, is_match), [1]));
        assert!(set_eq(t.add_arc(3, 1, 1, is_match), [3]));
        assert_eq!(
            t.add_batch([(2, 3), (4, 2), (5, 1)], 2, is_match),
            map([
                (1, set([5])),
                (2, set([4])),
                (3, set([1, 2, 4])),
            ])
        );
        assert!(set_eq(t.add_arc(1, 2, 3, is_match), [1, 3, 5]));
    }
}
