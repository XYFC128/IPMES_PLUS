use itertools::enumerate;
use itertools::Itertools;
use log::debug;
use petgraph::algo;
use petgraph::dot::Dot;
use petgraph::graph::Edge;
use petgraph::graph::NodeIndex;
/// prevent NodeIndex from varying after deletions
// use petgraph::stable_graph::StableGraph;
use petgraph::Graph;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

use crate::input_event::InputEvent;
use crate::pattern::order_relation::OrderRelation;
use crate::pattern::Pattern;
use regex::Regex;

#[derive(Clone, Debug)]
struct EdgeWeight {
    signatures: Vec<String>,
    // signatures: Vec<Rc<String>>,
    timestamp: u64,
}

impl fmt::Display for EdgeWeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.signatures.iter().join(", "))
    }
}

// 把 signature 疊起來 (OR)
// 先做 isomorphism 再檢查 temporal relation
// Maybe we need to store "node_entity_map" for both pattern graph and data graph
pub struct IsomorphismLayer<'p, P> {
    prev_layer: P,
    // node weight: node id; edge weight: list of signatures (multiedges are overlapped)
    pattern_graph: Graph<usize, EdgeWeight>,
    // post-processing: check temporal order after subgraphs are matched
    temporal_order: &'p OrderRelation,
    all_matched_subgrphs: Vec<Vec<usize>>,
    window_size: u64,

    data_graph: Graph<u64, EdgeWeight>,
    // these are for data graph
    node_entity_map: HashMap<NodeIndex, u64>,
    entity_node_map: HashMap<usize, NodeIndex>,
}

impl<'p, P> IsomorphismLayer<'p, P> {
    pub fn new(prev_layer: P, pattern: &'p Pattern, window_size: u64) -> Self {
        let mut pattern_graph = Graph::<usize, EdgeWeight>::new();
        // let mut data_graph = Graph::<u64, EdgeWeight>::new();
        let mut entity_list = HashSet::new();

        for event in &pattern.events {
            entity_list.insert(event.subject);
            entity_list.insert(event.object);
        }

        debug!("entity_list: {:?}", entity_list);

        // this is for pattern graph, which `self.entity_node_map` is for data graph
        let mut entity_node_map = HashMap::new();
        for entity in entity_list {
            let index = pattern_graph.add_node(entity);
            entity_node_map.insert(entity, index);
        }

        debug!("entity_node_map: {:?}", entity_node_map);

        for event in &pattern.events {
            let n0 = entity_node_map[&event.subject];
            let n1 = entity_node_map[&event.object];
            pattern_graph.add_edge(
                n0,
                n1,
                EdgeWeight {
                    signatures: vec![event.signature.clone()],
                    // signatures: vec![Rc::new(event.signature.clone())],
                    timestamp: 0,
                },
            );
        }

        debug!("pattern_graph: {}", Dot::new(&pattern_graph));

        Self {
            prev_layer,
            pattern_graph,
            temporal_order: &pattern.order,
            all_matched_subgrphs: Vec::new(),
            window_size,
            data_graph: Graph::<u64, EdgeWeight>::new(),
            node_entity_map: HashMap::new(),
            entity_node_map: HashMap::new(),
        }
    }

    fn try_match(&mut self) {
        let mut node_match = |_: &usize, _: &u64| true;
        // note that `e1` is `pattern`, and `e2` is `data`
        let mut edge_match = |e1: &EdgeWeight, e2: &EdgeWeight| {
            let signatures = &e1.signatures;
            let mut all_signatures = String::new();
            for signature in signatures {
                all_signatures.push_str("|");
                all_signatures.push_str(signature);
            }
            let re = Regex::new(&all_signatures).unwrap();
            re.is_match(&e2.signatures[0])
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
        let mut expired_edges = Vec::new();
        // Maybe this loop can be optimized
        // Note that in general Graph (not StableGraph), edge order varies after deletion
        // Thus timestamps are not monotone
        for eid in self.data_graph.edge_indices() {
            if latest_timestamp.saturating_sub(self.window_size)
                > self.data_graph.edge_weight(eid).unwrap().timestamp
            {
                expired_edges.push(eid);
            }
        }

        if !expired_edges.is_empty() {
            self.try_match();
        }

        for expired_edge in expired_edges {
            // only edges are removed, isolated nodes remain (correctness is satisfied)
            // maybe isolated nodes should be cleared
            self.data_graph.remove_edge(expired_edge);
        }
    }

    fn get_match(&mut self) -> Option<Vec<usize>> {
        // Let `matched_subgraph` = `self.all_matched_subgrphs.pop()`.
        // `matched_subgraph[i]` is the input node that matches pattern node `i`.
        self.all_matched_subgrphs.pop()
    }
}

impl<'p, P> Iterator for IsomorphismLayer<'p, P>
where
    P: Iterator<Item = Vec<Rc<InputEvent>>>,
{
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.all_matched_subgrphs.is_empty() {
            if let Some(event_batch) = self.prev_layer.next() {
                // All events in a batch has the same timestamp,
                // and thus expiration check only needs to be performed once
                self.check_expiration(event_batch.last().unwrap().timestamp);
                for event in event_batch {
                    debug!("input event: {:?}", event);
                    debug!("current entity_node_map: {:?}", self.entity_node_map);

                    let n0 = self.entity_node_map.get(&(event.subject as usize));
                    let n1 = self.entity_node_map.get(&(event.object as usize));

                    if let (Some(a), Some(b)) = (n0, n1) {
                        debug!("(a, b): ({:?}, {:?})", a, b);

                        // compress events (compress multiedges)
                        if self.data_graph.contains_edge(*a, *b) {
                            // there should be only "1" edge that connects `a` and `b`
                            let edge = self.data_graph.edges_connecting(*a, *b).last().unwrap();
                            let mut weight = edge.weight().clone();
                            weight.signatures.push(event.signature.clone());
                            // weight.signatures.push(Rc::new(event.signature));
                            self.data_graph.update_edge(*a, *b, weight);
                        } else {
                            self.data_graph.add_edge(
                                *a,
                                *b,
                                EdgeWeight {
                                    signatures: vec![event.signature.clone()],
                                    // signatures: vec![Rc::new(event.signature)],
                                    timestamp: event.timestamp,
                                },
                            );
                        }
                    } else {
                        let mut a = NodeIndex::new(0);
                        let mut b = NodeIndex::new(0);

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
                                signatures: vec![event.signature.clone()],
                                // signatures: vec![Rc::new(event.signature)],
                                timestamp: event.timestamp,
                            },
                        );

                        self.entity_node_map.insert(event.subject as usize, a);
                        self.entity_node_map.insert(event.object as usize, b);
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