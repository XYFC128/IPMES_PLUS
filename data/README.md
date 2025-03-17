# Pattern and Data Graph

## Pattern Format

```json
{
  "Version": "0.2.0",
  "UseRegex": true,
  "Entities": [
    {
      "ID": 0,
      "Signature": "123"
    },
    {
      "ID": 1,
      "Signature": "456"
    }
  ],
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
      "ObjectID": 0,
      "Parents": [
        0
      ]
    }
  ]
}
```

The pattern is represented in [JSON](https://www.json.org) format. The root object contains 3 keys:

- `Version`: the version of the pattern format, the latest version is `0.2.0`.
- `UseRegex`: the `Signature` in this pattern is supposed to be treated as regex expressions. We use the regex crate to handle regex expresions, the supported regex syntax can be found [here](https://docs.rs/regex/latest/regex/#syntax).
- `Entities`: an array of **Pattern Entity Object**.
- `Events`: an array of **Pattern Event Object**.

**Pattern Entity Object**:

- `ID`: the unique id of this pattern entity.
- `Signature` the signature of this pattern entity. It will match input entities in data graph with the same signature. 

**Pattern Event Object**:

- `ID`: the unique id of this pattern event.
- `Signature` the signature of this pattern event. It will match input events in data graph with the same signature. If `UseRegex` is set to `true`, the signature will be treated as a regex expression to match the signatures of input events in the data graph.
- `SubjectID`: the subject of this event. If 2 events are arise from the same subject, they share the subject id.
- `ObjectID`: the object of this event. If 2 events act on the same object, they share the object id.
- `Parents`: an array of pattern event id. The pattern event should be matched after all of its parents are matched.

Current limitations:

- Pattern event id must be assigned in the range `[0, num_id)`, where `num_id` is the number of unique ids.

See the files in `universal_patterns` for more information.

## Data Graph (Provenance Graph) Format

t1, t2, event_id, event_sig, subject_id, subject_sig, object_id, object_sig

Data graphs are in csv format. The columns in the csv are: [`start_time`, `end_time`, `event_id`, `event_sig`, `subject_id`, `subject_sig`, `object_id`, `object_sig`], which represents:

- `start_time`: the event start time
- `end_time`:   the event end time
- `event_id`:   the event id
- `event_sig`: the event signature
- `subject_id`: the subject entity id
- `subject_sig`: the subject signature
- `object_id`:  the object id
- `object_sig`: the object signature

A preprocessed provanence graph can be downloaded at [link](https://drive.google.com/file/d/1dKsFX7NB5D85DGkLZdoh8qrgRMrfSbBk/view?usp=sharing).

## Directory Structure

- `darpa_pattern`: Patterns for the DARPA dataset.
- `patterns`: Patterns for the SPADE dataset.
- `universal_patterns`: Patterns for both the DARPA dataset and the SPADE dataset, which are pre-processed into an uniform format.
- `flow_patterns`: Patterns that support flow semantic.
- `freq_patterns`: Patterns that support frequency semantic.
- `paper`: The example data graph and the behavioral pattern shown in the published paper.
