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
    let events = parse_events(events_json, &entity_id2index, &entities)?;

    let order = parse_order_relation(events_json)?;
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

        let event_type = event["Type"].as_str().unwrap_or("Default");
        let event_type = match event_type {
            "Default" => PatternEventType::Default,
            "Frequency" => {
                let freq = event["Frequency"]
                    .as_u64()
                    .ok_or(PatternParsingError::KeyError("Frequency"))?
                    as u32;
                if freq <= 1 {
                    return Err(PatternParsingError::InvalidFrequency(freq));
                }
                PatternEventType::Frequency(freq)
            }
            "Flow" => PatternEventType::Flow,
            _ => {
                return Err(PatternParsingError::UnknownEventType(
                    event_type.to_string(),
                ))
            }
        };

        let signature = event["Signature"].as_str().unwrap_or_default().to_string();
        if event_type == PatternEventType::Flow && !signature.is_empty() {
            warn!("Signature on pattern event of type Flow will be ignored");
        } else if signature.is_empty() {
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

fn parse_order_relation(events: &[Value]) -> Result<OrderRelation, PatternParsingError> {
    let mut orel_edges = Vec::new();

    for event in events {
        let my_id = event["ID"]
            .as_u64()
            .ok_or(PatternParsingError::KeyError("ID"))? as u32;

        let parents = event["Parents"]
            .as_array()
            .ok_or(PatternParsingError::KeyError("Parents"))?;

        for parent in parents {
            let parent_id = parent
                .as_u64()
                .ok_or(PatternParsingError::KeyError("Parents"))?
                as u32;
            orel_edges.push((parent_id + 1, my_id + 1));
        }

        if parents.is_empty() {
            orel_edges.push((0, my_id + 1));
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
}
