# Pattern and Data Graph
## Pattern Graph

### Format
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

The format is the same as that of provanence graph (see [below](#data-graph-provenance-graph)), but `start_time` and `end_time` are not important here (the can be any value) and thus omitted; the `signature` column is the signature for the pattern edge.

Current limitations:
- Edge and node id must be assigned in the range `[0, num_id)`, where `num_id` is the number of unique ids.

## Data Graph (Provenance Graph)
### Format

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
