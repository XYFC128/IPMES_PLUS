use std::borrow::Borrow;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use petgraph::algo::tarjan_scc;
use petgraph::graphmap::DiGraphMap;
use petgraph::Direction;
use slab::Slab;

/// A set of nodes. It keeps track of the time of each nodes in the set, and supports querying the
/// updated nodes after union with another set.
#[derive(Clone)]
pub struct ReachSet {
    /// Map node_id -> update_tim
    node_update_time: HashMap<u64, u64>,

    /// The lower bound of the oldest update time in this set.
    oldest_time_hint: u64,

    /// The upper bound of the latest update time in this set.
    latest_time: u64,
}

impl ReachSet {
    pub fn new(id: u64, time: u64, is_match: bool) -> Self {
        let mut node_update_time = HashMap::new();
        if is_match {
            node_update_time.insert(id, time);
        }

        Self {
            node_update_time,
            oldest_time_hint: time,
            latest_time: time,
        }
    }

    /// Merge multiple sets to a single set.
    ///
    /// Parameters:
    /// - `sets`: an iterator of sets
    /// - `time_bound`: during the merging process, it will ignore the nodes that is older than the
    ///   `time_bound`.
    ///
    /// Returns the newly merged set.
    pub fn merge<I, V>(sets: I, time_bound: u64) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Borrow<Self>,
    {
        let mut node_update_time = HashMap::<u64, u64>::new();
        let mut oldest_time_hint = u64::MAX;
        let mut latest_time = 0;
        for set in sets {
            let set = set.borrow();

            if set.latest_time < time_bound {
                continue;
            }

            latest_time = std::cmp::max(latest_time, set.latest_time);

            for (id, time) in &set.node_update_time {
                if *time < time_bound {
                    continue;
                } else if let Some(prev_time) = node_update_time.get_mut(id) {
                    if *prev_time < *time {
                        *prev_time = *time;
                    }
                } else {
                    node_update_time.insert(*id, *time);
                }
                oldest_time_hint = std::cmp::min(oldest_time_hint, *time);
            }
        }

        Self {
            node_update_time,
            oldest_time_hint,
            latest_time,
        }
    }

    /// Update the time of the given `id` if it is in this set. Otherwise, do nothing.
    pub fn refresh_node(&mut self, id: u64, time: u64) {
        if let Some(t) = self.node_update_time.get_mut(&id) {
            *t = time;
            self.latest_time = time;
        }
    }

    /// Union this set with another set. Returns a `Vec` of node ids that is updated in this set.
    ///
    /// The nodes in `other` set which are older than `time_bound` will be ignored.
    pub fn unioned_by(&mut self, other: &Self, time_bound: u64) -> Vec<u64> {
        let mut updated_nodes = Vec::new();

        for (id, time) in &other.node_update_time {
            if self.update_or_insert(*id, *time, time_bound) {
                updated_nodes.push(*id);
            }
        }

        updated_nodes
    }

    /// Calculate the difference of this set and `other`. Returns a `Vec` of node which is
    /// 1. only in this set or
    /// 2. in both sets but the one in this set is newer
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

    /// If this set contains a node of `id` and its time is older than `update_time`, set
    /// its time to `upate_time`. Otherwise, insert a new node into this set with it's time
    /// set to `update_time`.
    ///
    /// Returns `true` if this set is modified. Otherwise, `false` is returned.
    pub fn update_or_insert(&mut self, id: u64, update_time: u64, time_bound: u64) -> bool {
        if update_time < time_bound {
            return false;
        } else if let Some(our_update_time) = self.node_update_time.get_mut(&id) {
            if *our_update_time < update_time {
                *our_update_time = update_time;
                self.latest_time = std::cmp::max(self.latest_time, update_time);
                return true;
            }
        } else {
            self.node_update_time.insert(id, update_time);
            self.latest_time = std::cmp::max(self.latest_time, update_time);
            return true;
        }
        false
    }

    /// Returns the update time of the node `src` in this set.
    /// Returns `None` if `src` is not in this set.
    pub fn get_update_time_of(&self, src: u64) -> Option<u64> {
        self.node_update_time.get(&src).copied()
    }

    /// Returns an iterator of the id of nodes in this set
    pub fn iter(&self) -> impl IntoIterator<Item = u64> + '_ {
        self.node_update_time.keys().copied()
    }

    /// Returns `ture` if this set is empty
    pub fn is_empty(&self) -> bool {
        self.node_update_time.is_empty()
    }

    /// Remove nodes that is older than the `time_bound`.
    pub fn del_outdated(&mut self, time_bound: u64) {
        if self.oldest_time_hint >= time_bound {
            return;
        } else if self.latest_time < time_bound {
            self.node_update_time.clear();
            return;
        }

        let mut oldest_time = u64::MAX;
        self.node_update_time.retain(|_, t| {
            if *t >= time_bound {
                oldest_time = std::cmp::min(oldest_time, *t);
                true
            } else {
                false
            }
        });
        self.oldest_time_hint = oldest_time;
    }
}

/// Incrementally trace the flows in a streaming graph.
///
/// A flow is a path on a directed graph where the timestamp of each arc on the path is newer
/// than that of its previous arc.
pub struct FlowTracer {
    /// `reach_sets.get(&src)` contains the set of nodes that can reach `src`
    /// (rather than reachable from `src`). 
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
    /// - `is_match`: a function returns whether the given node matches any signature
    ///
    /// Returns the updated nodes of dst set.
    pub fn add_arc(
        &mut self,
        src: u64,
        dst: u64,
        time: u64,
        is_match: impl Fn(u64) -> bool,
    ) -> Vec<u64> {
        if src == dst {
            return vec![];
        }

        let src_match = is_match(src);
        let dst_match = is_match(dst);

        let time_bound = time.saturating_sub(self.window_size);
        let mut dst_set = self
            .reach_sets
            .remove(&dst)
            .unwrap_or(ReachSet::new(dst, time, dst_match));
        dst_set.refresh_node(dst, time);
        
        let diff = if let Some(src_set) = self.reach_sets.get_mut(&src) {
            // If `src` is already reachable from other nodes, then after adding 
            // this arc, all those nodes can now reach `dst`.
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
    /// Returns a mapping from a node id to the a of its new reachable sources.
    pub fn add_batch(
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
                        let updates = updated_nodes.entry(*id).or_default();
                        if let Some(old_set) = self.reach_sets.get(id) {
                            let diff = new_sets[new_key].difference(old_set);
                            updates.extend(diff);
                        } else {
                            let diff = new_sets[new_key].iter();
                            updates.extend(diff);
                        }
                        updates.remove(id); // avoid root appearing in updated_nodes
                        id2key.insert(*id, new_key);
                    }
                } else {
                    let mut set = self.reach_sets.remove(first).unwrap_or(ReachSet::new(
                        *first,
                        time,
                        is_match(*first),
                    ));
                    set.refresh_node(*first, time); // refresh the root to prevent it from
                                                    // appearing in updated_nodes
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

                    // prevent unioning with itself or unioning the same pair of sets multiple
                    // times
                    let union_same_set = *src_key == *dst_key;
                    let already_unioned = last_union[*dst_key] == *src_key as i32;
                    if union_same_set || already_unioned {
                        continue;
                    }

                    last_union[*dst_key] = *src_key as i32;
                    if let Some((src_set, dst_set)) = new_sets.get2_mut(*src_key, *dst_key) {
                        let diff = dst_set.unioned_by(src_set, time_bound);
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

    /// Get the time when the flow from `src` to `dst` is started. Returns `None` if there is no
    /// such flow.
    pub fn get_updated_time(&self, src: u64, dst: u64) -> Option<u64> {
        self.reach_sets
            .get(&dst)
            .and_then(|s| s.get_update_time_of(src))
    }

    /// Remove the oudated internal states that is older than `time_bound`
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

    fn set_eq<A, B, V>(a: A, b: B) -> bool
    where
        A: IntoIterator<Item = V>,
        B: IntoIterator<Item = V>,
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
        I: IntoIterator<Item = (K, V)>,
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
        I: IntoIterator<Item = V>,
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
        let ans = map([(1, set([2, 3])), (3, set([1, 2]))]);
        assert_eq!(res, ans);
    }

    #[test]
    fn test_scc() {
        let mut t = FlowTracer::new(10);
        let is_match = |_| true;
        assert!(set_eq(t.add_arc(1, 2, 0, is_match), [1]));
        assert_eq!(
            t.add_batch([(2, 3), (3, 4), (4, 2), (3, 5)], 1, is_match),
            map([
                (2, set([3, 4])),
                (3, set([1, 2, 4])),
                (4, set([1, 2, 3])),
                (5, set([1, 2, 3, 4])),
            ])
        );
        assert!(set_eq(t.add_arc(6, 2, 2, is_match), [6]));
        assert!(set_eq(t.add_arc(3, 7, 2, is_match), [1, 2, 3, 4]));
    }

    #[test]
    fn mix_add_batch_and_add_arc() {
        let mut t = FlowTracer::new(10);
        let is_match = |_| true;
        assert!(set_eq(t.add_arc(1, 2, 0, is_match), [1]));
        assert!(set_eq(t.add_arc(3, 1, 1, is_match), [3]));
        assert_eq!(
            t.add_batch([(2, 3), (4, 2), (5, 1)], 2, is_match),
            map([(1, set([5])), (2, set([4])), (3, set([1, 2, 4])),])
        );
        assert!(set_eq(t.add_arc(1, 2, 3, is_match), [1, 3, 5]));
    }
}
