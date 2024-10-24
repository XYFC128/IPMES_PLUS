use std::vec;

use criterion::{criterion_group, criterion_main, Criterion};
use ipmes_rust::{
    pattern::{decompose, parser::parse_json, SubPattern},
    process_layers::{
        composition_layer::MatchInstance,
        JoinLayer,
    },
    universal_match_event::UniversalMatchEvent,
};
use itertools::{enumerate, Itertools};
use log::debug;
use serde_json::Value;

fn gen_match_instance_from_subpattern<'p>(sub_pattern: &SubPattern<'p>, set_time: u64) -> MatchInstance<'p> {
    let mut match_events = vec![];
    let mut match_entities = vec![];
    for match_event in &sub_pattern.events {
        match_events.push(UniversalMatchEvent {
            matched: *match_event,
            start_time: set_time,
            end_time: set_time,
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
        start_time: set_time,
        match_events: match_events.into_boxed_slice(),
        match_entities: match_entities.into_boxed_slice(),
        event_ids: event_ids.into_boxed_slice(),
        state_id: 0,
    }
}

fn gen_match_instances<'p>(sub_patterns: &Vec<SubPattern<'p>>, has_id: &[usize], set_time: u64) -> Vec<(u32, MatchInstance<'p>)> {
    let mut match_instances = vec![];
    for (id, sub_pattern) in enumerate(sub_patterns) {
        if has_id.binary_search(&id).is_err() {
            continue;
        }
        match_instances.push((
            sub_pattern.id as u32,
            gen_match_instance_from_subpattern(sub_pattern, set_time),
        ));
    }

    match_instances
}

fn run_join_layer() {
    let raw_pattern = r#"{"Version": "0.2.0", "UseRegex": true, "Entities": [{"ID": 0, "Signature": "0"}, {"ID": 1, "Signature": "1"}, {"ID": 2, "Signature": "2"}, {"ID": 3, "Signature": "3"}, {"ID": 4, "Signature": "4"}, {"ID": 5, "Signature": "5"}, {"ID": 6, "Signature": "6"}, {"ID": 7, "Signature": "7"}, {"ID": 8, "Signature": "8"}], "Events": [{"ID": 0, "Signature": "0", "SubjectID": 0, "ObjectID": 1, "Parents": []}, {"ID": 1, "Signature": "1", "SubjectID": 3, "ObjectID": 4, "Parents": [0]}, {"ID": 2, "Signature": "2", "SubjectID": 7, "ObjectID": 8, "Parents": [0]}, {"ID": 3, "Signature": "3", "SubjectID": 5, "ObjectID": 2, "Parents": [0]}, {"ID": 4, "Signature": "4", "SubjectID": 4, "ObjectID": 5, "Parents": [1]}, {"ID": 5, "Signature": "5", "SubjectID": 2, "ObjectID": 6, "Parents": [3]}, {"ID": 6, "Signature": "6", "SubjectID": 5, "ObjectID": 7, "Parents": [2]}, {"ID": 7, "Signature": "7", "SubjectID": 1, "ObjectID": 2, "Parents": [0]}]}"#;
    let json_obj: Value = serde_json::from_str(raw_pattern).expect("error reading json");
    let pattern = parse_json(&json_obj).expect("Failed to parse pattern");

    let windows_size = 1 * 1000;
    let sub_patterns = decompose(&pattern);

    debug!("sub_patterns: {:#?}", sub_patterns);

    let mut join_layer = JoinLayer::new((), &pattern, &sub_patterns, windows_size);


    /*
        Buffer structure:
                  0
               /     \
              1       2
            /   \   /   \
           3     4 5     6
        Below shows the expected number of joins occur in each buffer node, with format "(node1, node2): <all, success>"
            (3, 4): <8, 2>
            (5, 6): <4, 4>
            (1, 2): <2, 1>
        
        Expected complete pattern match: 1
        Rate of success joins: 50% (fail reason: order relation)

        Note that the above figures are for a single iteration. There are 100 iterations, and thus all numbers should
        be multiplied by 100.

    */
    let mut match_instances = vec![];
    // Mind that the end-of-loop "0" and "1" instances may be joined with beginning-of-loop "[0, 1]" instances,
    // if timestamps are not properly set.
    for i in 0..100 {
        match_instances.append(&mut gen_match_instances(&sub_patterns, &[0, 1], 9*i*windows_size + 100));
        match_instances.append(&mut gen_match_instances(&sub_patterns, &[2, 3], (9*i+1)*windows_size + 101));
        match_instances.append(&mut gen_match_instances(&sub_patterns, &[1, 3], (9*i+2)*windows_size + 1));
        match_instances.append(&mut gen_match_instances(&sub_patterns, &[0, 2], (9*i+3)*windows_size)); // subpattern 0 and 1 join fail
        match_instances.append(&mut gen_match_instances(&sub_patterns, &[1], (9*i+3)*windows_size + 1)); // subpattern 0 and 1 join success
        match_instances.append(&mut gen_match_instances(&sub_patterns, &[3], (9*i+3)*windows_size + 2)); // subpattern 0 and 1 join success

        for j in 0..5 {
            match_instances.append(&mut gen_match_instances(&sub_patterns, &[1], (9*i+j+4)*windows_size + 3 + 2*j));
            match_instances.append(&mut gen_match_instances(&sub_patterns, &[0], (9*i+j+4)*windows_size + 4 + 2*j));
        }
    }

    join_layer.run_isolated_join_layer(&mut match_instances);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("run join layer", |b| b.iter(|| run_join_layer()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
