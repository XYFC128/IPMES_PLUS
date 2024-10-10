use std::vec;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ipmes_rust::{
    pattern::{decompose, parser::parse_json, SubPattern},
    pattern_match,
    process_layers::{
        composition_layer::{match_instance, MatchInstance},
        JoinLayer,
    },
    universal_match_event::UniversalMatchEvent,
};
use itertools::{enumerate, Itertools};
use log::debug;
use rand::{seq::SliceRandom, SeedableRng}; // 0.6.5
use rand_chacha::{ChaCha20Rng, ChaChaRng};
use serde_json::Value;

fn gen_match_instance_from_subpattern<'p>(sub_pattern: &SubPattern<'p>) -> MatchInstance<'p> {
    let mut match_events = vec![];
    let mut match_entities = vec![];
    for match_event in &sub_pattern.events {
        match_events.push(UniversalMatchEvent {
            matched: *match_event,
            start_time: 0,
            end_time: 0,
            subject_id: match_event.subject.id as u64,
            object_id: match_event.object.id as u64,
            event_ids: vec![match_event.id as u64].into_boxed_slice(),
        });

        // We prescribe that the input event id is identical to the pattern event id.
        match_entities.push((match_event.subject.id as u64, match_event.subject.id as u64));
        match_entities.push((match_event.object.id as u64, match_event.object.id as u64));
    }

    match_entities = match_entities.into_iter().sorted().unique().collect_vec();

    let event_ids = match_events
        .iter()
        .flat_map(|x| x.event_ids.iter())
        .cloned()
        .sorted()
        .collect_vec();

    // Create match instances for each subpattern.
    MatchInstance {
        start_time: 0,
        match_events: match_events.into_boxed_slice(),
        match_entities: match_entities.into_boxed_slice(),
        event_ids: event_ids.into_boxed_slice(),
        state_id: 0,
    }
}

fn gen_match_instances<'p>(sub_patterns: &Vec<SubPattern<'p>>) -> Vec<(u32, MatchInstance<'p>)> {
    let mut match_instances = vec![];
    for sub_pattern in sub_patterns {
        match_instances.push((
            sub_pattern.id as u32,
            gen_match_instance_from_subpattern(sub_pattern),
        ));
    }

    match_instances
}

fn run_join_layer(fixed: bool) {
    let raw_pattern = r#"{"Version": "0.2.0", "UseRegex": true, "Entities": [{"ID": 0, "Signature": "0"}, {"ID": 1, "Signature": "1"}, {"ID": 2, "Signature": "2"}, {"ID": 3, "Signature": "3"}, {"ID": 4, "Signature": "4"}, {"ID": 5, "Signature": "5"}, {"ID": 6, "Signature": "6"}, {"ID": 7, "Signature": "7"}, {"ID": 8, "Signature": "8"}], "Events": [{"ID": 0, "Signature": "0", "SubjectID": 0, "ObjectID": 1, "Parents": []}, {"ID": 1, "Signature": "1", "SubjectID": 3, "ObjectID": 4, "Parents": [0]}, {"ID": 2, "Signature": "2", "SubjectID": 7, "ObjectID": 8, "Parents": [0]}, {"ID": 3, "Signature": "3", "SubjectID": 5, "ObjectID": 2, "Parents": [0]}, {"ID": 4, "Signature": "4", "SubjectID": 4, "ObjectID": 5, "Parents": [1]}, {"ID": 5, "Signature": "5", "SubjectID": 2, "ObjectID": 6, "Parents": [3]}, {"ID": 6, "Signature": "6", "SubjectID": 5, "ObjectID": 7, "Parents": [2]}, {"ID": 7, "Signature": "7", "SubjectID": 1, "ObjectID": 2, "Parents": [0]}]}"#;
    let json_obj: Value = serde_json::from_str(raw_pattern).expect("error reading json");
    let pattern = parse_json(&json_obj).expect("Failed to parse pattern");

    let windows_size = 1800 * 1000;
    let sub_patterns = decompose(&pattern);

    debug!("sub_patterns: {:#?}", sub_patterns);

    let mut join_layer = JoinLayer::new((), &pattern, &sub_patterns, windows_size);

    let mut match_instances = gen_match_instances(&sub_patterns);

    // Randomly shuffle match_instances
    let seed = 123456;
    let mut rng = ChaChaRng::seed_from_u64(seed);
    if !fixed {  
        rng = ChaChaRng::from_entropy();
    }
    match_instances.shuffle(&mut rng);

    join_layer.run_isolated_join_layer(&mut match_instances);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("run join layer", |b| b.iter(|| run_join_layer(false)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
