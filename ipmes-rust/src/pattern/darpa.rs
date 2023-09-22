use serde_json::Value;
use crate::pattern::parser::PatternParser;

pub struct DarpaPatternParser;

impl PatternParser for DarpaPatternParser {
    fn node_signature(obj: &Value) -> Option<String> {
        todo!()
    }

    fn edge_signature(obj: &Value) -> Option<String> {
        todo!()
    }
}