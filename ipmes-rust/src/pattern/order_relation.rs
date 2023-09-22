use crate::pattern::parser::PatternParsingError;
use petgraph::graph::{DefaultIx, Graph};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, Read};
use petgraph::graph::NodeIndex;
use petgraph::Direction;

pub struct OrderRelation {
    graph: Graph<usize, ()>,
}

impl OrderRelation {
    pub fn get_previous(&self, eid: usize) -> impl Iterator<Item = usize> + '_ {
        let idx = NodeIndex::<DefaultIx>::new(eid + 1);
        self.graph.neighbors_directed(idx, Direction::Incoming)
            .filter_map(|idx| {
                if idx.index() > 0 {
                    Some(idx.index() - 1)
                } else {
                    None
                }
            })
    }

    pub fn get_next(&self, eid: usize) -> impl Iterator<Item = usize> + '_ {
        let idx = NodeIndex::<DefaultIx>::new(eid + 1);
        self.graph.neighbors_directed(idx, Direction::Outgoing)
            .map(|idx| idx.index() - 1)
    }

    pub fn parse(order_relation_file: &str) -> Result<Self, PatternParsingError> {
        let mut file = File::open(order_relation_file)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        let json_obj: Value = serde_json::from_slice(&content)?;

        let orel_edges = Self::parse_json_obj(&json_obj).ok_or(PatternParsingError::FormatError(
            0,
            "Json format of order relation file is not right.",
        ))?;

        Ok(Self {
            graph: Graph::from_edges(&orel_edges),
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
}
