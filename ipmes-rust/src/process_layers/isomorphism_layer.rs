use petgraph::algo;
use petgraph::data::Build;
use petgraph::graph::NodeIndex;
/// prevent NodeIndex from varying after deletions
// use petgraph::stable_graph::StableGraph;
use petgraph::visit::NodeRef;
use petgraph::Directed;
use petgraph::Graph;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use crate::pattern::order_relation::OrderRelation;
use crate::pattern::Pattern;
use crate::{
    input_event::InputEvent, match_event::MatchEvent, pattern::Event as PatternEvent,
    sub_pattern::SubPattern,
};
use regex::Error as RegexError;
use regex::Regex;

#[derive(Clone)]
struct EdgeWeight {
    signatures: Vec<String>,
    // signatures: Vec<Rc<String>>,
    timestamp: u64,
}

// 把 signature 疊起來 (OR)
// 先做 isomorphism 再檢查 temporal relation
pub struct IsoLayer<'p, P> {
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
    entity_node_map: HashMap<u64, NodeIndex>,
}

impl<'p, P> IsoLayer<'p, P> {
    pub fn new(prev_layer: P, pattern: &'p Pattern, window_size: u64) -> Self {
        let mut pattern_graph = Graph::<usize, EdgeWeight>::new();
        // let mut data_graph = Graph::<u64, EdgeWeight>::new();
        let mut entity_list = HashSet::new();

        for event in &pattern.events {
            entity_list.insert(event.subject);
            entity_list.insert(event.object);
        }

        let mut entity_node_map = HashMap::new();
        for entity in entity_list {
            let index = pattern_graph.add_node(entity);
            entity_node_map.insert(entity, index);
        }

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
        let mut node_match = |a: &usize, b: &u64| true;
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
        let match_result = algo::subgraph_isomorphisms_iter(
            &ref_pattern_graph,
            &ref_data_graph,
            &mut node_match,
            &mut edge_match,
        );

        if let Some(iter_matches) = match_result {
            for subgraph in iter_matches {
                self.all_matched_subgrphs.push(subgraph);
            }
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
            self.data_graph.remove_edge(expired_edge);
        }
    }

    fn get_match(&mut self) -> Option<Vec<usize>> {
        self.all_matched_subgrphs.pop()
    }
}

impl<'p, P> Iterator for IsoLayer<'p, P>
where
    P: Iterator<Item = Vec<Rc<InputEvent>>>,
{
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut new_nodes = vec![];
        while self.all_matched_subgrphs.is_empty() {
            let event_batch = self.prev_layer.next()?;
            // All events in a batch has the same timestamp,
            // and thus expiration check only needs to be performed once 
            self.check_expiration(event_batch.last().unwrap().timestamp);
            for event in event_batch {
                let n0 = self.entity_node_map.get(&event.subject);
                let n1 = self.entity_node_map.get(&event.object);

                if let (Some(a), Some(b)) = (n0, n1) {
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
                    if n0.is_none() {
                        a = self.data_graph.add_node(event.subject);
                        // self.entity_node_map.insert(event.subject, a);
                        new_nodes.push((event.subject, a));
                    } else {
                        a = *n0.unwrap();
                    }

                    if n1.is_none() {
                        b = self.data_graph.add_node(event.object);
                        // self.entity_node_map.insert(event.object, b);
                        new_nodes.push((event.object, b));
                    } else {
                        b = *n1.unwrap();
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
                }

                for new_node in &new_nodes {
                    self.entity_node_map.insert(new_node.0, new_node.1);
                }
            }
        }

        self.get_match()
    }
}
