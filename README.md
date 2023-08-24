# IPMES

IPMES is a system to perform incremental pattern matching over event streams.

## Requirement

- Java >= 11
- Apache Maven >= 3.8.7

## Build from source

```bash
cd ipmes-java/
mvn compile
```

## Running

```bash
mvn exec:java -Dexec.args="[options] data_graph pattern_graph"
```

### Usage

```
usage: ipmes-java [-h] [-r] [--darpa] [-w WINDOWSIZE] [--debug] pattern_prefix data_graph

IPMES implemented in Java.

positional arguments:
  pattern_prefix         The path prefix of pattern's files, e.g. ./data/patterns/TTP11
  data_graph             The path to the preprocessed data graph

named arguments:
  -h, --help             show this help message and exit
  -r, --regex            Explicitly use regex matching. Default will automatically depend on the pattern prefix name (default: false)
  --darpa                We are running on DARPA dataset. (default: false)
  -w WINDOWSIZE, --window-size WINDOWSIZE
                         Time window size (sec) when joining. (default: 1800)
  --debug                Output debug information. (default: false)
```