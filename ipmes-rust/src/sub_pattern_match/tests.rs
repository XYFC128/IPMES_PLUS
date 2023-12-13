use std::rc::Rc;
use crate::input_event::InputEvent;
use crate::pattern::Event;
use super::*;

#[test]
/// Fail
fn test_check_edge_uniqueness_1() {
    let pattern_edge = Event {
        id: 0,
        signature: "".to_string(),
        subject: 0,
        object: 0,
    };
    let match_edges = vec![
        MatchEvent {
            input_event: Rc::new(InputEvent {
                timestamp: 0,
                signature: "".to_string(),
                id: 1,
                subject: 0,
                object: 0,
            }),
            matched: &pattern_edge,
        },
        MatchEvent {
            input_event: Rc::new(InputEvent {
                timestamp: 0,
                signature: "".to_string(),
                id: 2,
                subject: 0,
                object: 0,
            }),
            matched: &pattern_edge
        },
        MatchEvent {
            input_event: Rc::new(InputEvent {
                timestamp: 0,
                signature: "".to_string(),
                id: 2,
                subject: 0,
                object: 0,
            }),
            matched: &pattern_edge
        }
    ];

    assert_eq!(check_edge_uniqueness(&match_edges), false);
}

#[test]
/// Pass
fn test_check_edge_uniqueness_2() {
    let pattern_edge = Event {
        id: 0,
        signature: "".to_string(),
        subject: 0,
        object: 0,
    };
    let match_edges = vec![
        MatchEvent {
            input_event: Rc::new(InputEvent {
                timestamp: 0,
                signature: "".to_string(),
                id: 1,
                subject: 0,
                object: 0,
            }),
            matched: &pattern_edge,
        },
        MatchEvent {
            input_event: Rc::new(InputEvent {
                timestamp: 0,
                signature: "".to_string(),
                id: 2,
                subject: 0,
                object: 0,
            }),
            matched: &pattern_edge
        },
        MatchEvent {
            input_event: Rc::new(InputEvent {
                timestamp: 0,
                signature: "".to_string(),
                id: 3,
                subject: 0,
                object: 0,
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

    assert_eq!(ans, merge_event_id_map(&edge_id_map1, &edge_id_map2));
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

    assert_eq!(ans, merge_event_id_map(&edge_id_map1, &edge_id_map2));
}