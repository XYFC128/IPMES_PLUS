pub mod darpa;
pub mod order_relation;
pub mod parser;
pub mod spade;
mod tests;

use std::{collections::HashSet, fs::File, io::Read};

use order_relation::OrderRelation;
use petgraph::Graph;
use serde_json::Value;

use self::parser::PatternParsingError;

#[derive(Debug, Eq, PartialEq)]
pub struct Event {
    pub id: usize,
    pub signature: String,
    pub subject: usize,
    pub object: usize,
}

#[derive(Debug)]
pub struct Pattern {
    pub use_regex: bool,
    pub events: Vec<Event>,
    pub order: OrderRelation,
    pub num_entities: usize,
}

impl Pattern {
    pub fn parse(pattern_file: &str) -> Result<Pattern, PatternParsingError> {
        let mut file = File::open(pattern_file)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        let json_obj: Value = serde_json::from_slice(&content)?;

        Pattern::parse_json(&json_obj)
    }

    fn parse_json(json_obj: &Value) -> Result<Pattern, PatternParsingError> {
        let use_regex = json_obj["UseRegex"]
            .as_bool()
            .ok_or(PatternParsingError::KeyError("UseRegex"))?;
        let events_json = json_obj["Events"]
            .as_array()
            .ok_or(PatternParsingError::KeyError("Events"))?;

        let events = Pattern::parse_events(events_json)?;
        let order = Pattern::parse_order_relation(&events_json)?;
        let num_entities = Pattern::count_entities(&events);

        Ok(Pattern {
            use_regex,
            events,
            order,
            num_entities,
        })
    }

    fn parse_events(events_json: &[Value]) -> Result<Vec<Event>, PatternParsingError> {
        let mut events = vec![];
        for event in events_json {
            let id = event["ID"]
                .as_u64()
                .ok_or(PatternParsingError::KeyError("ID"))? as usize;
            let signature = event["Signature"]
                .as_str()
                .ok_or(PatternParsingError::KeyError("Signature"))?
                .to_string();
            let subject_id = event["SubjectID"]
                .as_u64()
                .ok_or(PatternParsingError::KeyError("SubjectID"))?
                as usize;
            let object_id = event["ObjectID"]
                .as_u64()
                .ok_or(PatternParsingError::KeyError("ObjectID"))?
                as usize;
            events.push(Event {
                id,
                signature,
                subject: subject_id,
                object: object_id,
            });
        }

        Ok(events)
    }

    fn count_entities(events: &[Event]) -> usize {
        let mut entity_ids = HashSet::new();
        for event in events {
            entity_ids.insert(event.subject);
            entity_ids.insert(event.object);
        }
        entity_ids.len()
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

        Ok(OrderRelation { graph })
    }
}
