use crate::pattern::{Pattern, PatternEvent};

/// Decomposed sub-pattern of a behavioral pattern
#[derive(Debug)]
pub struct SubPattern<'a> {
    /// sub-pattern id, given after decomposition
    pub id: usize,
    pub events: Vec<&'a PatternEvent>,
}

impl<'a> SubPattern<'a> {}

// maybe we can set a maximum sub-pattern size
/// Decompose the input behavioral pattern into disjoint sub-patterns.
pub fn decompose(pattern: &Pattern) -> Vec<SubPattern> {
    let mut sub_patterns: Vec<SubPattern> = Vec::new();
    let mut parents: Vec<&PatternEvent> = Vec::new();
    for edge in &pattern.events {
        generate_sub_patterns(pattern, edge, &mut parents, &mut sub_patterns);
    }

    let mut selected: Vec<SubPattern> = select_sub_patterns(pattern.events.len(), sub_patterns);
    for (id, x) in selected.iter_mut().enumerate() {
        x.id = id;
    }
    // this.TCQRelation = genRelations(selected); [reserved to "join_layer"]
    selected
}

/// A DFS search that enumerates all possible sub-patterns.
fn generate_sub_patterns<'a>(
    pattern: &'a Pattern,
    edge: &'a PatternEvent,
    parents: &mut Vec<&'a PatternEvent>,
    results: &mut Vec<SubPattern<'a>>,
) {
    // if "has_shared_node" is false, the sub-pattern would be disconnected (not allowed)
    if !has_shared_node(edge, parents) {
        return;
    }
    parents.push(edge);
    results.push(SubPattern {
        id: 0,
        events: parents.clone(),
    });
    for eid in pattern.order.get_next(edge.id) {
        generate_sub_patterns(pattern, &pattern.events[eid], parents, results);
    }
    parents.pop();
}

/// Check whether two events have any shared entity (node).
fn has_shared_node(edge: &PatternEvent, parents: &Vec<&PatternEvent>) -> bool {
    if parents.is_empty() {
        return true;
    }
    for parent in parents {
        let node_shared = edge.subject == parent.subject
            || edge.subject == parent.object
            || edge.object == parent.subject
            || edge.object == parent.object;
        if node_shared {
            return true;
        }
    }
    false
}

/// Select (heuristically) valid ones from all sub-patterns.
fn select_sub_patterns(num_edges: usize, mut sub_patterns: Vec<SubPattern>) -> Vec<SubPattern> {
    // sort in decreasing size
    sub_patterns.sort_by(|x, y| y.events.len().cmp(&x.events.len()));

    let mut selected_sub_patterns: Vec<SubPattern> = Vec::new();
    let mut is_edge_selected: Vec<bool> = vec![false; num_edges];
    for sub_pattern in sub_patterns.into_iter() {
        if contains_selected_edge(&sub_pattern, &is_edge_selected) {
            continue;
        }

        for edge in &sub_pattern.events {
            is_edge_selected[edge.id] = true;
        }
        selected_sub_patterns.push(sub_pattern);
    }

    selected_sub_patterns
}

/// Check whether an event (edge) is already selected.
fn contains_selected_edge(sub_pattern: &SubPattern, is_edge_selected: &[bool]) -> bool {
    for edge in &sub_pattern.events {
        if is_edge_selected[edge.id] {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::{Pattern, PatternEntity, PatternEventType};

    #[test]
    fn test_gsp() {
        let pattern = Pattern::parse("../data/universal_patterns/SP9.json").unwrap();

        let edge: &PatternEvent = &pattern.events[0];
        let mut parents: Vec<&PatternEvent> = Vec::new();
        let mut results: Vec<SubPattern> = Vec::new();
        generate_sub_patterns(&pattern, edge, &mut parents, &mut results);

        println!("{:#?}", pattern.events);
    }

    fn simple_pattern_event(signature: &str, subject: usize, object: usize) -> PatternEvent {
        PatternEvent {
            id: 0,
            signature: signature.to_string(),
            event_type: PatternEventType::Default,
            subject: PatternEntity {
                id: subject,
                signature: "".to_string(),
            },
            object: PatternEntity {
                id: object,
                signature: "".to_string(),
            },
        }
    }

    #[test]
    fn test_hsn() {
        let e1 = &simple_pattern_event("a", 10, 19);
        let e2 = &simple_pattern_event("b", 9, 15);
        let e3 = &simple_pattern_event("c", 11, 13);
        let e4 = &simple_pattern_event("d", 10, 13);

        // true
        let parents: Vec<&PatternEvent> = vec![e1, e3];
        assert!(has_shared_node(e4, &parents));

        // false
        let parents: Vec<&PatternEvent> = vec![e2];
        assert!(!has_shared_node(e4, &parents));

        // true
        let parents: Vec<&PatternEvent> = vec![e3];
        assert!(has_shared_node(e4, &parents));

        // true
        let parents: Vec<&PatternEvent> = vec![];
        assert!(has_shared_node(e4, &parents));
    }

    #[test]
    fn test_ssp() {
        let pattern = Pattern::parse("../data/universal_patterns/SP12.json").unwrap();

        let edge: &PatternEvent = &pattern.events[0];
        let mut parents: Vec<&PatternEvent> = Vec::new();
        let mut results: Vec<SubPattern> = Vec::new();
        generate_sub_patterns(&pattern, edge, &mut parents, &mut results);

        let num_edges: usize = 4;
        let selected = select_sub_patterns(num_edges, results);

        println!("{:?}", selected);
    }

    #[test]
    fn test_cse() {
        let pattern = Pattern::parse("../data/universal_patterns/SP12.json").unwrap();

        let edge: &PatternEvent = &pattern.events[0];
        let mut parents: Vec<&PatternEvent> = Vec::new();
        let mut results: Vec<SubPattern> = Vec::new();
        generate_sub_patterns(&pattern, edge, &mut parents, &mut results);

        let sub_pattern = &results[0];
        let is_selected = vec![false, false, true, true];
        assert!(!contains_selected_edge(sub_pattern, &is_selected));

        let sub_pattern = &results[1];
        let is_selected = vec![false, false, true, true];
        assert!(!contains_selected_edge(sub_pattern, &is_selected));

        let sub_pattern = &results[2];
        let is_selected = vec![false, false, true, true];
        assert!(contains_selected_edge(sub_pattern, &is_selected));

        let sub_pattern = &results[3];
        let is_selected = vec![false, false, false, true];
        assert!(contains_selected_edge(sub_pattern, &is_selected));
    }

    #[test]
    fn test_decompose() {
        let pattern = Pattern::parse("../data/universal_patterns/SP9.json").unwrap();

        println!("{:#?}", decompose(&pattern));
    }
}
