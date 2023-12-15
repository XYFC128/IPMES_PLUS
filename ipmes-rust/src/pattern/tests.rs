use serde_json::json;

use super::*;

#[test]
fn test_sample_pattern() {
    let json_obj = json!({
        "Version": "0.1",
        "UseRegex": false,
        "Events": [
            {
                "ID": 0,
                "Signature": "aaa",
                "SubjectID": 0,
                "ObjectID": 1,
                "Parents": []
            },
            {
                "ID": 1,
                "Signature": "bbb",
                "SubjectID": 1,
                "ObjectID": 2,
                "Parents": [ 0 ]
            }
        ]
    });

    let pattern = Pattern::parse_json(&json_obj).unwrap();

    assert_eq!(pattern.use_regex, false);
    assert_eq!(pattern.num_entities, 3);

    let correct_events = [
        Event {
            id: 0,
            signature: "aaa".to_string(),
            subject: 0,
            object: 1
        },
        Event {
            id: 1,
            signature: "bbb".to_string(),
            subject: 1,
            object: 2
        }
    ];
    assert_eq!(pattern.events, correct_events);

    assert!(itertools::equal(pattern.order.get_roots(), [0]));
    assert!(itertools::equal(pattern.order.get_next(0), [1]));
    assert!(pattern.order.get_next(1).next().is_none());
}