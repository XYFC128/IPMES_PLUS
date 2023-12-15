use crate::pattern::{Event, Pattern};

#[derive(Debug)]
pub struct SubPattern<'a> {
    // sub-pattern id
    pub id: usize,
    pub events: Vec<&'a Event>,
}

impl<'a> SubPattern<'a> {}

// maybe we can set a maximum sub-pattern size
pub fn decompose(pattern: &Pattern) -> Vec<SubPattern> {
    let mut sub_patterns: Vec<SubPattern> = Vec::new();
    let mut parents: Vec<&Event> = Vec::new();
    for edge in &pattern.events {
        generate_sub_patterns(pattern, &edge, &mut parents, &mut sub_patterns);
    }

    let mut selected: Vec<SubPattern> = select_sub_patterns(pattern.events.len(), sub_patterns);
    for (id, x) in selected.iter_mut().enumerate() {
        x.id = id;
    }
    // this.TCQRelation = genRelations(selected); [reserved to "join_layer"]
    selected
}

fn generate_sub_patterns<'a>(
    pattern: &'a Pattern,
    edge: &'a Event,
    parents: &mut Vec<&'a Event>,
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

fn has_shared_node(edge: &Event, parents: &Vec<&Event>) -> bool {
    if parents.is_empty() {
        return true;
    }
    for parent in parents {
        let node_shared = if edge.subject == parent.subject || edge.subject == parent.object {
            true
        } else if edge.object == parent.subject || edge.object == parent.object {
            true
        } else {
            false
        };
        if node_shared {
            return true;
        }
    }
    false
}

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
    use crate::pattern::parser::LegacyParser;
    // use crate::sub_pattern::SubPattern;
    use super::*;
    use crate::pattern::spade::SpadePatternParser;

    #[test]
    fn test_gsp() {
        let parser = SpadePatternParser;
        let pattern = parser
            .parse(
                "../data/patterns/TTP9_node.json",
                "../data/patterns/TTP9_edge.json",
                "../data/patterns/TTP9_oRels.json",
            )
            .unwrap();

        let edge: &Event = &pattern.events[0];
        let mut parents: Vec<&Event> = Vec::new();
        let mut results: Vec<SubPattern> = Vec::new();
        generate_sub_patterns(&pattern, edge, &mut parents, &mut results);

        println!("{:#?}", pattern.events);
    }

    #[test]
    fn test_hsn() {
        let e1 = &Event {
            id: 0,
            signature: "a".to_string(),
            subject: 10,
            object: 19,
        };
        let e2 = &Event {
            id: 0,
            signature: "b".to_string(),
            subject: 9,
            object: 15,
        };
        let e3 = &Event {
            id: 0,
            signature: "c".to_string(),
            subject: 11,
            object: 13,
        };
        let e4 = &Event {
            id: 0,
            signature: "d".to_string(),
            subject: 10,
            object: 13,
        };

        // true
        let parents: Vec<&Event> = vec![e1, e3];
        assert!(has_shared_node(e4, &parents));

        // false
        let parents: Vec<&Event> = vec![e2];
        assert!(!has_shared_node(e4, &parents));

        // true
        let parents: Vec<&Event> = vec![e3];
        assert!(has_shared_node(e4, &parents));

        // true
        let parents: Vec<&Event> = vec![];
        assert!(has_shared_node(e4, &parents));
    }

    #[test]
    fn test_ssp() {
        let parser = SpadePatternParser;
        let pattern = parser
            .parse(
                "../data/patterns/TTP11_node.json",
                "../data/patterns/TTP11_edge.json",
                "../data/patterns/TTP11_oRels.json",
            )
            .unwrap();

        let edge: &Event = &pattern.events[0];
        let mut parents: Vec<&Event> = Vec::new();
        let mut results: Vec<SubPattern> = Vec::new();
        generate_sub_patterns(&pattern, edge, &mut parents, &mut results);

        let num_edges: usize = 4;
        let selected = select_sub_patterns(num_edges, results);

        println!("{:?}", selected);
    }

    #[test]
    fn test_cse() {
        let parser = SpadePatternParser;
        let pattern = parser
            .parse(
                "../data/patterns/TTP11_node.json",
                "../data/patterns/TTP11_edge.json",
                "../data/patterns/TTP11_oRels.json",
            )
            .unwrap();

        let edge: &Event = &pattern.events[0];
        let mut parents: Vec<&Event> = Vec::new();
        let mut results: Vec<SubPattern> = Vec::new();
        generate_sub_patterns(&pattern, edge, &mut parents, &mut results);

        let sub_pattern = &results[0];
        let is_selected = vec![false, false, true, true];
        assert!(!contains_selected_edge(&sub_pattern, &is_selected));

        let sub_pattern = &results[1];
        let is_selected = vec![false, false, true, true];
        assert!(!contains_selected_edge(&sub_pattern, &is_selected));

        let sub_pattern = &results[2];
        let is_selected = vec![false, false, true, true];
        assert!(contains_selected_edge(&sub_pattern, &is_selected));

        let sub_pattern = &results[3];
        let is_selected = vec![false, false, false, true];
        assert!(contains_selected_edge(&sub_pattern, &is_selected));
    }

    #[test]
    fn test_decompose() {
        let parser = SpadePatternParser;
        let pattern = parser
            .parse(
                "../data/patterns/TTP9_node.json",
                "../data/patterns/TTP9_edge.json",
                "../data/patterns/TTP9_oRels.json",
            )
            .unwrap();

        println!("{:#?}", decompose(&pattern));
    }
}
