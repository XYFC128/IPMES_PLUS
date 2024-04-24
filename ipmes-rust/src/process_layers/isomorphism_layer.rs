use core::time;
use std::num;
use itertools::enumerate;
use itertools::zip;
use itertools::Itertools;
use log::debug;
use nix::libc::times;
use petgraph::algo;
use petgraph::data::DataMapMut;
use petgraph::dot::Dot;
use petgraph::graph::Edge;
use petgraph::graph::EdgeIndex;
use petgraph::graph::Node;
use petgraph::graph::NodeIndex;
use petgraph::visit::IntoEdges;
/// prevent NodeIndex from varying after deletions
// use petgraph::stable_graph::StableGraph;
use petgraph::Graph;
use std::any::Any;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

use crate::input_event::InputEvent;
use crate::pattern::order_relation::OrderRelation;
use crate::pattern::Pattern;
use regex::Regex;

#[derive(Clone, Debug)]
struct EdgeWeight {
    signatures: VecDeque<String>,
    timestamps: VecDeque<u64>,
    /// Edge id is used for answer checking
    edge_id: VecDeque<usize>,
    expiration_counter: usize,
}

impl fmt::Display for EdgeWeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.signatures.iter().join(", "))
    }
}

#[derive(Debug)]
struct ZippedEdge<'p> {
    zipped_edge: petgraph::graph::EdgeReference<'p, EdgeWeight>,
    from_node: usize,
    to_node: usize,
}

// 把 signature 疊起來 (OR)
// 先做 isomorphism 再檢查 temporal relation
// Maybe we need to store "node_entity_map" for both pattern graph and data graph
pub struct IsomorphismLayer<'p, P> {
    prev_layer: P,
    // node weight: node id; edge weight: list of signatures (multiedges are overlapped)
    pattern: &'p Pattern,
    pattern_graph: Graph<usize, EdgeWeight>,
    // post-processing: check temporal order after subgraphs are matched
    // order_relation: &'p OrderRelation,
    /// Let `matched_subgraph` = `self.all_matched_subgrphs.pop()`.
    /// `matched_subgraph[i]` is the input node that matches pattern node `i`. (NodeIndex)
    all_matched_subgrphs: Vec<Vec<usize>>,

    pattern_node_entity_map: HashMap<NodeIndex, usize>,
    pattern_entity_node_map: HashMap<usize, NodeIndex>,
    window_size: u64,

    data_graph: Graph<u64, EdgeWeight>,
    // Below are for data graph
    // !todo("Key can be NodeIndex's index, not NodeIndex itself")
    data_node_entity_map: HashMap<NodeIndex, usize>,
    data_entity_node_map: HashMap<usize, NodeIndex>,

    edges_to_remove: Vec<EdgeIndex>,
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

        // this is for pattern graph, which `self.entity_node_map` is for data graph
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
                    // signatures: vec![Rc::new(event.signature.clone())],
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
            // order_relation: &pattern.order,
            all_matched_subgrphs: Vec::new(),
            pattern_node_entity_map,
            pattern_entity_node_map,
            window_size,
            data_graph: Graph::<u64, EdgeWeight>::new(),
            data_node_entity_map: HashMap::new(),
            data_entity_node_map: HashMap::new(),
            edges_to_remove: Vec::new(),
        }
    }

    fn try_match(&mut self) {
        let mut node_match = |_: &usize, _: &u64| true;
        // note that `e1` is `pattern`, and `e2` is `data`
        let mut edge_match = |e1: &EdgeWeight, e2: &EdgeWeight| {
            let signatures = &e1.signatures;
            let mut all_signatures = String::new();

            // See document: https://doc.rust-lang.org/std/collections/struct.VecDeque.html#method.as_slices
            // signatures.make_contiguous();
            // "signatures" is only updated by "push_back"
            for signature in &signatures.as_slices().0[e1.expiration_counter..] {
                all_signatures.push_str(signature);
                all_signatures.push_str("|");
            }
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
            self.all_matched_subgrphs.extend(iter_mappings);
        }
    }

    fn check_expiration(&mut self, latest_timestamp: u64) {
        // remove edges that expire at "previous" call of "check_expiration"
        // edges are removed after "get_match" is called
        for edge in self.edges_to_remove.drain(..) {
            self.data_graph.remove_edge(edge);
        }
        assert!(self.edges_to_remove.is_empty());

        let mut expired_edges = Vec::new();
        // Maybe this loop can be optimized
        // Note that in general Graph (not StableGraph), edge order varies after deletion
        // Thus timestamps are not monotone
        for eid in self.data_graph.edge_indices() {
            // for (i, timestamp) in enumerate(self.data_graph.edge_weight(eid).unwrap().timestamps) {
            for timestamp in &self.data_graph.edge_weight(eid).unwrap().timestamps {
                if latest_timestamp.saturating_sub(self.window_size) > *timestamp {
                    expired_edges.push(eid);
                } else {
                    // timestamps are (non-strictly) increasing
                    break;
                }
            }
        }

        if !expired_edges.is_empty() {
            self.try_match();
        }

        for eid in expired_edges {
            // only edges are removed, isolated nodes remain (correctness is satisfied)
            // NOTE: Nodes should not be deleted, since we rely on NodeIndex::index()

            // let (eid, i) = expired_edge;
            let mut is_empty = false;
            if let Some(weight) = self.data_graph.edge_weight_mut(eid) {
                // weight.signatures.pop_front();
                // let timestamp = weight.timestamps.pop_front();
                // if timestamp == None {
                // is_empty = true;
                // }
                weight.expiration_counter += 1;
                if weight.expiration_counter == weight.edge_id.len() {
                    is_empty = true;
                }
            }
            if is_empty {
                // self.data_graph.remove_edge(eid);
                self.edges_to_remove.push(eid);
            }
        }
        // !todo("remove isolated nodes");
    }

    // maybe we can use "Graph::edges(EdgeIndex)" to simplfy this function
    fn node_indices_to_edges(&self, subgraph: &Vec<usize>) -> Vec<ZippedEdge> {
        debug!("In node_indices_to_edges()!");

        for edge in self.data_graph.edge_references() {
            debug!("Original edge: {:?}", edge);
        }

        let mut zipped_subgraph_edges: Vec<ZippedEdge> = Vec::new();
        for (i, n0) in enumerate(subgraph) {
            for n1 in subgraph {
                let a = NodeIndex::from(*n0 as u32);
                let b = NodeIndex::from(*n1 as u32);

                if self.data_graph.contains_edge(a, b) {
                    debug!("There are edges between: ({}, {})", n0, n1);
                    // for edge in self.data_graph.edges(a) {
                    //     debug!("Original edge: {:?}", edge);
                    // }

                    // there should be "exactly one" edge
                    let edge = self.data_graph.edges_connecting(a, b).last().unwrap();
                    // debug!("The Edge: {:?}", edge);
                    // debug!("signatures: {:?}", edge.weight().signatures);
                    zipped_subgraph_edges.push(ZippedEdge {
                        zipped_edge: edge,
                        from_node: *n0,
                        to_node: *n1,
                    });

                    for e in self.data_graph.edges_connecting(a, b) {
                        debug!("edge: {:?}", e);
                    }
                }
            }
        }
        zipped_subgraph_edges
    }

    // maybe this "HashMap" stuff can be converted to "Graph"
    fn expand_subgraphs(
        &self,
        current_pos: usize,
        zipped_subgraph_edges: &Vec<ZippedEdge>,
        expanded_subgraph_edges: &mut HashMap<usize, Vec<InputEvent>>,
        // expanded_subgraph_edges: &mut Vec<InputEvent>,
        all_expanded_subgraph_edges: &mut Vec<HashMap<usize, Vec<InputEvent>>>,
    ) {
        if current_pos == zipped_subgraph_edges.len() {
            all_expanded_subgraph_edges.push(expanded_subgraph_edges.clone());
            return;
        }

        let zipped_edge = zipped_subgraph_edges[current_pos].zipped_edge;
        let n0 = zipped_subgraph_edges[current_pos].from_node;
        let n1 = zipped_subgraph_edges[current_pos].to_node;
        // let subject = zipped_subgraph_edges[current_pos].subject;
        // let object = zipped_subgraph_edges[current_pos].object;
        for (i, signature) in enumerate(&zipped_edge.weight().signatures) {
            let id = zipped_edge.weight().edge_id[i];
            let timestamp = zipped_edge.weight().timestamps[i];

            // expanded_subgraph_edges.push(InputEvent {
            //     timestamp,
            //     signature: signature.clone(),
            //     id: id as u64,
            //     subject,
            //     object,
            // });

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
        subgraph: &Vec<usize>,
        subgraph_edges: &'p HashMap<usize, Vec<InputEvent>>,
    ) -> Option<&InputEvent> {
        // Assume that pattern events are ordered by ther "id" (from 0, 1, ...)
        let event = &self.pattern.events[pattern_eid];
        let pattern_n0 = self.pattern_entity_node_map[&event.subject].index();
        let pattern_n1 = self.pattern_entity_node_map[&event.object].index();

        let data_n0 = subgraph[pattern_n0];
        let data_n1 = subgraph[pattern_n1];

        let edges = subgraph_edges.get(&data_n0)?;
        for edge in edges {
            if edge.object == data_n1 as u64 {
                return Some(edge);
            }
        }
        None
    }

    fn check_order_relation(
        &self,
        root: usize,
        subgraph: &Vec<usize>,
        subgraph_edges: &HashMap<usize, Vec<InputEvent>>,
    ) -> bool {
        let root_data_event = self
            .get_mapped_edge(root, subgraph, subgraph_edges)
            .unwrap();
        for eid in self.pattern.order.get_next(root) {
            let data_event = self.get_mapped_edge(eid, subgraph, subgraph_edges).unwrap();
            if data_event.timestamp < root_data_event.timestamp {
                return false;
            }
            if !self.check_order_relation(eid, subgraph, subgraph_edges) {
                return false;
            }
        }
        true
    }

    // fn get_match(&mut self) -> Option<Vec<HashMap<usize, Vec<InputEvent>>>> {
    fn get_match(&mut self) -> Option<Vec<Vec<u64>>> {
        debug!("matched size: {}", self.all_matched_subgrphs.len());
        while let Some(subgraph) = self.all_matched_subgrphs.pop() {
            debug!("Now expanding {:?}", subgraph);

            let zipped_subgraph_edges = self.node_indices_to_edges(&subgraph);
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

            let mut answers = Vec::new();
            for subgraph_edges in all_expanded_subgraph_edges {
                let mut valid = true;
                for root in self.pattern.order.get_roots() {
                    // signature need to be checked as well! (todo)
                    if !self.check_order_relation(root, &subgraph, &subgraph_edges) {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    answers.push(subgraph_edges);
                }
            }
            if !answers.is_empty() {
                // return Some(answers);
                return Some(self.to_event_representation(&answers, &subgraph))
            }

            debug!("------------------------");
        }
        None
    }

    fn to_event_representation(
        &self,
        old_answers: &Vec<HashMap<usize, Vec<InputEvent>>>,
        subgraph: &Vec<usize>,
    ) -> Vec<Vec<u64>> {
        let num_events = self.pattern.events.len();
        let mut answers = Vec::with_capacity(old_answers.len());
        for subgraph_edges in old_answers {
            let mut answer = Vec::with_capacity(num_events);
            for i in 0..num_events {
                let data_edge = self.get_mapped_edge(i, subgraph, subgraph_edges).unwrap();
                answer.push(data_edge.id);
            }
            answers.push(answer);
        }
        answers
    }
}

impl<'p, P> Iterator for IsomorphismLayer<'p, P>
where
    P: Iterator<Item = Vec<Rc<InputEvent>>>,
{
    // type Item = Vec<HashMap<usize, Vec<InputEvent>>>;
    type Item = Vec<Vec<u64>>;

    fn next(&mut self) -> Option<Self::Item> {
        // all_matched_subgrphs.is_empty(): 要改
        while self.all_matched_subgrphs.is_empty() {
            if let Some(event_batch) = self.prev_layer.next() {
                // All events in a batch has the same timestamp,
                // and thus expiration check only needs to be performed once
                self.check_expiration(event_batch.last().unwrap().timestamp);
                for event in event_batch {
                    debug!("input event: {:?}", event);
                    debug!("current entity_node_map: {:?}", self.data_entity_node_map);

                    let n0 = self.data_entity_node_map.get(&(event.subject as usize));
                    let n1 = self.data_entity_node_map.get(&(event.object as usize));

                    if let (Some(a), Some(b)) = (n0, n1) {
                        debug!("(a, b): ({:?}, {:?})", a, b);

                        // compress events (compress multiedges)
                        if self.data_graph.contains_edge(*a, *b) {
                            // there should be only "1" edge that connects `a` and `b`
                            let edge = self.data_graph.edges_connecting(*a, *b).last().unwrap();
                            let mut weight = edge.weight().clone();
                            weight.signatures.push_back(event.signature.clone());
                            // weight.signatures.push(Rc::new(event.signature));
                            self.data_graph.update_edge(*a, *b, weight);
                        } else {
                            self.data_graph.add_edge(
                                *a,
                                *b,
                                EdgeWeight {
                                    signatures: VecDeque::from([event.signature.clone()]),
                                    // signatures: vec![Rc::new(event.signature)],
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
                                // signatures: vec![Rc::new(event.signature)],
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
        self.get_match()
    }
}
