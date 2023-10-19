// use crate::pattern_match::PatternMatch;
// use crate::sub_pattern::SubPattern;
// use crate::sub_pattern_match::SubPatternMatch;
// use std::cmp::Reverse;
//
// use crate::pattern::Pattern;
// use crate::process_layers::join_layer::TimeOrder::{FirstToSecond, SecondToFirst};
// use petgraph::adj::DefaultIx;
// use petgraph::dot::Config::NodeIndexLabel;
// use petgraph::graph::NodeIndex;
// use petgraph::visit::FilterNode;
// use serde_json::to_string;
// use std::collections::{BinaryHeap, HashMap, HashSet};
// use std::hash::Hash;
// use std::sync::atomic::Ordering::Relaxed;
//
// /*
//    It seems that "Huffman encoding" is not feasible: The sizes of Relation are the weights, but we need
//    to know "who is my sibling" when generating relations.
//
//    Really? Reflect on this more.
// */
//
// enum TimeOrder {
//     FirstToSecond,
//     SecondToFirst,
// }
// struct Relation {
//     common_nodes: Vec<usize>,
//     edge_orders: Vec<(usize, usize, TimeOrder)>,
// }
//
// impl Relation {
//     pub fn new() -> Self {
//         Self {
//             common_nodes: Vec::new(),
//             edge_orders: Vec::new(),
//         }
//     }
//     pub fn has_relation(&self, myself: &SubPatternMatch, sibling: &SubPatternMatch) -> bool {
//         // for common_node in self.common_nodes {
//         //     if !(myself.nodes.get(&common_node) && sibling.nodes.get(&common_node)) {
//         //         return false;
//         //     }
//         // }
//
//         for (id1, id2, time_order) in self.edge_orders {
//             match time_order {
//                 FirstToSecond => {
//                     if myself.matched_edges_table.get(&id1).unwrap().timestamp
//                         >= sibling.matched_edges_table.get(&id2).unwrap().timestamp
//                     {
//                         return false
//                     }
//                 }
//                 SecondToFirst => {
//                     if myself.matched_edges_table.get(&id1).unwrap().timestamp
//                         <= sibling.matched_edges_table.get(&id2).unwrap().timestamp
//                     {
//                         return false
//                     }
//                 }
//             }
//         }
//         true
//     }
// }
//
// // struct SubPatternBuffer<'p> {
// struct SubPatternBuffer {
//
//     id: usize,
//     sibling_id: usize,
//     node_id_list: HashSet<usize>,
//     // matched_buffer: BinaryHeap<Reverse<SubPatternMatch<'p>>>,
//     relation: Relation,
// }
//
// impl SubPatternBuffer {
//     pub fn new(id: usize, sibling_id: usize, sub_patterns: &Vec<SubPattern>) -> Self {
//         let mut node_id_list = HashSet::new();
//         for &edge in sub_patterns[id].edges {
//             node_id_list.insert(edge.start);
//             node_id_list.insert(edge.end);
//         }
//         Self {
//             id,
//             sibling_id,
//             node_id_list,
//             // matched_buffer: BinaryHeap::new(),
//             relation: Relation::new(),
//         }
//     }
//     pub fn init(
//         &mut self,
//         pattern: &Pattern,
//         sub_patterns: &Vec<SubPattern>,
//         sub_pattern_buffers: &Vec<SubPatternBuffer>,
//         distances_table: HashMap<(NodeIndex, NodeIndex), i32>,
//     ) {
//         self.generate_relations(pattern, sub_patterns, sub_pattern_buffers, distances_table);
//         todo!()
//     }
//
//     fn generate_relations(
//         &mut self,
//         pattern: &Pattern,
//         sub_patterns: &Vec<SubPattern>,
//         sub_pattern_buffers: &Vec<SubPatternBuffer>,
//         distances_table: HashMap<(NodeIndex, NodeIndex), i32>,
//     ) {
//         let mut common_nodes = Vec::new();
//         let mut edge_orders = Vec::new();
//
//         for i in 0..pattern.num_nodes {
//             if sub_pattern_buffers[self.id].node_id_list.contains(&i)
//                 && sub_pattern_buffers[self.sibling_id]
//                     .node_id_list
//                     .contains(&i)
//             {
//                 common_nodes.push(i);
//             }
//         }
//
//         for &edge1 in sub_patterns[self.id].edges {
//             for &edge2 in sub_patterns[self.sibling_id].edges {
//                 let id1 = NodeIndex::<DefaultIx>::new(edge1.id);
//                 let id2 = NodeIndex::<DefaultIx>::new(edge2.id);
//                 let distance_1_2 = distances_table.get(&(id1, id2)).unwrap();
//                 let distance_2_1 = distances_table.get(&(id2, id1)).unwrap();
//
//                 // "2" is "1"'s parent
//                 if distance_1_2 == &i32::MAX && distance_2_1 != &i32::MAX {
//                     edge_orders.push((edge1.id, edge2.id, SecondToFirst));
//                 } else if distance_1_2 != &i32::MAX && distance_2_1 == &i32::MAX {
//                     edge_orders.push((edge1.id, edge2.id, FirstToSecond));
//                 }
//             }
//         }
//         self.relation = Relation {
//             common_nodes,
//             edge_orders,
//         };
//     }
// }
//
// struct JoinLayer<'p, P>
// where
//     P: Iterator<Item = Vec<SubPatternMatch<'p>>>,
// {
//     prev_layer: P,
//     num_sub_patterns: usize,
//     // sub_pattern_buffers: Vec<SubPatternBuffer<'p>>,
//     distances_table: HashMap<(NodeIndex, NodeIndex), i32>,
//     patterns_matched: Vec<PatternMatch>,
// }
//
// impl<'p, P> JoinLayer<'p, P>
// where
//     P: Iterator<Item = Vec<SubPatternMatch<'p>>>,
// {
//     pub fn new(prev_layer: P, pattern: &'p Pattern, sub_patterns: &'p Vec<SubPattern>) -> Self {
//         Self {
//             prev_layer,
//             num_sub_patterns: sub_patterns.len(),
//             // sub_pattern_buffers: vec![SubPatternBuffer::new(), 2 * sub_patterns.len() - 1],
//             distances_table: pattern.order.calculate_distances().unwrap(),
//             patterns_matched: Vec::new(),
//             // relations: ,
//         }
//     }
//     fn add_sub_pattern_match(sub_pattern_match: SubPatternMatch) {
//         todo!()
//     }
// }
//
// impl<'p, P> Iterator for JoinLayer<'p, P>
// where
//     P: Iterator<Item = Vec<SubPatternMatch<'p>>>,
// {
//     type Item = PatternMatch;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         while self.patterns_matched.is_empty() {
//             let sub_pattern_match_batch = self.prev_layer.next()?;
//         }
//
//         todo!()
//     }
// }