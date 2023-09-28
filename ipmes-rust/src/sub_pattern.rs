use crate::pattern::order_relation::OrderRelation;
use crate::pattern::{Edge, Pattern};
use std::ops::Sub;

#[derive(Debug)]
pub struct SubPattern<'a> {
    pub id: usize,
    pub edges: Vec<&'a Edge>,
}

impl<'a> SubPattern<'a> {}

pub fn decompose(pattern: &Pattern) -> Vec<SubPattern> {
    let mut sub_patterns: Vec<SubPattern> = Vec::new();
    let mut parents: Vec<&Edge> = Vec::new();
    for edge in &pattern.edges {
        generate_sub_patterns(pattern, &edge, &mut parents, &mut sub_patterns);
    }

    let mut selected: Vec<SubPattern> = select_sub_patterns(pattern.edges.len(), sub_patterns);
    for (id, x) in selected.iter_mut().enumerate() {
        x.id = id;
    }
    // this.TCQRelation = genRelations(selected);
    selected
}

fn generate_sub_patterns<'a>(
    pattern: &'a Pattern,
    edge: &'a Edge,
    parents: &mut Vec<&'a Edge>,
    results: &mut Vec<SubPattern<'a>>,
) {
    if !has_shared_node(edge, parents) {
        return;
    }
    parents.push(edge);
    results.push(SubPattern {
        id: 0,
        edges: parents.clone(),
    });
    for eid in pattern.order.get_next(edge.id) {
        generate_sub_patterns(pattern, &pattern.edges[eid], parents, results);
    }
    parents.pop();
}

fn has_shared_node(edge: &Edge, parents: &Vec<&Edge>) -> bool {
    if parents.is_empty() {
        return true;
    }
    for parent in parents {
        let node_shared = if edge.start == parent.start || edge.start == parent.end {
            true
        } else if edge.end == parent.start || edge.end == parent.end {
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
    sub_patterns.sort_by(|x, y| y.edges.len().cmp(&x.edges.len()));

    let mut selected_sub_patterns: Vec<SubPattern> = Vec::new();
    let mut is_edge_selected: Vec<bool> = vec![false; num_edges];
    for sub_pattern in sub_patterns.into_iter() {
        if contains_selected_edge(&sub_pattern, &is_edge_selected) {
            continue;
        }

        for edge in &sub_pattern.edges {
            is_edge_selected[edge.id] = true;
        }
        selected_sub_patterns.push(sub_pattern);
    }

    selected_sub_patterns
}

fn contains_selected_edge(sub_pattern: &SubPattern, is_edge_selected: &[bool]) -> bool {
    for edge in &sub_pattern.edges {
        if is_edge_selected[edge.id] {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::pattern::parser::PatternParser;
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

        let edge: &Edge = &pattern.edges[0];
        let mut parents: Vec<&Edge> = Vec::new();
        let mut results: Vec<SubPattern> = Vec::new();
        generate_sub_patterns(&pattern, edge, &mut parents, &mut results);

        println!("{:#?}", pattern.edges);
    }

    #[test]
    fn test_hsn() {
        let e1 = &Edge {
            id: 0,
            signature: "a".to_string(),
            start: 10,
            end: 19,
        };
        let e2 = &Edge {
            id: 0,
            signature: "b".to_string(),
            start: 9,
            end: 15,
        };
        let e3 = &Edge {
            id: 0,
            signature: "c".to_string(),
            start: 11,
            end: 13,
        };
        let e4 = &Edge {
            id: 0,
            signature: "d".to_string(),
            start: 10,
            end: 13,
        };

        // true
        let parents: Vec<&Edge> = vec![e1, e3];
        assert!(has_shared_node(e4, &parents));

        // false
        let parents: Vec<&Edge> = vec![e2];
        assert!(!has_shared_node(e4, &parents));

        // true
        let parents: Vec<&Edge> = vec![e3];
        assert!(has_shared_node(e4, &parents));

        // true
        let parents: Vec<&Edge> = vec![];
        assert!(has_shared_node(e4, &parents));
    }

    #[test]
    fn test_ss() {
        let parser = SpadePatternParser;
        let pattern = parser
            .parse(
                "../data/patterns/TTP11_node.json",
                "../data/patterns/TTP11_edge.json",
                "../data/patterns/TTP11_oRels.json",
            )
            .unwrap();

        let edge: &Edge = &pattern.edges[0];
        let mut parents: Vec<&Edge> = Vec::new();
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

        let edge: &Edge = &pattern.edges[0];
        let mut parents: Vec<&Edge> = Vec::new();
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
