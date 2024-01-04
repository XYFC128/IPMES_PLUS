# IPMES

## Description

**IPMES** (**I**ncremental Behavioral **P**attern **M**atching Algorithm over the System Audit **E**vent **S**tream for APT Detection) is a system that performs incremental pattern matching over event streams.

Given a **provenance graph** $G$ and a **behaviroal pattern** $EP$, IPMES streamingly outputs subgraphs of $G$ that matches $EP$.

## Requirement

- Rust
  - [Installation](https://www.rust-lang.org/zh-TW/tools/install)

## Build from source

```bash
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

## Directory Structure

- `data/`: example input data for the program
    - `universal_patterns`: patterns for the DARPA dataset and the SPADE dataset, pre-processed into an uniform format
- `docs/`: Documentations
- `ipmes-rust`: Rust implementation of IPMES