use crate::pattern::parser::PatternParser;
use serde_json::Value;

pub struct SpadePatternParser;

impl PatternParser for SpadePatternParser {
    fn node_signature(obj: &Value) -> Option<String> {
        let properties = &obj["properties"];
        let node_type = properties["type"].as_str()?;

        match node_type {
            "Process" => {
                let name = properties["name"].as_str()?;
                Some(format!("{node_type}::{name}"))
            }

            "Artifact" => {
                let subtype = properties["subtype"].as_str()?;
                match subtype {
                    "file" | "directory" => {
                        let path = properties["path"].as_str()?;
                        Some(format!("{node_type}::{subtype}::{path}"))
                    }

                    "network socket" => {
                        let addr = properties["remote address"].as_str()?;
                        let port = properties["remote port"].as_str()?;
                        Some(format!("{node_type}::{subtype}::{addr}:{port}"))
                    }

                    _ => None,
                }
            }

            _ => None,
        }
    }

    fn edge_signature(obj: &Value) -> Option<String> {
        obj["properties"]["operation"]
            .as_str()
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_node_signature() {
        assert_eq!(
            SpadePatternParser::node_signature(&json!({
                "properties": {
                    "type": "Process",
                    "name": ".*"
                }
            })),
            Some(String::from("Process::.*"))
        );

        assert_eq!(
            SpadePatternParser::node_signature(&json!({
                "properties": {
                    "type": "Artifact",
                    "subtype": "file",
                    "path": "./hello.sh"
                }
            })),
            Some(String::from("Artifact::file::./hello.sh"))
        );

        assert_eq!(
            SpadePatternParser::node_signature(&json!({
                "properties": {
                    "type": "Artifact",
                    "subtype": "directory",
                    "path": "/tmp"
                }
            })),
            Some(String::from("Artifact::directory::/tmp"))
        );

        assert_eq!(
            SpadePatternParser::node_signature(&json!({
                "properties": {
                    "type": "Artifact",
                    "subtype": "network socket",
                    "remote address": "localhost",
                    "remote port": "8080"
                }
            })),
            Some(String::from("Artifact::network socket::localhost:8080"))
        );
    }

    #[test]
    fn test_edge_signature() {
        let obj = json!({
            "properties": {
                "operation": "open"
            }
        });
        assert_eq!(
            SpadePatternParser::edge_signature(&obj),
            Some(String::from("open"))
        );
    }
}
