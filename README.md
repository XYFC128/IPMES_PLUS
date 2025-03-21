# IPMES

## Overview

* [Introduction (2 human-minute)](#introduction-2-human-minute)
* [Configuration and Installation (1 human-minute, 6 compute-minutes)](#configuration-and-installation-1-human-minute-6-compute-minutes)
    + [Dependency (3 compute-minutes)](#dependency-3-compute-minutes)
    + [Build IPMES+ (3 compute-minutes)](#build-ipmes-3-compute-minutes)
* [Reproduce and Validate Experiment Results (15 human-minutes, 7 compute-days)](#reproduce-and-validate-experiment-results-15-human-minutes-7-compute-days)
* [Execution / How to reuse beyond paper (10 human-minutes, 1 compute-minute)](#execution--how-to-reuse-beyond-paper-10-human-minutes-1-compute-minute)
* [Authors (1 human-minute)](#authors-1-human-minute)

## Introduction (2 human-minute)

**IPMES+** is the successor of the original [IPMES](https://github.com/littleponywork/IPMES), which was developed in 2023. The original IPMES was published in IEEE/IFIP International Conference on Dependable Systems and Networks (DSN) 2024, titled **IPMES: A Tool for Incremental TTP Detection over the System Audit Event Stream (Tool)**.

**IPMES+** is a system that performs incremental pattern matching over event streams.

The core concept of the original IPMES involves decomposing a target behavioral pattern into multiple total-ordered subpatterns (**Preprocessing**), matching and reordering events (**Matching Layer**), composing events against these subpatterns (**Composition Layer**), and then combining subpattern matches into complete instances (**Join Layer**). **IPMES+** retains a similar architecture with key differences:

- Integrate **frequency** and **flow** semantics by extending event pattern types and merging the **Matching Layer** into the **Composition Layer** for efficient support.
- Enhancing event matching and state management through Shared Entity Filtration, Flow Contraction, and Sibling Entity Sharing Enforcement to reduce search space and state explosion.
- Port the prototype from Java to Rust for better memory control and locality.

An overview of **IPMES+** is illustrated in the below figure.

![IPMES Flow Chart](docs/images/flowchart-IPMES+.png)

## Configuration and Installation (1 human-minute, 6 compute-minutes)

### Dependency (3 compute-minutes)

- Rust (rustc) >= 1.75.0

Install on Ubuntu/Debian:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# vefify installation
rustc -V
```

### Build IPMES+ (3 compute-minutes)

**IPMES+** can be built with a simple command:

```bash
# clone this repository to IPMES_PLUS/
cd IPMES_PLUS/
cargo build --release
```

The first build will take longer due to downloading the dependencies.

## Reproduce and Validate Experiment Results (15 human-minutes, 7 compute-days)

Please refer to [dsn_artifact_manual.md](docs/dsn_artifact_manual.md).

## Execution / How to reuse beyond paper (10 human-minutes, 1 compute-minute)

### Directory Structure

- `data/`: Example input data for the program.
- `docs/`: Documentations.
- `scripts/`: Helper scripts to carry out expriment and data preprocessing.
- `src/`: Source codes of **IPMES+**.
- `testcases/`: Test data for the experiments and code tests.

### Command-line Syntax

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

#### Example

- `./target/release/ipmes-rust -w 1800 data/paper/behavioral_pattern.json data/paper/data_graph.csv`
  - `-w 1800`: Set the time window size to be `1800` seconds.
  - `data/paper/behavioral_pattern.json`: An example pattern used in our paper. See [data/README.md](data/README.md) for more information.
  - `data/paper/data_graph.csv`: Input data graph to search for pattern. See [data/README.md](data/README.md) for its format.

### Input Format

See [data/README.md](data/README.md) for more information.

### Output Format

The program output for the [above example](#example) is shown below:

```
Pattern Match: <5.000, 11.000>[(1 -> 3), (3, 5), (4, 6)]
Total number of matches: 1
CPU time elapsed: 0.000108047 secs
Peak memory usage: 8604 kB
```

The output message means:

- **Pattern Match**: Each entry of `Pattern Match` denotes a matched instance of the pattern such that they are in the following format: `<StartTime, EndTime>[list of MatchIDs]`, where
    - **StartTime**: The timestamp of the earliest event of this match instance.
    - **EndTime**: The timestamp of the latest event of this match instance.
    - **MatchIDs**: The IDs of the matched input events, whose index in this array corresponds to the pattern event they are matched to.
        - If the corresponding pattern event is a **flow event**, it will be in the format `(StartEntityID -> EndEntityID)`. In this example, the pattern event 0 is a flow pattern, and IPMES+ found the flow from entity 1 to entity 3 that matches the pattern event.
        - If the corresponding pattern is a **frequency event**, the match id format is `(EventID, ...)`. The numbers in the parentheses is a list of matched input event IDs for that frequency pattern event. In this example, the pattern event 1 is a frequency event, and input event 3 and 5 both match that frequency event.
        - If the corresponding pattern is a normal regex pattern, the match id is simply the ID of the matched input event.
- **Total number of matches**: The number of matched instances of the pattern on the data graph.
- **CPU time elapsed**: The CPU time spent for pattern matching.
- **Peak memory usage**: The maximum total system memory usage in kilobytes.

## Authors (1 human-minute)

- Hong-Wei Li (Research Center for Information Technology Innovation, Academia Sinica, Taiwan) <g6_7893000@hotmail.com>
- Ping-Ting Liu (Department of Computer Science, National Yang Ming Chiao Tung University, Taiwan) <xyfc128@gmail.com>
- Bo-Wei Lin (Department of Computer Science, National Yang Ming Chiao Tung University, Taiwan) <0800680274united@gmail.com>
- Yennun Huang (Research Center for Information Technology Innovation, Academia Sinica, Taiwan) <yennunhuang@citi.sinica.edu.tw>
