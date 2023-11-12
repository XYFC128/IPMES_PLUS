use crate::pattern::parser::PatternParsingError;
use petgraph::algo::floyd_warshall;
use petgraph::graph::NodeIndex;
use petgraph::graph::{DefaultIx, Graph};
use petgraph::Direction;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub struct OrderRelation {
    graph: Graph<usize, ()>,
}

impl OrderRelation {
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

        Self {
            graph: Graph::from_edges(&edges),
        }
    }

    pub fn parse(order_relation_file: &str) -> Result<Self, PatternParsingError> {
        let mut file = File::open(order_relation_file)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        let json_obj: Value = serde_json::from_slice(&content)?;

        let orel_edges = Self::parse_json_obj(&json_obj).ok_or(
            PatternParsingError::FormatError(0, "Json format of order relation file is not right."),
        )?;

        Ok(Self {
            graph: Graph::from_edges(&orel_edges),
            // distances_table: HashMap::new(),
        })
    }

    fn parse_json_obj(json_obj: &Value) -> Option<Vec<(u32, u32)>> {
        let mut orel_edges = Vec::new();

        for (key, val) in json_obj.as_object()? {
            let children = val["children"].as_array()?;

            let cur_id = if key == "root" {
                0
            } else {
                let id = key.parse::<u32>().ok()?;
                id + 1 // 0 is reserved for root
            };

            for child in children {
                let child_id = child.as_u64()? + 1;
                orel_edges.push((cur_id, child_id as u32));
            }
        }

        Some(orel_edges)
    }

    pub fn calculate_distances(&self) -> Option<HashMap<(NodeIndex, NodeIndex), i32>> {
        floyd_warshall(&self.graph, |_| 1).ok()
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
    fn test_calculate_distances() {
        let ord = OrderRelation::parse("../data/patterns/TTP11_oRels.json")
            .expect("fail to parse order relation file");

        // ord.distances_table = ord.calculate_distances().unwrap();

        // println!("{:?}", ord.distances_table);

        // println!("{:?}", ord.distances_table.get(&(NodeIndex::new(1), NodeIndex::new(0))).unwrap());
        // println!("{:?}", ord.graph.edge_indices());
    }
}
