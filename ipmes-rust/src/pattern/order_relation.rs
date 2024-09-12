use crate::pattern::parser::PatternParsingError;
use petgraph::algo::floyd_warshall;
use petgraph::graph::NodeIndex;
use petgraph::graph::{DefaultIx, Graph};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct OrderRelation {
    pub graph: Graph<usize, ()>,
    distances_table: HashMap<(NodeIndex, NodeIndex), i32>,
}

impl From<Graph<usize, ()>> for OrderRelation {
    fn from(value: Graph<usize, ()>) -> Self {
        let distances_table = floyd_warshall(&value, |_| 1).ok().unwrap();
        Self {
            graph: value,
            distances_table,
        }
    }
}

impl OrderRelation {
    /// Return an iterator over the pair (source id, target id) of dependency edges, where the ids
    /// are those of pattern edges'.
    pub fn get_dependencies(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.graph
            .edge_references()
            .filter_map(|edge| {
                let src = edge.source().index();
                let tgt = edge.target().index();
                
                if src == 0 || tgt == 0 {
                    None
                } else {
                    Some((src - 1, tgt - 1))
                }
            })
    }

    /// Returns an iterator over the id of pattern edges that should appear **before** the given pattern
    /// edge.
    pub fn get_previous(&self, eid: usize) -> impl Iterator<Item = usize> + '_ {
        // Indices in "graph" is incremented by 1, since "0" is reserved for "root".
        let idx = NodeIndex::<DefaultIx>::new(eid + 1);
        self.graph
            .neighbors_directed(idx, Direction::Incoming)
            .filter_map(|idx| {
                if idx.index() > 0 {
                    Some(idx.index() - 1)
                } else {
                    None
                }
            })
    }

    /// Returns an iterator over the id of pattern edges that should appear **after** the given pattern
    /// edge.
    pub fn get_next(&self, eid: usize) -> impl Iterator<Item = usize> + '_ {
        let idx = NodeIndex::<DefaultIx>::new(eid + 1);
        self.graph
            .neighbors_directed(idx, Direction::Outgoing)
            .map(|idx| idx.index() - 1)
    }

    /// Returns an iterator over the id of pattern edges that are roots
    pub fn get_roots(&self) -> impl Iterator<Item = usize> + '_ {
        let idx = NodeIndex::<DefaultIx>::new(0);
        self.graph
            .neighbors_directed(idx, Direction::Outgoing)
            .map(|idx| idx.index() - 1)
    }

    /// Construct OrderRelation from order rules for easier unit testing.
    ///
    pub fn from_order_rules(order_rules: &[(u32, u32)], roots: &[u32]) -> Self {
        let mut edges = Vec::new();
        for (a, b) in order_rules {
            edges.push((a + 1, b + 1))
        }
        for root in roots {
            edges.push((0, root + 1))
        }

        Graph::from_edges(&edges).into()
    }

    pub fn parse(order_relation_file: &str) -> Result<Self, PatternParsingError> {
        let mut file = File::open(order_relation_file)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        let json_obj: Value = serde_json::from_slice(&content)?;

        let orel_edges = Self::parse_json_obj(&json_obj)?;

        Ok(Graph::from_edges(orel_edges).into())
    }

    fn parse_json_obj(json_obj: &Value) -> Result<Vec<(u32, u32)>, PatternParsingError> {
        let mut orel_edges = Vec::new();

        let root_obj = json_obj.as_object().ok_or(PatternParsingError::TypeError(
            "order relation should be a json object",
        ))?;

        for (key, val) in root_obj {
            let children = val["children"]
                .as_array()
                .ok_or(PatternParsingError::KeyError("children"))?;

            let cur_id = if key == "root" {
                0
            } else {
                let id = key.parse::<u32>().map_err(|_| {
                    PatternParsingError::TypeError(
                        "key should be a string of positive 32-bit integer",
                    )
                })?;
                id + 1 // 0 is reserved for root
            };

            for child in children {
                let child_id = child.as_u64().ok_or(PatternParsingError::TypeError(
                    "child id should be an integer",
                ))? + 1;
                orel_edges.push((cur_id, child_id as u32));
            }
        }

        Ok(orel_edges)
    }

    /// Return the distance from "eid1" to "eid2" (in DAG).
    pub fn get_distance(&self, eid1: &usize, eid2: &usize) -> i32 {
        let id1 = NodeIndex::<DefaultIx>::new(*eid1 + 1);
        let id2 = NodeIndex::<DefaultIx>::new(*eid2 + 1);
        *self.distances_table.get(&(id1, id2)).unwrap()
    }

    /// Validate the order relation. That is, check if there exist
    /// a cycle in the dependency graph. If a cycle is found, return
    /// `false`, otherwise return `true`.
    pub fn is_valid(&self) -> bool {
        for root in self.get_roots() {
            let mut ancestors = HashSet::new();
            let mut visited = HashSet::new();
            let root = NodeIndex::<DefaultIx>::new(root);
            if self.contains_cycle(root, &mut ancestors, &mut visited) {
                return false;
            }
        }
        true
    }

    fn contains_cycle(
        &self,
        root: NodeIndex,
        ancestors: &mut HashSet<NodeIndex>,
        visited: &mut HashSet<NodeIndex>,
    ) -> bool {
        if visited.contains(&root) {
            return false;
        }

        ancestors.insert(root);
        visited.insert(root);
        let neighbors = self.graph.neighbors_directed(root, Direction::Outgoing);
        for nxt in neighbors {
            if ancestors.contains(&nxt) || self.contains_cycle(nxt, ancestors, visited) {
                return true;
            }
        }
        ancestors.remove(&root);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        let ord = OrderRelation::parse("../data/patterns/TTP11_oRels.json")
            .expect("fail to parse order relation file");
        for neighbor in ord.get_previous(1) {
            println!("{:?}", neighbor);
        }
    }

    #[test]
    fn test_cycle_detection() {
        let normal_rules = [(0, 1), (1, 2), (2, 3)];
        let order = OrderRelation::from_order_rules(&normal_rules, &[0]);
        assert!(order.is_valid());

        let cycle_rules = [(0, 1), (1, 2), (2, 3), (3, 1)];
        let order = OrderRelation::from_order_rules(&cycle_rules, &[0]);
        assert!(!order.is_valid());
    }
}
