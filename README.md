# IPMES

**IPMES** (**I**ncremental Behavioral **P**attern **M**atching Algorithm over the System Audit **E**vent **S**tream for APT Detection) is a system that performs incremental pattern matching over event streams.

Given a **provenance graph** $G$ and a **behaviroal pattern** $EP$, IPMES streamingly outputs subgraphs of $G$ that matches $EP$.

## Requirement

- Rust
  - [Installation](https://www.rust-lang.org/zh-TW/tools/install)

## Build from source

```bash
git clone https://github.com/XYFC128/IPMES.git
cd ipmes-rust/
cargo build --release
```

## Running

```bash
./target/release/ipmes-rust
```

### Usage

```
IPMES implemented in rust

Usage: ipmes-rust [OPTIONS] <PATTERN_FILE> <DATA_GRAPH>

Arguments:
  <PATTERN_FILE>  The path prefix of pattern's files, e.g. ../data/universal_patterns/SP12.json
  <DATA_GRAPH>    The path to the preprocessed data graph

Options:
  -w, --window-size <WINDOW_SIZE>  Window size (sec) [default: 1800]
  -h, --help                       Print help
  -V, --version                    Print version
```

#### Example
- `ipmes-rust -w 3600 ../data/universal_patterns/SP12.json ../data/preprocessed/attack.csv`
  - `-w 1800`: Set the window size to be `3600` seconds.
  - `SP12.json`: A pattern for the SPADE dataset. See [data/README.md](data/README.md) for more information.
  - `preprocessed/attack.csv`: Input data graph which is either generated from real-world log data, or is manually synthesized. See [data/README.md](data/README.md) for more information.

### Output Format
- A single line show the total number of matches
  - `Total number of matches: 8`, for [the above example](#example).

## Directory Structure

- `data/`: Example input data for the program
- `docs/`: Documentations
- `ipmes-rust`: Rust implementation of IPMES