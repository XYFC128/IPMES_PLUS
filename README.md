# IPMES+

**IPMES+** is a system that performs incremental pattern matching over audit event streams (provenance graph). It is the successor of the original [IPMES](https://github.com/littleponywork/IPMES).

The core concept of the original IPMES involves decomposing a target behavioral pattern into multiple total-ordered subpatterns (**Preprocessing**), matching and reordering events (**Matching Layer**), composing events against these subpatterns (**Composition Layer**), and then combining subpattern matches into complete instances (**Join Layer**). **IPMES+** retains a similar architecture with key differences:

- Integrate **frequency** and **flow** semantics by extending event pattern types and merging the **Matching Layer** into the **Composition Layer** for efficient support.
- Enhancing event matching and state management through Shared Entity Filtration, Flow Contraction, and Sibling Entity Sharing Enforcement to reduce search space and state explosion.
- Port the prototype from Java to Rust for better memory control and locality.

The simplified flow chart of **IPMES+** is illustrated in the below figure.

![IPMES+ Flow Chart](docs/images/flowchart-IPMES+.png)

## Building

The only build dependencies of **IPMES+** is rust compiler and `cargo`, which can be install with the following commands on linux:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# vefify installation
rustc -V
```

Get the latest version of **IPMES+** and build it:

```bash
git clone https://github.com/XYFC128/IPMES_PLUS.git
cd IPMES_PLUS/
cargo build --release
```

The output binary is located in `target/release/ipmes-rust`. The first build will take longer due to downloading the dependencies.

## Command-line Syntax

```
IPMES implemented in rust

Usage: ipmes-rust [OPTIONS] <PATTERN_FILE> <DATA_GRAPH>

Arguments:
  <PATTERN_FILE>  The path to the pattern file in json format, e.g. data/universal_patterns/SP12.json
  <DATA_GRAPH>    The path to the preprocessed data graph (provenance graph) in csv format

Options:
  -w, --window-size <WINDOW_SIZE>  Window size (sec) [default: 1800]
  -s, --silent                     Enable silent mode will not print individual pattern matches
  -h, --help                       Print help
  -V, --version                    Print version
```

### Minimal Running Example

- `./target/release/ipmes-rust -w 1800 data/paper/behavioral_pattern.json data/paper/data_graph.csv`
    - `-w 1800`: Set the time window size to be `1800` seconds.
    - `data/paper/behavioral_pattern.json`: An example pattern used in our paper. See the section below for more information.
    - `data/paper/data_graph.csv`: Input data graph to search for pattern. See the section below for its format.

## Output Format

The program output for the [above example](#minimal-running-example) is shown below:

```
Pattern Match: <5.000, 11.000>[(1 -> 3), (3, 5), (4, 6)]
Total number of matches: 1
CPU time elapsed: 0.000108047 secs
Peak memory usage: 8604 kB
```

The output message contains:

- Zero or multiple lines of **Pattern Match**: Each entry of `Pattern Match` denotes a matched instance of the pattern such that they are in the following format: `<StartTime, EndTime>[MatchEventList...]`, where
    - **StartTime**: The timestamp of the earliest event of this match instance.
    - **EndTime**: The timestamp of the latest event of this match instance.
    - **MatchEventList**: A comma seperated list of the matched input events, whose index in this array corresponds to the pattern event they are matched to.
        - If the corresponding pattern event is a **flow event**, it will be in the format `(StartEntityID -> EndEntityID)`. In this example, the pattern event 0 is a flow pattern, and IPMES+ found the flow from entity 1 to entity 3 that matches the pattern event.
        - If the corresponding pattern is a **frequency event**, the match id format is `(EventID, ...)`. The numbers in the parentheses is a list of matched input event IDs for that frequency pattern event. In this example, the pattern event 1 is a frequency event, and input event 3 and 5 both match that frequency event.
        - If the corresponding pattern is a normal regex pattern, the match id is simply the ID of the matched input event.
- **Total number of matches**: The number of matched instances of the pattern on the data graph.
- **CPU time elapsed**: The CPU time spent for pattern matching.
- **Peak memory usage**: The maximum total system memory usage in kilobytes.

## Input Format

**IPMES+** takes 2 files as input: The **pattern description file** and the **data graph file**. **IPMES+** will search for pattern in the data graph.

### Data Graph File Format

The data graph is a streaming provenance graph where each line is an audit event. We use normal file in our experiments, but it can also be an UNIX PIPE to support ture streaming input. An event in provenance graph is associated with 2 entities: the subject that initiates the event and the object affected by the event.

An event is given by a CSV format in each line of the input data graph. The columns in this CSV format are: [`start_time`, `end_time`, `event_id`, `event_sig`, `subject_id`, `subject_sig`, `object_id`, `object_sig`], which represents:

- `start_time`: the event start time in seconds. e.g. `5.000`
- `end_time`:   the event end time in seconds. e.g. `10.000`
- `event_id`:   the numerical event id
- `event_sig`: the event signature string
- `subject_id`: the subject entity numerical id
- `subject_sig`: the subject signature string
- `object_id`:  the object numerical id
- `object_sig`: the object signature string

See `data/paper/data_graph.csv` for example.

### Pattern File Format

A pattern describes a subgraph of the data graph by specifying the signature of events and entities of the subgraph. **IPMES+** additionally support flow and frequency event pattern to match high-level event patterns. The format of pattern description file is in this [JSON](https://www.json.org) scheme:

```json
{
  "Version": "0.2.0",
  "UseRegex": true,
  "Entities": [
    {
      "ID": 1,
      "Signature": "Socket::ip::.*"
    },
    {
      "ID": 2,
      "Signature": "Process::name::.*"
    },
    {
      "ID": 3,
      "Signature": "File::path::/.*/crontabs/root"
    }
  ],
  "Events": [
    {
      "ID": 1,
      "Type": "Flow",
      "SubjectID": 1,
      "ObjectID": 2,
      "Parents": []
    },
    {
      "ID": 2,
      "Signature": "read",
      "Type": "Frequency",
      "Frequency": 2,
      "SubjectID": 3,
      "ObjectID": 2,
      "Parents": [
        1
      ]
    },
    {
      "ID": 3,
      "Signature": "write",
      "Frequency": 2,
      "SubjectID": 2,
      "ObjectID": 3,
      "Parents": [
        1
      ]
    }
  ]
}
```

- `Version`: the version of the pattern format, the latest version is `0.2.0`.
- `UseRegex`: the `Signature` in this pattern is supposed to be treated as regex expressions. We use the regex crate to handle regex expresions, the supported regex syntax can be found [here](https://docs.rs/regex/latest/regex/#syntax).
- `Entities`: an array of **Pattern Entity Object**.
- `Events`: an array of **Pattern Event Object**.

**Pattern Entity Object**:

- `ID`: the unique id of this pattern entity.
- `Signature` the signature of this pattern entity. It will match input entities in data graph with the same signature. 

**Pattern Event Object**:

- `ID`: the unique id of this pattern event.
- `Type`: `Default`, `Frequency` or `Flow`.
    - `Default`: The default event pattern that matches the input event with the signature specified in `Signature`. If `UseRegex` is set to `true`, the signature will be treated as a regex expression to match the signatures of input events in the data graph. Can ignore `Type` for default event pattern.
    - `Frequency`: Similar to the default event pattern except it must be matched $f$ times to count as a frequency pattern match (i.e. there must be at least $f$ events in data graph that matches the signature of this pattern event). The parameter $f$ is specifed by the `Frequency` attribute of this event.
    - `Flow`: This pattern event has no signature. It finds a flow from the entity matches its subject pattern to the entity matches its object pattern.
- `SubjectID`: the subject of this event. If 2 events are arise from the same subject, they share the subject id.
- `ObjectID`: the object of this event. If 2 events act on the same object, they share the object id.
- `Parents`: an array of pattern event id. This determins the dependency of a pattern event. The pattern event should be matched after all of its parents are matched.

## Directory Structure

- `data/`: Example input data for the program. Check [data/README.md](data/README.md) for more information.
- `docs/`: Documentations.
- `scripts/`: Helper scripts to carry out expriment and data preprocessing. Check [scripts/README.md](scripts/README.md) for more information.
- `src/`: Source codes of **IPMES+**. Document are written as code comments.
- `testcases/`: Test data for the experiments and code tests.

## License

- All the files in `data/` are licensed under [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/).
- The license for the source code of **IPMES+** is stated in `LICENSE`.

## Authors

- Hong-Wei Li (Research Center for Information Technology Innovation, Academia Sinica, Taiwan) <g6_7893000@hotmail.com>
- Ping-Ting Liu (Department of Computer Science, National Yang Ming Chiao Tung University, Taiwan) <xyfc128@gmail.com>
- Bo-Wei Lin (Department of Computer Science, National Yang Ming Chiao Tung University, Taiwan) <0800680274united@gmail.com>
- Yennun Huang (Research Center for Information Technology Innovation, Academia Sinica, Taiwan) <yennunhuang@citi.sinica.edu.tw>
