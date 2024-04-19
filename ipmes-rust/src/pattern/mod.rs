pub mod order_relation;
pub mod parser;

use self::parser::parse_json;
pub use self::parser::PatternParsingError;
use order_relation::OrderRelation;
use serde_json::Value;
use std::{fs::File, io::Read};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct PatternEntity {
    pub id: usize,
    pub signature: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PatternEventType {
    Default,
    Frequency(u32),
    Flow,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PatternEvent {
    pub id: usize,
    pub event_type: PatternEventType,
    pub signature: String,
    pub subject: PatternEntity,
    pub object: PatternEntity,
}

#[derive(Debug)]
pub struct Pattern {
    pub use_regex: bool,
    pub entities: Vec<PatternEntity>,
    pub events: Vec<PatternEvent>,
    pub order: OrderRelation,
}

impl Pattern {
    pub fn parse(pattern_file: &str) -> Result<Self, PatternParsingError> {
        let mut file = File::open(pattern_file)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        let json_obj: Value = serde_json::from_slice(&content)?;

        parse_json(&json_obj)
    }

    /// Create pattern from graph (V, E). Each vertex is associated with a signature.
    /// Edges are given by a pair *(u, v, sig)* where *u* and *v* are indics into the `vertices` slice,
    /// and *sig* is the signature of this edge.
    ///
    /// The dependency is not specified, so there are no dependency requirement in the returned
    /// pattern.
    pub fn from_graph(vertices: &[&str], edges: &[(usize, usize, &str)], use_regex: bool) -> Self {
        let mut entities = vec![];
        for (id, signature) in vertices.iter().enumerate() {
            entities.push(PatternEntity {
                id,
                signature: signature.to_string(),
            });
        }

        let mut events = vec![];
        let mut roots = vec![];
        for (id, edge) in edges.iter().enumerate() {
            roots.push(id as u32); // all edges are root (no dependency)
            events.push(PatternEvent {
                id,
                event_type: PatternEventType::Default,
                signature: edge.2.to_string(),
                subject: entities[edge.0].clone(),
                object: entities[edge.1].clone(),
            });
        }

        let order = OrderRelation::from_order_rules(&[], &roots);

        Self {
            use_regex,
            entities,
            events,
            order,
        }
    }
}
