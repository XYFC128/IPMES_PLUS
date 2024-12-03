# IPMES+

IPMES+ is the successor of the original [IPMES](https://github.com/littleponywork/IPMES), which was developed by [Ping-Ting Liu](https://github.com/XYFC128) and [Yi-Chun Liao](https://github.com/Datou0718/) in 2023. The original IPMES was published in IEEE/IFIP International Conference on Dependable Systems and Networks (DSN) 2024, titled **IPMES: A Tool for Incremental TTP Detection over the System Audit Event Stream (Tool)**.

**IPMES+** is a system that performs incremental pattern matching over event streams.

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
  - `-w 3600`: Set the window size to be `3600` seconds.
  - `SP12.json`: A pattern for the SPADE dataset. See [data/README.md](data/README.md) for more information.
  - `preprocessed/attack.csv`: Input data graph which is either generated from real-world log data, or is manually synthesized. See [data/README.md](data/README.md) for more information.

### Output Format
- A single line show the total number of matches
  - `Total number of matches: 8`, for [the above example](#example).

## Directory Structure

- `data/`: Example input data for the program
- `docs/`: Documentations
- `ipmes-rust`: Rust implementation of IPMES

## Related Repositories

- [littleponywork/IPMES](https://github.com/littleponywork/IPMES) - the polished source code for DSN 2024 paper
- [Datou0718/IPMES](https://github.com/Datou0718/IPMES) - the original repository to hold IPMES source code.
