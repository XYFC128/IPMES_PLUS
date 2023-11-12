use crate::pattern::parser::PatternParser;
use serde_json::Value;

pub struct DarpaPatternParser;

impl PatternParser for DarpaPatternParser {
    fn node_signature(obj: &Value) -> Option<String> {
        let properties = &obj["properties"].as_object()?;

        let node_type = if let Some(type_val) = properties.get("type") {
            type_val.as_str()?
        } else if properties.contains_key("NetFlowObject_baseObject_epoch") {
            "OBJECT_SOCKET"
        } else if properties.contains_key("UnnamedPipeObject_baseObject_epoch") {
            "OBJECT_UNNAMEPIPE"
        } else {
            "OTHER"
        };

        let signature = match node_type {
            "SUBJECT_PROCESS" | "SUBJECT_UNIT" => {
                properties.get("Subject_properties_map_name")?.as_str()?
            }
            "FILE_OBJECT_CHAR" | "FILE_OBJECT_FILE" | "FILE_OBJECT_DIR" => {
                properties.get("path")?.as_str()?
            }
            "OBJECT_SOCKET" => {
                let addr = properties.get("NetFlowObject_remoteAddress")?.as_str()?;
                let port = properties.get("NetFlowObject_remotePort")?;
                if port.is_string() {
                    return Some(format!("{node_type}::{addr}:{}", port.as_str()?));
                } else {
                    return Some(format!("{node_type}::{addr}:{port}"));
                }
            }
            _ => "",
        };

        Some(format!("{node_type}::{signature}"))
    }

    fn edge_signature(obj: &Value) -> Option<String> {
        obj["properties"]["type"].as_str().map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_node_signature() {
        assert_eq!(
            DarpaPatternParser::node_signature(&json!({
                "properties": {
                    "type": "SUBJECT_PROCESS",
                    "Subject_properties_map_name": ".*"
                }
            })),
            Some(String::from("SUBJECT_PROCESS::.*"))
        );

        assert_eq!(
            DarpaPatternParser::node_signature(&json!({
                "properties": {
                    "type": "SUBJECT_UNIT",
                    "Subject_properties_map_name": ".*"
                }
            })),
            Some(String::from("SUBJECT_UNIT::.*"))
        );

        assert_eq!(
            DarpaPatternParser::node_signature(&json!({
                "properties": {
                    "type": "FILE_OBJECT_CHAR",
                    "path": "/dev/tty0"
                }
            })),
            Some(String::from("FILE_OBJECT_CHAR::/dev/tty0"))
        );

        assert_eq!(
            DarpaPatternParser::node_signature(&json!({
                "properties": {
                    "type": "FILE_OBJECT_FILE",
                    "path": "./hello.sh"
                }
            })),
            Some(String::from("FILE_OBJECT_FILE::./hello.sh"))
        );

        assert_eq!(
            DarpaPatternParser::node_signature(&json!({
                "properties": {
                    "type": "FILE_OBJECT_DIR",
                    "path": "/tmp"
                }
            })),
            Some(String::from("FILE_OBJECT_DIR::/tmp"))
        );

        assert_eq!(
            DarpaPatternParser::node_signature(&json!({
                "properties": {
                    "type": "Foo",
                }
            })),
            Some(String::from("Foo::"))
        );

        assert_eq!(
            DarpaPatternParser::node_signature(&json!({
                "properties": {
                }
            })),
            Some(String::from("OTHER::"))
        );
    }

    #[test]
    fn test_node_signature_socket() {
        assert_eq!(
            DarpaPatternParser::node_signature(&json!({
                "properties": {
                    "NetFlowObject_baseObject_epoch": 10,
                    "NetFlowObject_remoteAddress": "localhost",
                    "NetFlowObject_remotePort": "8080"
                }
            })),
            Some(String::from("OBJECT_SOCKET::localhost:8080"))
        );

        // port's type in some pattern is integer
        assert_eq!(
            DarpaPatternParser::node_signature(&json!({
                "properties": {
                    "NetFlowObject_baseObject_epoch": 10,
                    "NetFlowObject_remoteAddress": "localhost",
                    "NetFlowObject_remotePort": 8080
                }
            })),
            Some(String::from("OBJECT_SOCKET::localhost:8080"))
        );
    }

    #[test]
    fn test_edge_signature() {
        let obj = json!({
            "properties": {
                "type": "open"
            }
        });
        assert_eq!(
            DarpaPatternParser::edge_signature(&obj),
            Some(String::from("open"))
        );
    }
}
