use std::cmp::max;
use crate::pattern::{Edge, Pattern};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use log::info;
use thiserror::Error;
use crate::pattern::order_relation::OrderRelation;

#[derive(Error, Debug)]
pub enum PatternParsingError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),

    #[error("Json format error")]
    SerdeError(#[from] serde_json::Error),

    #[error("Pattern format error at line {0} : {1}")]
    FormatError(usize, &'static str),
}

fn parse_u64(obj: &Value) -> Result<u64, &'static str> {
    obj.as_str()
        .ok_or("ID of node should be a string")?
        .parse::<u64>()
        .map_err(|_| "ID is not u64")
}
pub trait PatternParser {
    fn node_signature(obj: &Value) -> Option<String>;

    fn edge_signature(obj: &Value) -> Option<String>;

    fn parse(
        &self,
        node_file: &str,
        edges_file: &str,
        order_relation_file: &str,
    ) -> Result<Pattern, PatternParsingError> {
        info!("Parsing: {node_file}, {edges_file}, {order_relation_file}");
        let (id_convert, node_signatures) = self.parse_node_file(node_file)?;
        let edges = self.parse_edge_file(edges_file, &id_convert, &node_signatures)?;
        let num_nodes = edges
            .iter()
            .map(|e| max(e.start, e.end))
            .max()
            .unwrap() + 1;

        Ok(Pattern {
            edges,
            order: OrderRelation::parse(order_relation_file)?,
            num_nodes
        })
    }

    fn parse_node_file(
        &self,
        node_file: &str,
    ) -> Result<(HashMap<u64, usize>, Vec<String>), PatternParsingError> {
        use PatternParsingError::*;

        let mut id_convert = HashMap::new();
        let mut node_signatures = Vec::new();

        let node_file = File::open(node_file)?;
        let node_reader = BufReader::new(node_file);
        for (line_num, line) in node_reader.lines().filter_map(|r| r.ok()).enumerate() {
            let json_obj: Value = serde_json::from_str(&line)?;
            let node_obj = &json_obj["node"];
            let raw_id =
                parse_u64(&node_obj["id"]).map_err(|e| FormatError(line_num + 1, e))?;
            id_convert.insert(raw_id, line_num);

            let signature = Self::node_signature(node_obj).ok_or(FormatError(
                line_num + 1,
                "Fail to extract node signature, maybe the pattern format is wrong",
            ))?;

            node_signatures.push(signature);
        }

        Ok((id_convert, node_signatures))
    }

    fn parse_edge_file(
        &self,
        edge_file: &str,
        id_convert: &HashMap<u64, usize>,
        node_sigs: &Vec<String>,
    ) -> Result<Vec<Edge>, PatternParsingError> {
        use PatternParsingError::*;

        let mut edges = Vec::new();

        let edge_file = File::open(edge_file)?;
        let edge_reader = BufReader::new(edge_file);
        for (line_num, line) in edge_reader.lines().filter_map(|r| r.ok()).enumerate() {
            let json_obj: Value = serde_json::from_str(&line)?;
            let edge_obj = &json_obj["edge"];
            let raw_start = parse_u64(&edge_obj["start"]["id"])
                .map_err(|e| FormatError(line_num + 1, e))?;
            let raw_end = parse_u64(&edge_obj["end"]["id"])
                .map_err(|e| FormatError(line_num + 1, e))?;
            let signature = Self::edge_signature(edge_obj).ok_or(FormatError(
                line_num + 1,
                "Fail to extract edge signature, maybe the pattern format is wrong",
            ))?;

            let start_id = id_convert.get(&raw_start).ok_or(FormatError(
                line_num + 1,
                "The start id not exist in the node file",
            ))?;
            let end_id = id_convert.get(&raw_end).ok_or(FormatError(
                line_num + 1,
                "The end id not exist in the node file",
            ))?;

            let full_signature = format!(
                "{}#{}#{}",
                signature, node_sigs[*start_id], node_sigs[*end_id]
            );
            edges.push(Edge {
                id: line_num,
                signature: full_signature,
                start: *start_id,
                end: *end_id,
            })
        }

        Ok(edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestParser;
    impl PatternParser for TestParser {
        fn node_signature(obj: &Value) -> Option<String> {
            Some(String::from("node"))
        }

        fn edge_signature(obj: &Value) -> Option<String> {
            Some(String::from("edge"))
        }
    }

    #[test]
    fn test_parsing() {
        let parser = TestParser;
        let pattern = parser.parse(
            "../data/patterns/TTP11_node.json",
            "../data/patterns/TTP11_edge.json",
            "../data/patterns/TTP11_orels.json",
        );
    }
}
