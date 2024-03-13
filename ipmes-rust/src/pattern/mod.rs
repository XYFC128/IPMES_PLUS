pub mod order_relation;
pub mod parser;

use self::parser::parse_json;
pub use self::parser::PatternParsingError;
use order_relation::OrderRelation;
use serde_json::Value;
use std::{fs::File, io::Read};

#[derive(Debug, Eq, PartialEq)]
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
    pub subject: usize,
    pub object: usize,
}

#[derive(Debug)]
pub struct Pattern {
    pub use_regex: bool,
    pub entities: Vec<PatternEntity>,
    pub events: Vec<PatternEvent>,
    pub order: OrderRelation,
}

impl Pattern {
    pub fn parse(pattern_file: &str) -> Result<Pattern, PatternParsingError> {
        let mut file = File::open(pattern_file)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        let json_obj: Value = serde_json::from_slice(&content)?;

        parse_json(&json_obj)
    }
}
