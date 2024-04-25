use itertools::enumerate;
use itertools::Itertools;
use log::debug;
use petgraph::algo;
use petgraph::dot::Dot;
use petgraph::graph::EdgeIndex;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Graph;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

use crate::input_event::InputEvent;
use crate::pattern::Pattern;
use regex::Regex;

#[derive(Clone, Debug)]
struct EdgeWeight {
    signatures: VecDeque<String>,
    timestamps: VecDeque<u64>,
    /// Edge id is used for answer checking
    edge_id: VecDeque<usize>,
    /// This counts how many edges (in this compressed "large" edge) are expired,
    /// which counts from the "front" of the deques.
    expiration_counter: usize,
}

impl fmt::Display for EdgeWeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.signatures.iter().join(", "))
    }
}

#[derive(Debug)]
struct ZippedEdge<'p> {
    /// Reference to an edge that collects all signatures
    /// (of the edges that start and end with the same nodes) together.
    zipped_edge: petgraph::graph::EdgeReference<'p, EdgeWeight>,
    /// Corresponds to `subject`.
    from_node: usize,
    /// Corresponds to `object`.
    to_node: usize,
}

// 先做 isomorphism 再檢查 temporal relation
pub struct IsomorphismLayer<'p, P> {
    prev_layer: P,
    pattern: &'p Pattern,
    pattern_graph: Graph<usize, EdgeWeight>,
    /// Let `matched_subgraph` = `self.all_matched_subgrphs.pop()`.
    /// Then `NodeIndex(matched_subgraph[i])` is the input node that matches the pattern node `NodeIndex(i)` .
    all_node_mappings: Vec<Vec<usize>>,

    pattern_entity_node_map: HashMap<usize, NodeIndex>,
    window_size: u64,

    data_graph: Graph<u64, EdgeWeight>,
    data_node_entity_map: HashMap<NodeIndex, usize>,
    data_entity_node_map: HashMap<usize, NodeIndex>,

    /// See `check_expiration()`.
    edges_to_remove: Vec<EdgeIndex>,
    /// A sequence of answers (matched subgraphs) in "event-id mapping" format.
    /// See `PatternMatch::matched_events` for more information.
    event_id_answers: Vec<Vec<u64>>,
}

impl<'p, P> IsomorphismLayer<'p, P> {
    pub fn new(prev_layer: P, pattern: &'p Pattern, window_size: u64) -> Self {
        let mut pattern_graph = Graph::<usize, EdgeWeight>::new();
        let mut entity_list = HashSet::new();
        let mut pattern_node_entity_map = HashMap::new();

        for event in &pattern.events {
            entity_list.insert(event.subject);
            entity_list.insert(event.object);
        }

        debug!("entity_list: {:?}", entity_list);

        let mut pattern_entity_node_map = HashMap::new();
        for entity in entity_list {
            let index = pattern_graph.add_node(entity);
            pattern_entity_node_map.insert(entity, index);
            pattern_node_entity_map.insert(index, entity);
        }

        debug!("entity_node_map: {:?}", pattern_entity_node_map);

        for event in &pattern.events {
            let n0 = pattern_entity_node_map[&event.subject];
            let n1 = pattern_entity_node_map[&event.object];
            pattern_graph.add_edge(
                n0,
                n1,
                EdgeWeight {
                    signatures: VecDeque::from([event.signature.clone()]),
                    timestamps: VecDeque::from([0]),
                    edge_id: VecDeque::from([event.id]),
                    expiration_counter: 0,
                },
            );
        }

        debug!("pattern_graph: {}", Dot::new(&pattern_graph));

        Self {
            prev_layer,
            pattern,
            pattern_graph,
            all_node_mappings: Vec::new(),
            pattern_entity_node_map,
            window_size,
            data_graph: Graph::<u64, EdgeWeight>::new(),
            data_node_entity_map: HashMap::new(),
            data_entity_node_map: HashMap::new(),
            edges_to_remove: Vec::new(),
            event_id_answers: Vec::new(),
        }
    }

    /// Perform subgraph isomorphism, on the current graph with zipped edges.
    /// Add newly matched subgraphs to `self.all_matched_subgrphs`.
    fn try_match(&mut self) {
        let mut node_match = |_: &usize, _: &u64| true;
        // note that `e1` is `pattern`, and `e2` is `data`
        let mut edge_match = |e1: &EdgeWeight, e2: &EdgeWeight| {
            let signatures = &e1.signatures;
            let mut all_signatures = String::new();

            // See document: https://doc.rust-lang.org/std/collections/struct.VecDeque.html#method.as_slices
            // signatures.make_contiguous();
            // It is guaranteed that "signatures" is only updated by "push_back()".
            // Thus, we can safely get all signatures.
            for signature in &signatures.as_slices().0[e1.expiration_counter..] {
                all_signatures.push_str(signature);
                all_signatures.push_str("|");
            }
            // Remove trailing `|`, which is excessive.
            if all_signatures.ends_with("|") {
                all_signatures.pop();
            }
            let re = Regex::new(&all_signatures).unwrap();
            re.is_match(&e2.signatures.front().unwrap())
        };

        let ref_pattern_graph = &self.pattern_graph;
        let ref_data_graph = &self.data_graph;
        // The returned mappings are between "NodeIndex" of `g0` and `g1`
        // That is, if `mapping[0] == 4`, it means that `NodeIndex(0)` of pattern graph matches `NodeIndex(4)` of data graph
        let match_result = algo::subgraph_isomorphisms_iter(
            &ref_pattern_graph,
            &ref_data_graph,
            &mut node_match,
            &mut edge_match,
        );

        if let Some(iter_mappings) = match_result {
            self.all_node_mappings.extend(iter_mappings);
        }
    }

    fn remove_node_from_graph(&mut self, node: NodeIndex) {
        if let Some(entity) = self.data_node_entity_map.get(&node) {
            self.data_entity_node_map.remove(&entity);
            self.data_node_entity_map.remove(&node);
            self.data_graph.remove_node(node);
        } else {
            debug!("node {:?} has already been removed!", node);
        }
    }

    /// Time Complexity:
    ///     - If some edge expires: Depends on VF2
    ///     - Else: `O(|E|)`
    /// Note: 
    ///     Removal of edges (nodes) are necessary in order to reduce the running time of VF2.
    ///     However, dur to the nature of our graph structure, we cannot efficiently spot which
    ///     edges (nodes) should be removed, and thus a traversal of all edges is mandatory.
    fn check_expiration(&mut self, latest_timestamp: u64) {
        let mut nodes_to_remove = Vec::new();
        // Remove edges that expire at "previous" call of `check_expiration()`.
        // If we remove them right away, things will go wrong when we check signatures for those answer candidates.
        // That's because all edges are "references"!
        // In particular, edges are removed after `get_match()` is called.        

        // Edges are removed in sequence, ordered by their index decreasingly.
        // This prevents the situation that, after some edge is removed, the indices 
        // of the remaining edges to be removed are rearranged; if edge index is adjjusted,
        // it is hard to identify the edge without knowing its new index.
        // The same arguement holds for the removal of nodes as well.

        // Note that `self.edges_to_remove()` is ordered increasingly by the indices, by nature.
        // Thus we only need to reverse it.
        for edge in self.edges_to_remove.drain(..).rev() {
            debug!("edge (to be removed): {:?}", edge);
            let (n0, n1) = self.data_graph.edge_endpoints(edge).unwrap();
            // This `edge` is the only one that connects `n0` and `n1`.
            if self.data_graph.edges_connecting(n0, n1).count() == 1 {
                nodes_to_remove.push(n0);
                nodes_to_remove.push(n1);
            }
            self.data_graph.remove_edge(edge);
        }

        // Sort by node index in decreasing order.
        nodes_to_remove.sort_by(|u, v| v.index().cmp(&u.index()));
        for node in nodes_to_remove {
            debug!("node to remove: {:?}", node);
            self.remove_node_from_graph(node);
        }

        debug_assert!(self.edges_to_remove.is_empty());

        let mut expired_edges = Vec::new();
        // Maybe this loop can be optimized.
        // Note that in a general `Graph` (not `StableGraph`), edge order varies after deletion.
        // Thus timestamps (across edges) are not monotone.
        // Time: O(|E|)
        for eid in self.data_graph.edge_indices() {
            for timestamp in &self.data_graph.edge_weight(eid).unwrap().timestamps {
                if latest_timestamp.saturating_sub(self.window_size) > *timestamp {
                    expired_edges.push(eid);
                } else {
                    // However, timestamps within a zipped edge are (non-strictly) increasing.
                    break;
                }
            }
        }

        // Try to match only if some edges expires.
        // This prevents repeated matches and saves time.
        if !expired_edges.is_empty() {
            self.try_match();
            // Once there are new matches, check whether they are valid.
            // If valid, put them into `self.event_id_answers`.
            if !self.all_node_mappings.is_empty() {
                self.verify_matches();
            }
        }

        for eid in expired_edges {
            // NOTE: Nodes should not be deleted, since we rely on NodeIndex::index()
            // However, it is fine to delete nodes after all current matches are flushed.
            let mut is_empty = false;
            if let Some(weight) = self.data_graph.edge_weight_mut(eid) {
                weight.expiration_counter += 1;
                // The flag `is_empty` is set to true only when the equation holds exactly (not `>=`).
                // This prevents multiple entries of the same edge are stored in `edges_to_remove`.
                if weight.expiration_counter == weight.edge_id.len() {
                    is_empty = true;
                }
            }
            if is_empty {
                self.edges_to_remove.push(eid);
            }
        }
    }

    fn get_subgraph_from_mapping(&self, node_mapping: &Vec<usize>) -> Vec<ZippedEdge> {
        let mut zipped_subgraph_edges: Vec<ZippedEdge> = Vec::new();
        for node in node_mapping {
            for edge in self.data_graph.edges(NodeIndex::from(*node as u32)) {
                zipped_subgraph_edges.push(ZippedEdge {
                    zipped_edge: edge,
                    from_node: *node,
                    to_node: edge.target().index(),
                });
                debug_assert_eq!(*node, edge.source().index());
            }
        }
        zipped_subgraph_edges
    }

    /// Since we do not know the maximum node index, we use `HashMap` for adjacency list here.
    fn expand_subgraphs(
        &self,
        current_pos: usize,
        zipped_subgraph_edges: &Vec<ZippedEdge>,
        expanded_subgraph_edges: &mut HashMap<usize, Vec<InputEvent>>,
        all_expanded_subgraph_edges: &mut Vec<HashMap<usize, Vec<InputEvent>>>,
    ) {
        if current_pos == zipped_subgraph_edges.len() {
            all_expanded_subgraph_edges.push(expanded_subgraph_edges.clone());
            return;
        }

        let zipped_edge = zipped_subgraph_edges[current_pos].zipped_edge;
        let n0 = zipped_subgraph_edges[current_pos].from_node;
        let n1 = zipped_subgraph_edges[current_pos].to_node;

        for (i, signature) in enumerate(&zipped_edge.weight().signatures) {
            let id = zipped_edge.weight().edge_id[i];
            let timestamp = zipped_edge.weight().timestamps[i];

            if let Some(edges) = expanded_subgraph_edges.get_mut(&n0) {
                edges.push(InputEvent {
                    timestamp,
                    signature: signature.clone(),
                    id: id as u64,
                    // Caveat! Here "n0" and "n1" are NodeIndex::index(), not original data entity ids
                    subject: n0 as u64,
                    object: n1 as u64,
                });
            } else {
                expanded_subgraph_edges.insert(
                    n0,
                    vec![InputEvent {
                        timestamp,
                        signature: signature.clone(),
                        id: id as u64,
                        // Caveat! Here "n0" and "n1" are NodeIndex::index(), not original data entity ids
                        subject: n0 as u64,
                        object: n1 as u64,
                    }],
                );
            }
            self.expand_subgraphs(
                current_pos + 1,
                zipped_subgraph_edges,
                expanded_subgraph_edges,
                all_expanded_subgraph_edges,
            );
            expanded_subgraph_edges.get_mut(&n0).unwrap().pop();
        }
    }

    fn get_mapped_edge(
        &self,
        pattern_eid: usize,
        node_mapping: &Vec<usize>,
        subgraph_edges: &'p HashMap<usize, Vec<InputEvent>>,
    ) -> Option<&InputEvent> {
        // Assume that pattern events are ordered by ther "id" (from 0, 1, ...).
        let event = &self.pattern.events[pattern_eid];
        let pattern_n0 = self.pattern_entity_node_map[&event.subject].index();
        let pattern_n1 = self.pattern_entity_node_map[&event.object].index();

        let data_n0 = node_mapping[pattern_n0];
        let data_n1 = node_mapping[pattern_n1];

        let edges = subgraph_edges.get(&data_n0)?;
        for edge in edges {
            if edge.object == data_n1 as u64 {
                return Some(edge);
            }
        }
        None
    }

    fn check_order_relation(&self, root: usize, matched_events: &Vec<&InputEvent>) -> bool {
        for eid in self.pattern.order.get_next(root) {
            if matched_events[eid].timestamp < matched_events[root].timestamp {
                return false;
            }
            if !self.check_order_relation(eid, matched_events) {
                return false;
            }
        }
        true
    }

    fn check_signatures(&self, matched_events: &Vec<&InputEvent>) -> bool {
        for (i, event) in enumerate(&self.pattern.events) {
            let re = Regex::new(&event.signature).unwrap();
            if !re.is_match(&matched_events[i].signature) {
                return false;
            }
        }
        true
    }

    /// If we flush all matches in `self.all_node_mappings` all together, then isolated nodes can be removed.
    /// CURRENT implementation indeed flush "all" matches. Thus nodes can be deleted.
    fn verify_matches(&mut self) {
        debug!("matched size: {}", self.all_node_mappings.len());

        while let Some(node_mapping) = self.all_node_mappings.pop() {
            debug!("Now expanding {:?}", node_mapping);
            let zipped_subgraph_edges = self.get_subgraph_from_mapping(&node_mapping);
            debug!("zipped subgraph edges: {:?}", zipped_subgraph_edges);
            
            let mut expanded_subgraph_edges = HashMap::new();
            let mut all_expanded_subgraph_edges = Vec::new();
            self.expand_subgraphs(
                0,
                &zipped_subgraph_edges,
                &mut expanded_subgraph_edges,
                &mut all_expanded_subgraph_edges,
            );

            debug!(
                "all expanded_subgraph_edges: {:?}",
                all_expanded_subgraph_edges
            );

            for subgraph_edges in all_expanded_subgraph_edges {
                let matched_events = self.to_event_representation(&node_mapping, &subgraph_edges);
                if !self.check_signatures(&matched_events) {
                    continue;
                }

                let mut valid = true;
                for root in self.pattern.order.get_roots() {
                    if !self.check_order_relation(root, &matched_events) {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    self.event_id_answers.push(
                        self.to_event_representation(&node_mapping, &subgraph_edges)
                            .iter()
                            .map(|e| e.id)
                            .collect_vec(),
                    );
                }
            }
        }
    }

    fn to_event_representation(
        &self,
        node_mapping: &Vec<usize>,
        subgraph_edges: &'p HashMap<usize, Vec<InputEvent>>,
    ) -> Vec<&InputEvent> {
        let num_events = self.pattern.events.len();
        let mut events = Vec::with_capacity(num_events);
        for i in 0..num_events {
            let data_edge = self.get_mapped_edge(i, node_mapping, subgraph_edges).unwrap();
            events.push(data_edge);
        }
        events
    }
}

impl<'p, P> Iterator for IsomorphismLayer<'p, P>
where
    P: Iterator<Item = Vec<Rc<InputEvent>>>,
{
    type Item = Vec<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.event_id_answers.is_empty() {
            if let Some(event_batch) = self.prev_layer.next() {
                // All events in a batch has the same timestamp,
                // and thus expiration check only needs to be performed once.
                self.check_expiration(event_batch.last().unwrap().timestamp);
                for event in event_batch {
                    debug!("input event: {:?}", event);
                    debug!("current entity_node_map: {:?}", self.data_entity_node_map);

                    let n0 = self.data_entity_node_map.get(&(event.subject as usize));
                    let n1 = self.data_entity_node_map.get(&(event.object as usize));

                    if let (Some(a), Some(b)) = (n0, n1) {
                        debug!("(a, b): ({:?}, {:?})", a, b);

                        // Compress events (compress multiedges).
                        if self.data_graph.contains_edge(*a, *b) {
                            // There should be only "1" edge that connects `a` and `b`.
                            let edge = self.data_graph.edges_connecting(*a, *b).last().unwrap();
                            let mut weight = edge.weight().clone();
                            weight.signatures.push_back(event.signature.clone());
                            weight.timestamps.push_back(event.timestamp);
                            weight.edge_id.push_back(event.id as usize);

                            self.data_graph.update_edge(*a, *b, weight);
                        } else {
                            self.data_graph.add_edge(
                                *a,
                                *b,
                                EdgeWeight {
                                    signatures: VecDeque::from([event.signature.clone()]),
                                    timestamps: VecDeque::from([event.timestamp]),
                                    edge_id: VecDeque::from([event.id as usize]),
                                    expiration_counter: 0,
                                },
                            );
                        }
                    } else {
                        let mut a = NodeIndex::from(0u32);
                        let mut b = NodeIndex::from(0u32);

                        if let Some(node) = n0 {
                            a = *node;
                        } else {
                            a = self.data_graph.add_node(event.subject);
                        }

                        if let Some(node) = n1 {
                            b = *node;
                        } else {
                            b = self.data_graph.add_node(event.object);
                        }

                        self.data_graph.add_edge(
                            a,
                            b,
                            EdgeWeight {
                                signatures: VecDeque::from([event.signature.clone()]),
                                timestamps: VecDeque::from([event.timestamp]),
                                edge_id: VecDeque::from([event.id as usize]),
                                expiration_counter: 0,
                            },
                        );

                        self.data_entity_node_map.insert(event.subject as usize, a);
                        self.data_entity_node_map.insert(event.object as usize, b);
                        self.data_node_entity_map.insert(a, event.subject as usize);
                        self.data_node_entity_map.insert(b, event.object as usize);
                    }

                    debug!("data graph: {}", Dot::new(&self.data_graph));
                }
            } else {
                debug!("prev layer no stuff, flush all");
                self.check_expiration(u64::MAX);
                break;
            }
        }
        self.event_id_answers.pop()
    }
}
