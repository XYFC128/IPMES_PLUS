use std::fs::File;
use std::rc::Rc;
use csv::Reader;
use itertools::assert_equal;
use crate::input_edge::InputEdge;
use crate::pattern::Edge;
use super::*;

#[test]
/// Fail
fn test_check_edge_uniqueness_1() {
    let pattern_edge = Edge {
        id: 0,
        signature: "".to_string(),
        start: 0,
        end: 0,
    };
    let match_edges = vec![
        MatchEdge {
            input_edge: Rc::new(InputEdge {
                timestamp: 0,
                signature: "".to_string(),
                id: 1,
                start: 0,
                end: 0,
            }),
            matched: &pattern_edge,
        },
        MatchEdge {
            input_edge: Rc::new(InputEdge {
                timestamp: 0,
                signature: "".to_string(),
                id: 2,
                start: 0,
                end: 0,
            }),
            matched: &pattern_edge
        },
        MatchEdge {
            input_edge: Rc::new(InputEdge {
                timestamp: 0,
                signature: "".to_string(),
                id: 2,
                start: 0,
                end: 0,
            }),
            matched: &pattern_edge
        }
    ];

    assert_eq!(check_edge_uniqueness(&match_edges), false);
}

#[test]
/// Pass
fn test_check_edge_uniqueness_2() {
    let pattern_edge = Edge {
        id: 0,
        signature: "".to_string(),
        start: 0,
        end: 0,
    };
    let match_edges = vec![
        MatchEdge {
            input_edge: Rc::new(InputEdge {
                timestamp: 0,
                signature: "".to_string(),
                id: 1,
                start: 0,
                end: 0,
            }),
            matched: &pattern_edge,
        },
        MatchEdge {
            input_edge: Rc::new(InputEdge {
                timestamp: 0,
                signature: "".to_string(),
                id: 2,
                start: 0,
                end: 0,
            }),
            matched: &pattern_edge
        },
        MatchEdge {
            input_edge: Rc::new(InputEdge {
                timestamp: 0,
                signature: "".to_string(),
                id: 3,
                start: 0,
                end: 0,
            }),
            matched: &pattern_edge
        }
    ];

    assert_eq!(check_edge_uniqueness(&match_edges), true);
}

#[test]
fn test_merge_edge_id_map_1() {
    let num_edges = 5;
    let edge_id_map1 = vec![
        None,
        Some(3),
        Some(2),
        None,
        None
    ];

    let edge_id_map2 = vec![
        Some(1),
        None,
        None,
        None,
        Some(7)
    ];

    let ans = vec![
        Some(1),
        Some(3),
        Some(2),
        None,
        Some(7)
    ];

    assert_eq!(ans, merge_edge_id_map(&edge_id_map1, &edge_id_map2));
}

#[test]
fn test_merge_edge_id_map_2() {
    let num_edges = 5;
    let edge_id_map1 = vec![
        None,
        Some(3),
        None,
        None,
        None
    ];

    let edge_id_map2 = vec![
        Some(1),
        None,
        None,
        None,
        Some(7)
    ];

    let ans = vec![
        Some(1),
        Some(3),
        None,
        None,
        Some(7)
    ];

    assert_eq!(ans, merge_edge_id_map(&edge_id_map1, &edge_id_map2));
}