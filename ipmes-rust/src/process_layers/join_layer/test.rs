use std::arch::x86_64::_subborrow_u32;
use itertools::join;
use super::*;
use crate::pattern::parser::LegacyParser;
use crate::pattern::spade::SpadePatternParser;
use crate::sub_pattern::decompose;

// #[test]
// fn test_create_event_id_map() {
//
// }
// #[test]
// fn test_convert_entity_id_map() {
//
// }

#[test]
fn test_generate_sub_pattern_buffers() {
    let pattern = Pattern::parse("../data/universal_patterns/SP8_regex.json")
        .expect("Failed to parse pattern");

    let windows_size = 1800 * 1000;
    let sub_patterns = decompose(&pattern);
    println!("{:#?}", sub_patterns);
    println!("\n\n");
    let join_layer = JoinLayer::new((), &pattern, &sub_patterns, windows_size);
    println!("{:#?}", join_layer.sub_pattern_buffers);
}

#[test]
fn test_join_with_sibling() {}

#[test]
fn test_clear_expired() {}

#[test]
fn test_create_buffer_pair() {}
