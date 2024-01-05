# Pattern and Data Graph

## Pattern Format

```json
{
    "Version": "0.1",
    "UseRegex": false,
    "Events": [
        {
            "ID": 0,
            "Signature": "aaa",
            "SubjectID": 123,
            "ObjectID": 456,
            "Parents": []
        },
        {
            "ID": 1,
            "Signature": "bbb",
            "SubjectID": 789,
            "ObjectID": 456,
            "Parents": [ 0 ]
        }
    ]
}
```

The pattern is represented in [JSON](https://www.json.org) format. The root object contains 3 keys:

- `Version`: the version of the pattern format, the latest version is `0.1.0`
- `UseRegex`: the `Signature` in this pattern is supposed to be treated as regex expressions. We use the regex crate to handle regex expresions, the supported regex syntax can be found [here](https://docs.rs/regex/latest/regex/#syntax).
- `Events`: an array of **Pattern Event Object**.

**Pattern Event Object**:

- `ID`: the unique id of this pattern event
- `Signature` the signature of this pattern event. It will match input events in data graph with the same signature. If `UseRegex` is set to `true`, than the signature will be treated as a regex expression to match the signatures of input events in the data graph.
- `SubjectID`: the subject of this event. If 2 events are arise from the same subject, they share the subject id.
- `ObjectID`: the object of this event. If 2 events act on the same object, they share the object id.
- `Parents`: an array of pattern event id. The pattern event should be matched after all of its parents are matched.

Current limitations:

- Pattern event id must be assigned in the range `[0, num_id)`, where `num_id` is the number of unique ids.

See the files in `universal_patterns`.

## Data Graph (Provenance Graph) Format

Data graphs are in csv format. The columns in the csv are: [`start_time`, `end_time`, `event_sig`, `eid`, `start_id`, `end_id`], which represents:

- `start_time`: the event start time
- `end_time`:   the event end time
- `event_sig`:  event signature, a signature is in the format: `{edge label}#{start node label}#{end node label}`
- `eid`:        edge id
- `start_id`:   id of the start node
- `end_id`:     id of the end node

A preprocessed provanence graph can be downloaded at [link](https://drive.google.com/file/d/1Iwydm_JaF1p2fls1KXazExIxfnjygUeY/view?usp=sharing).

## Directory Structure

- `darpa_pattern`: Patterns for the DARPA dataset.
- `pattern`: Patterns for the SPADE dataset
- `universal_patterns`: Patterns for both the DARPA dataset and the SPADE dataset, which are pre-processed into an uniform format.
