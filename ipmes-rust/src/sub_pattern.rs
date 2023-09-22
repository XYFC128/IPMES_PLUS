use std::ops::Sub;
use crate::pattern::{Edge, Pattern};
use crate::pattern::order_relation::OrderRelation;

pub struct SubPattern<'a> {
    pub id: usize,
    pub edges: Vec<&'a Edge>,
}

impl<'a> SubPattern<'a> {
}

pub fn decompose(pattern: &Pattern, order_relations: &OrderRelation) -> Vec<SubPattern> {
    let mut sub_patterns: Vec<SubPattern> = Vec::new();
    let mut parents: Vec<&Edge> = Vec::new();
    for edge in pattern.edges {
        generate_sub_patterns(pattern, order_relations, &edge, &mut parents, &mut sub_patterns);
    }

    let mut selected: Vec<SubPattern> = select_sub_patterns(pattern, sub_patterns);
    for (id, x) in selected.iter_mut().enumerate() {
        x.id = id;
    }
    // this.TCQRelation = genRelations(selected);
    selected
}

fn generate_sub_patterns(pattern: &Pattern, order_relations: &OrderRelation, edge: &Edge, parents: &mut Vec<&Edge>, results: &mut Vec<SubPattern>) {
    if !has_shared_node(edge, parents) {
        return;
    }
    parents.push(edge);
    results.push(SubPattern {id: 0, edges: Vec::from(parents)});
    for eid in order_relations.get_next(edge.id) {
        generate_sub_patterns(pattern, order_relations, &pattern.edges[eid], parents, results);
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

fn select_sub_patterns(pattern: &Pattern, mut sub_patterns: Vec<SubPattern>) -> Vec<SubPattern> {
    // sort in decreasing size
    sub_patterns.sort_by(|x, y| x.cmp(&y));

    let mut selected_sub_patterns: Vec<SubPattern> = Vec::new();
    let is_edge_selected: Vec<bool> = vec![false; pattern.edges.len()];
    for sub_pattern in sub_patterns.into_iter() {
        if contains_selected_edge(&sub_pattern, &is_edge_selected) {
            continue;
        }

        for edge in sub_pattern.edges {
            is_edge_selected[edge.id] = true;
        }
        selected_sub_patterns.push(sub_pattern);
    }

    selected_sub_patterns
}

fn contains_selected_edge(sub_pattern: &SubPattern, is_edge_selected: &[bool]) -> bool {
    for edge in sub_pattern.edges {
        if is_edge_selected[edge.id] {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    // use crate::sub_pattern::SubPattern;
    use super::*;

    #[test]
    fn test () {
    }
}