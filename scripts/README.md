# Scripts

## `convert_data_graph.py`

Convert the old data graph format used in IPMES to the format used in IPMES+.

Usage:

```sh
python convert_data_graph.py < [old_data_graph] > [output_data_graph]
```

## `find_bugs.py`

Run IPMES+ on low level pattern and compare the result with the answers provided. If the output from IPMES+ is different from the answer, it will generate a small data graph that can reproduce the mismatch.

TODO: The answer is in an undocumented format.

Usage:

```
usage: find_bugs.py [-h] [-a ANS_FOLDER] [-d DATA_FOLDER] [-o OUT_DIR] pattern graph

Automatic bug finder

positional arguments:
  pattern               the name of pattern (ex. SP2_regex)
  graph                 the name of input graph (ex. attack)

options:
  -h, --help            show this help message and exit
  -a ANS_FOLDER, --ans-folder ANS_FOLDER
                        the path to answer folder (default: data/answer)
  -d DATA_FOLDER, --data-folder DATA_FOLDER
                        the path to data graph folder (default: data/preprocessed)
  -o OUT_DIR, --out-dir OUT_DIR
                        the path to output folder (default: data/temp)
```

Example Output:

```
$ python scripts/find_bugs.py SP2_regex attack
Indexing data graph...
['cargo', 'run', '--release', '--', 'data/universal_patterns/SP2_regex.json', 'data/preprocessed/attack.csv']
Among 0 match results, there are 0 results not in answer, and 131 answers not found in the results
Generating small input graph expecting 1 results to data/temp/expect_SP2_regex.csv
Use the command `cargo run --release -- data/universal_patterns/SP2_regex.json data/temp/expect_SP2_regex.csv` to run on the generated graph
```

## `gen_graph.py`

A script to generate random input graph from the given pattern.

Usage:

```
usage: gen_graph.py [-h] [-o OUTPUT_FILE] [-n N] [-s SEED] pattern_file

A script to generate random input graph from the given pattern

positional arguments:
  pattern_file

options:
  -h, --help            show this help message and exit
  -o OUTPUT_FILE, --output-file OUTPUT_FILE
                        The output folder of the upgrade result, left empty to use the same name as pattern
  -n N                  Number of output subgraphs
  -s SEED, --seed SEED  Random seed
```

## `pattern_upgrader.py`

Upgrade a pattern from an old version to the newest version (or the specified version). Since the new pattern format may not be backward compatible with the old formats, this script is added to help migrating between versions.

Usage:

```
usage: pattern_upgrader.py [-h] [-t TARGET_VERSION] [-o OUTPUT_DIR] pattern_files [pattern_files ...]

positional arguments:
  pattern_files

options:
  -h, --help            show this help message and exit
  -t TARGET_VERSION, --target-version TARGET_VERSION
  -o OUTPUT_DIR, --output-dir OUTPUT_DIR
                        The output folder of the upgrade result, left empty (default) if you intend to modify input files inplace
```

## `preprocess.py`

This program treats each line in stdin as a JSON object of an event.
It outputs the preprocessed event in csv format to stdout.

Example usage:

```sh
python preprocess.py < 12hour_attack_08_18.json > output.csv
```

The fields in the output csv:
- `start_time`: the event start time
- `end_time`:   the event end time
- `eid`:        edge id
- `event_sig`:  event signature
- `start_id`:   id of the start node
- `start_sig`:  start node signature
- `end_id`:     id of the end node
- `end_sig`:    end node signature


## `preprocess_darpa.py`

Similar to [`preprocessed.py`](#preprocesspy), but for data graphs in DARPA dataset.

## `run_all.py`

Run IPMES+ on all compatible combinations of the provided patterns and data graphs. It will collect the number of match results, the elapsed CPU-time and peak memory usage into a table. The table is beauty printed in stdout and saved as a csv file on the disk.

Usage:

```
usage: run_all.py [-h] [-d DATA_GRAPH] [-p PATTERN_DIR] [-o OUT_DIR] [-r RE_RUN] [--pre-run PRE_RUN] [--no-darpa] [--no-spade]

Run all pattern on all graph

options:
  -h, --help            show this help message and exit
  -d DATA_GRAPH, --data-graph DATA_GRAPH
                        the folder of data graphs (default: data/preprocessed/)
  -p PATTERN_DIR, --pattern-dir PATTERN_DIR
                        the folder of patterns (default: data/universal_patterns/)
  -o OUT_DIR, --out-dir OUT_DIR
                        the output folder (default: results/ipmes-rust/)
  -r RE_RUN, --re-run RE_RUN
                        Number of re-runs to measure CPU time (default: 1)
  --pre-run PRE_RUN     Number of runs before actual measurement (default: 0)
  --no-darpa            Do not run on DARPA (default: False)
  --no-spade            Do not run on SPADE (default: False)
```

Example output:

```
$ python3 scripts/run_all.py  --no-darpa -r 3
...(skipped)
Pattern Data Graph  Num Results  CPU Time (sec)  Peak Memory (MB)
    SP1     attack            1        0.823627         64.000000
    SP1        mix            1        1.117224         64.000000
    SP1     benign            0        0.320302         64.000000
    SP2     attack          131        0.775980         64.000000
    SP2        mix          106        1.066997         64.000000
    SP2     benign            0        0.287232         64.000000
    SP3     attack            2        0.890189         64.000000
    SP3        mix            0        1.243611         64.000000
    SP3     benign            0        0.354935         64.000000
...(skipped)
This table is saved to results/ipmes-rust/run_result.csv
```

## `to_universal_patterns.py`

Convert the old pattern format used in early IPMES to the new (universal) pattern format.

Usage:

1. Put the patterns for SPADE datasest in `data/patterns/`
2. Put the patterns for DARPA datasest in `data/darpa_patterns/`
3. Run this script: `python3 scripts/to_universal_patterns.py`
4. The converted patterns will be in `data/universal_patterns/`

The paths can be modified in the script's source code.

## `verify.py`

This program verify the preprocessed csv to ensure the number of columns is the same in every line. The input is read from stdin and verification results will be print to stdout.

Usage:

```sh
$ python3 scipts/verify.py < [data_graph.csv]
```
