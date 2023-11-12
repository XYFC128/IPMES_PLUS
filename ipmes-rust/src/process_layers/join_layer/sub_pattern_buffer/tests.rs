use std::fs::File;
use std::rc::Rc;
use csv::Reader;
use crate::input_edge::InputEdge;
use crate::pattern::Edge;
use super::*;
#[test]
/// shared node not shared between input nodes: Fail
fn test_try_merge_nodes1() {
    let max_node_id = 100;
    let tmp_sub_pattern = SubPattern { id: 0, edges: vec![] };
    let mut sub_pattern_buffer = SubPatternBuffer::new(
        0, 0, &tmp_sub_pattern, max_node_id, 0
    );

    let a = vec![
        (2, 19),
        (7, 20),
        (11, 9),
    ];

    let b = vec![
        (0, 17),
        (2, 22),
        (9, 11),
    ];

    let ans = None;

    let merged = sub_pattern_buffer.try_merge_nodes(&a, &b);
    assert_eq!(merged, ans);
}
use super::*;

#[test]
/// input node not unique: Fail
fn test_try_merge_nodes2() {
    let max_node_id = 100;
    let tmp_sub_pattern = SubPattern { id: 0, edges: vec![] };
    let mut sub_pattern_buffer = SubPatternBuffer::new(
        0, 0, &tmp_sub_pattern, max_node_id, 0
    );

    let a = vec![
        (2, 19),
        (7, 20),
        (11, 9),
    ];

    let b = vec![
        (0, 17),
        (25, 20),
    ];
    let ans = None;

    let merged = sub_pattern_buffer.try_merge_nodes(&a, &b);
    assert_eq!(merged, ans);
}

#[test]
/// Pass ("a" finished first)
fn test_try_merge_nodes3() {
    let max_node_id = 100;
    let tmp_sub_pattern = SubPattern { id: 0, edges: vec![] };
    let mut sub_pattern_buffer = SubPatternBuffer::new(
        0, 0, &tmp_sub_pattern, max_node_id, 0
    );

    let a = vec![
        (2, 19),
        (7, 20),
        (11, 9),
    ];

    let b = vec![
        (0, 17),
        (25, 27),
    ];
    let ans = vec![
        (0, 17),
        (2, 19),
        (7, 20),
        (11, 9),
        (25, 27)
    ];

    let merged = sub_pattern_buffer.try_merge_nodes(&a, &b);
    assert_ne!(merged, None);
    assert_eq!(merged.unwrap(), ans);
}

#[test]
/// Pass ("a" finished first)
fn test_try_merge_nodes4() {
    let max_node_id = 100;
    let tmp_sub_pattern = SubPattern { id: 0, edges: vec![] };
    let mut sub_pattern_buffer = SubPatternBuffer::new(
        0, 0, &tmp_sub_pattern, max_node_id, 0
    );

    let b = vec![
        (2, 19),
        (7, 20),
        (11, 9),
    ];

    let a = vec![
        (0, 17),
        (25, 27),
    ];
    let ans = vec![
        (0, 17),
        (2, 19),
        (7, 20),
        (11, 9),
        (25, 27)
    ];

    let merged = sub_pattern_buffer.try_merge_nodes(&a, &b);
    assert_ne!(merged, None);
    assert_eq!(merged.unwrap(), ans);
}

#[test]
/// pattern edge not shared: Fail
fn test_try_merge_edges1() {
    let num_edges1 = 2;
    let num_edges2 = 3;

    let num_edges = 20;
    let tmp_sub_pattern = SubPattern { id: 0, edges: vec![] };
    let mut sub_pattern_buffer = SubPatternBuffer::new(
        0, 0, &tmp_sub_pattern, 0, num_edges
    );

    let pattern_edge_ids1 = vec![1, 3];
    let pattern_edge_ids2 = vec![2, 4, 5];

    let input_edge_data1 = vec![
        (2, 0),
        (5, 0),
    ];

    let input_edge_data2 = vec![
        (2, 0),
        (10, 0),
        (12, 0),
    ];

    let mut pattern_edges1 = vec![];
    for id in pattern_edge_ids1 {
        pattern_edges1.push(gen_edge(id));
    }
    let mut pattern_edges2 = vec![];
    for id in pattern_edge_ids2 {
        pattern_edges2.push(gen_edge(id));
    }

    let match_edge1 = gen_match_edges(&pattern_edges1, &input_edge_data1);
    let match_edge2 = gen_match_edges(&pattern_edges2, &input_edge_data2);
    assert!(sub_pattern_buffer.try_merge_match_edges(&match_edge1, &match_edge2).is_none());
}

#[test]
/// Pass
fn test_try_merge_edges2() {
    let num_edges1 = 2;
    let num_edges2 = 3;

    let num_edges = 20;
    let tmp_sub_pattern = SubPattern { id: 0, edges: vec![] };
    let mut sub_pattern_buffer = SubPatternBuffer::new(
        0, 0, &tmp_sub_pattern, 0, num_edges
    );


    let pattern_edge_ids1 = vec![1, 3];
    let pattern_edge_ids2 = vec![1, 4, 5];

    let input_edge_data1 = vec![
        (2, 0),
        (5, 0),
    ];

    let input_edge_data2 = vec![
        (2, 0),
        (10, 0),
        (12, 0),
    ];

    let mut pattern_edges1 = vec![];
    for id in pattern_edge_ids1 {
        pattern_edges1.push(gen_edge(id));
    }
    let mut pattern_edges2 = vec![];
    for id in pattern_edge_ids2 {
        pattern_edges2.push(gen_edge(id));
    }

    let match_edge1 = gen_match_edges(&pattern_edges1, &input_edge_data1);
    let match_edge2 = gen_match_edges(&pattern_edges2, &input_edge_data2);

    let res = sub_pattern_buffer.try_merge_match_edges(&match_edge1, &match_edge2);
    assert!(!res.is_none());

    let ans_pattern_edge_ids = vec![1, 3, 4, 5];
    let mut ans_pattern_edges = vec![];
    for id in ans_pattern_edge_ids {
        ans_pattern_edges.push(gen_edge(id));
    }
    let ans_input_edge_data = vec![
        (2, 0),
        (5, 0),
        (10, 0),
        (12, 0),
    ];
    let ans_match_edge = gen_match_edges(&ans_pattern_edges, &ans_input_edge_data);
    let match_edge = res.unwrap();
    for i in 0..match_edge.len() {
        if !cmp_match_edge(&match_edge[i], &ans_match_edge[i]) {
            assert!(false);
        }
    }
}

fn gen_match_edges<'p>(pattern_edges: &'p Vec<Edge>, input_edge_data: &Vec<(u64, u64)>) -> Vec<MatchEdge<'p>> {
    let num_edges = input_edge_data.len();

    let mut input_edges = vec![];
    for (id, timestamp) in input_edge_data {
        input_edges.push(gen_input_edge(*id, *timestamp));
    }

    let mut match_edges = vec![];
    for i in 0..num_edges {
        match_edges.push(MatchEdge {
            input_edge: Rc::new(input_edges[i].clone()),
            matched: &pattern_edges[i],
        });
    }

    match_edges
}

fn gen_edge(id: usize) -> Edge {
    Edge {
        id,
        signature: "".to_string(),
        start: 0,
        end: 0,
    }
}

fn gen_input_edge(id: u64, timestamp: u64) -> InputEdge {
    InputEdge {
        timestamp,
        signature: "".to_string(),
        id,
        start: 0,
        end: 0,
    }
}

/// only check ids
fn cmp_match_edge(edge1: &MatchEdge, edge2: &MatchEdge) -> bool {
    if edge1.matched.id != edge2.matched.id { return false }
    else if edge1.input_edge.id != edge2.input_edge.id { return false }
    true
}

/// Below codes can be used to test "try_merge_nodes" for large testcases, but there might be some bugs.
fn get_line_u64(rdr: &mut Reader<File>) -> Option<Vec<u64>> {
    let mut line_u64 = Vec::new();
    let records = rdr.records().next()?;
    let binding = records.ok()?;
    let mut iter = binding.into_iter();

    while let Some(record) = iter.next() {
        line_u64.push(record.trim().parse().expect("parsing error"));
    }
    Some(line_u64)
}

fn fill(rdr: &mut Reader<File>, len: u64) -> Option<Vec<(u64, u64)>> {
    let mut a = Vec::new();
    for i in 0..len {
        let data = get_line_u64(rdr)?;
        a.push((data[0], data[1]));
    }
    Some(a)
}

fn check_same(a: &[(u64, u64)], b: &[(u64, u64)]) -> bool{
    if a.len() != b.len() {
        return false;
    }

    let (mut p1, mut p2) = (a.iter(), b.iter());
    while let (Some(x), Some(y)) = (p1.next(), p2.next()) {
        if x.ne(&y) {
            return false
        }
    }
    true
}

// #[test]
// fn test_try_merge_nodes_large(){
//     let max_node_id = 100;
//
//     let mut rdr = csv::ReaderBuilder::new()
//         .has_headers(false)
//         .from_path("testcases/sub_pattern_match/tests/t1.csv").unwrap();
//
//     let vec_len = get_line_u64(&mut rdr).expect("1st line error");
//     let a = fill(&mut rdr, vec_len[0]).expect("'a' error");
//     let b = fill(&mut rdr, vec_len[0]).expect("'b' error");
//
//     let mut ans_rdr = csv::ReaderBuilder::new()
//         .has_headers(false)
//         .from_path("testcases/sub_pattern_match/answers/a1.csv").unwrap();
//
//     let ans_len = get_line_u64(&mut ans_rdr).expect("get ans len error");
//     let ans = fill(&mut ans_rdr, ans_len[0]);
//
//
//     let mut used = vec![false; max_node_id];
//     if let Some(merged) = try_merge_nodes(&a, &b) {
//         assert_ne!(ans, None);
//         assert!(check_same(&merged, &ans.unwrap()));
//     } else {
//         assert_eq!(ans, None);
//     }
// }