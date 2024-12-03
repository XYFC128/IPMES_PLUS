use super::{
    order_relation::OrderRelation, Pattern, PatternEntity, PatternEvent, PatternEventType,
};
use log::warn;
use petgraph::Graph;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PatternParsingError {
    #[error("io error")]
    IoError(#[from] std::io::Error),

    #[error("json format error")]
    SerdeError(#[from] serde_json::Error),

    #[error("key not found or the type is unexpected: {0}")]
    KeyError(&'static str),

    #[error("unexpected type of json object: {0}")]
    TypeError(&'static str),

    #[error("there must be at least one pattern event")]
    NoEventInPattern,

    #[error("the pattern version is too old or invalid: {0}")]
    UnsupportedVersion(String),

    #[error("unknown event type: {0}")]
    UnknownEventType(String),

    #[error("undefined entity id: {0}")]
    UndefinedEntityId(usize),

    #[error("undefined event id: {0}")]
    UndefinedEventId(usize),

    #[error("frequency must be an integer that > 1, but {0} is provided")]
    InvalidFrequency(u32),

    #[error("cycle detected in the dependency graph")]
    DependencyCycle,
}

pub fn get_input_files(input_prefix: &str) -> (String, String, String) {
    let node_file = format!("{}_node.json", input_prefix);
    let edge_file = format!("{}_edge.json", input_prefix);
    let orels_file = if let Some(prefix) = input_prefix.strip_suffix("_regex") {
        format!("{}_oRels.json", prefix)
    } else {
        format!("{}_oRels.json", input_prefix)
    };

    (node_file, edge_file, orels_file)
}

pub fn parse_json(json_obj: &Value) -> Result<Pattern, PatternParsingError> {
    let version = json_obj["Version"]
        .as_str()
        .ok_or(PatternParsingError::KeyError("Version"))?;
    if version != "0.2.0" {
        return Err(PatternParsingError::UnsupportedVersion(version.to_string()));
    }

    let use_regex = json_obj["UseRegex"].as_bool().unwrap_or(true);

    let entities_json = json_obj["Entities"]
        .as_array()
        .ok_or(PatternParsingError::KeyError("Entities"))?;
    let entities = parse_entities(entities_json)?;
    let entity_id2index = get_entity_id2index(&entities);

    let events_json = json_obj["Events"]
        .as_array()
        .ok_or(PatternParsingError::KeyError("Events"))?;
    let mut events = parse_events(events_json, &entity_id2index, &entities)?;
    let event_id2index = reassign_event_id(&mut events);

    let order = parse_order_relation(events_json, &event_id2index)?;
    if !order.is_valid() {
        return Err(PatternParsingError::DependencyCycle);
    }

    Ok(Pattern {
        use_regex,
        entities,
        events,
        order,
    })
}

fn parse_entities(entities_json: &[Value]) -> Result<Vec<PatternEntity>, PatternParsingError> {
    let mut entities = vec![];
    for entity in entities_json {
        let id = entity["ID"]
            .as_u64()
            .ok_or(PatternParsingError::KeyError("ID"))? as usize;
        let signature = entity["Signature"]
            .as_str()
            .ok_or(PatternParsingError::KeyError("Signature"))?
            .to_string();
        entities.push(PatternEntity { id, signature });
    }

    Ok(entities)
}

fn get_entity_id2index(entities: &[PatternEntity]) -> HashMap<usize, usize> {
    let mut id2index = HashMap::new();
    for (index, entity) in entities.iter().enumerate() {
        id2index.insert(entity.id, index);
    }
    id2index
}

/// Reassign id of events to continuous sequence starting from 0.
/// As this makes handling of events a lot easier.
///
/// Returns the mapping from original id to the new id.
fn reassign_event_id(events: &mut [PatternEvent]) -> HashMap<usize, usize> {
    let mut id2index = HashMap::new();
    for (index, event) in events.iter_mut().enumerate() {
        id2index.insert(event.id, index);
        event.id = index;
    }
    id2index
}

fn parse_events(
    events_json: &[Value],
    entity_id2idx: &HashMap<usize, usize>,
    entities: &[PatternEntity],
) -> Result<Vec<PatternEvent>, PatternParsingError> {
    if events_json.is_empty() {
        return Err(PatternParsingError::NoEventInPattern);
    }

    let mut events = vec![];
    for event in events_json {
        let id = event["ID"]
            .as_u64()
            .ok_or(PatternParsingError::KeyError("ID"))? as usize;

        let event_type = parse_event_type(event)?;

        let signature = event["Signature"].as_str().unwrap_or_default().to_string();
        if event_type == PatternEventType::Flow && !signature.is_empty() {
            warn!("Signature on pattern event of type Flow will be ignored");
        } else if event_type != PatternEventType::Flow && signature.is_empty() {
            warn!("Empty signature detected, the matching behavior is undefined");
        }

        let subject_id = event["SubjectID"]
            .as_u64()
            .ok_or(PatternParsingError::KeyError("SubjectID"))? as usize;
        let subject_idx = entity_id2idx
            .get(&subject_id)
            .ok_or(PatternParsingError::UndefinedEntityId(subject_id))?;

        let object_id = event["ObjectID"]
            .as_u64()
            .ok_or(PatternParsingError::KeyError("ObjectID"))? as usize;
        let object_idx = entity_id2idx
            .get(&object_id)
            .ok_or(PatternParsingError::UndefinedEntityId(object_id))?;

        events.push(PatternEvent {
            id,
            event_type,
            signature,
            subject: entities[*subject_idx].clone(),
            object: entities[*object_idx].clone(),
        });
    }

    Ok(events)
}

fn parse_event_type(event_json: &Value) -> Result<PatternEventType, PatternParsingError> {
    let event_type = event_json["Type"].as_str();
    let event_type = match event_type {
        None => {
            if let Some(freq) = event_json["Frequency"].as_u64() {
                if freq == 0 {
                    return Err(PatternParsingError::InvalidFrequency(freq as u32));
                }
                PatternEventType::Frequency(freq as u32)
            } else {
                PatternEventType::Default
            }
        }
        Some("Default") => PatternEventType::Default,
        Some("Frequency") => {
            let freq = event_json["Frequency"]
                .as_u64()
                .ok_or(PatternParsingError::KeyError("Frequency"))? as u32;
            if freq == 0 {
                return Err(PatternParsingError::InvalidFrequency(freq));
            }
            PatternEventType::Frequency(freq)
        }
        Some("Flow") => {
            if event_json["Frequency"].is_u64() {
                warn!("Frequency in flow event is unsupported for now, so this has no effect");
            }
            PatternEventType::Flow
        }
        Some(unknow_type) => {
            return Err(PatternParsingError::UnknownEventType(
                unknow_type.to_string(),
            ))
        }
    };
    Ok(event_type)
}

fn parse_order_relation(events: &[Value], event_id2index: &HashMap<usize, usize>) -> Result<OrderRelation, PatternParsingError> {
    let mut orel_edges = Vec::new();

    for event in events {
        let my_id = event["ID"]
            .as_u64()
            .ok_or(PatternParsingError::KeyError("ID"))? as usize;
        let my_idx = event_id2index[&my_id] as u32;

        if let Some(parents) = event["Parents"].as_array() {
            for parent in parents {
                let parent_id = parent
                    .as_u64()
                    .ok_or(PatternParsingError::KeyError("Parents"))?
                    as usize;
                let parent_idx = event_id2index[&parent_id] as u32;
                orel_edges.push((parent_idx + 1, my_idx + 1));
            }

            if parents.is_empty() {
                orel_edges.push((0, my_idx + 1));
            }
        } else {
            orel_edges.push((0, my_idx + 1));
        }
    }

    let graph: Graph<usize, ()> = Graph::from_edges(&orel_edges);

    Ok(graph.into())
}

#[cfg(test)]
mod tests {
    use crate::pattern::PatternEventType;

    use super::*;
    use serde_json::json;

    #[test]
    fn test_input_file_name_parsing() {
        assert_eq!(
            get_input_files("TTP11"),
            (
                "TTP11_node.json".to_string(),
                "TTP11_edge.json".to_string(),
                "TTP11_oRels.json".to_string()
            )
        );

        assert_eq!(
            get_input_files("TTP11_regex"),
            (
                "TTP11_regex_node.json".to_string(),
                "TTP11_regex_edge.json".to_string(),
                "TTP11_oRels.json".to_string()
            )
        );
    }

    #[test]
    fn test_sample_pattern() {
        let json_obj = json!({
            "Version": "0.2.0",
            "UseRegex": false,
            "Entities": [
                {
                    "ID": 0,
                    "Signature": "",
                },
                {
                    "ID": 1,
                    "Signature": "",
                },
                {
                    "ID": 2,
                    "Signature": "",
                }
            ],
            "Events": [
                {
                    "ID": 0,
                    "Signature": "aaa",
                    "SubjectID": 0,
                    "ObjectID": 1,
                    "Parents": []
                },
                {
                    "ID": 1,
                    "Signature": "bbb",
                    "SubjectID": 1,
                    "ObjectID": 2,
                    "Parents": [ 0 ]
                }
            ]
        });

        let pattern = parse_json(&json_obj).unwrap();

        assert!(!pattern.use_regex);
        assert_eq!(pattern.entities.len(), 3);

        let correct_events = [
            PatternEvent {
                id: 0,
                event_type: PatternEventType::Default,
                signature: "aaa".to_string(),
                subject: PatternEntity {
                    id: 0,
                    signature: "".to_string(),
                },
                object: PatternEntity {
                    id: 1,
                    signature: "".to_string(),
                },
            },
            PatternEvent {
                id: 1,
                event_type: PatternEventType::Default,
                signature: "bbb".to_string(),
                subject: PatternEntity {
                    id: 1,
                    signature: "".to_string(),
                },
                object: PatternEntity {
                    id: 2,
                    signature: "".to_string(),
                },
            },
        ];
        assert_eq!(pattern.events, correct_events);

        assert!(itertools::equal(pattern.order.get_roots(), [0]));
        assert!(itertools::equal(pattern.order.get_next(0), [1]));
        assert!(pattern.order.get_next(1).next().is_none());
    }

    #[test]
    fn test_parse_event_type() {
        assert_eq!(
            parse_event_type(&json!({})).unwrap(),
            PatternEventType::Default
        );
        assert_eq!(
            parse_event_type(&json!({"Frequency": 10})).unwrap(),
            PatternEventType::Frequency(10)
        );
        assert_eq!(
            parse_event_type(&json!({"Type": "Default", "Frequency": 10})).unwrap(),
            PatternEventType::Default
        );
        assert_eq!(
            parse_event_type(&json!({"Type": "Flow", "Frequency": 10})).unwrap(),
            PatternEventType::Flow
        );
        assert!(parse_event_type(&json!({"Type": "Dummy"})).is_err());
        assert!(parse_event_type(&json!({"Type": "Frequency"})).is_err());
        assert!(parse_event_type(&json!({"Frequency": 0})).is_err());
    }
}
