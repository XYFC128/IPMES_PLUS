# IPMES

## Overview

* [Introduction (2 human-minute)](#introduction-2-human-minute)
* [Configuration and Installation (1 human-minute, 6 compute-minutes)](#configuration-and-installation-1-human-minute-6-compute-minutes)
    + [Dependency (3 compute-minutes)](#dependency-3-compute-minutes)
    + [Build IPMES+ (3 compute-minutes)](#build-ipmes-3-compute-minutes)
* [Reproduce and Validate Experiment Results (15 human-minutes, 7 compute-days)](#reproduce-and-validate-experiment-results-15-human-minutes-7-compute-days)
    + [Preparation (5 compute-minutes)](#preparation-5-compute-minutes)
    + [Effectiveness of frequency-type event patterns (3 compute-minutes)](#effectiveness-of-frequency-type-event-patterns-sec-ivb-table-iii-3-compute-minutes)
    + [Effectiveness of flow-type event patterns (1 compute-minutes)](#effectiveness-of-flow-type-event-patterns-sec-ivc-table-iv-1-compute-minutes)
    + [Efficiency of matching low-level attack patterns (6 compute-days)](#efficiency-of-matching-low-level-attack-patterns-sec-ivd-figure-8-figure-9-6-compute-days)
    + [Join layer optimization (1 compute-minutes)](#join-layer-optimization-sec-ive-table-v-1-compute-minutes)
* [Execution / How to reuse beyond paper (10 human-minutes, 1 compute-minute)](#execution-how-to-reuse-beyond-paper-10-human-minutes-1-compute-minute)

* [Authors (1 human-minute)](#authors-1-human-minute)

## Introduction (2 human-minute)

**IPMES+** is the successor of the original [IPMES](https://github.com/littleponywork/IPMES), which was developed in 2023. The original IPMES was published in IEEE/IFIP International Conference on Dependable Systems and Networks (DSN) 2024, titled **IPMES: A Tool for Incremental TTP Detection over the System Audit Event Stream (Tool)**.

**IPMES+** is a system that performs incremental pattern matching over event streams.

The core concept of the original IPMES involves decomposing a target behavioral pattern into multiple total-ordered subpatterns (**Preprocessing**), matching and reordering events (**Matching Layer**), composing events against these subpatterns (**Composition Layer**), and then combining subpattern matches into complete instances (**Join Layer**). **IPMES+** retains a similar architecture with key differences:

- Integrate **frequency** and **flow** semantics by extending event pattern types and merging the **Matching Layer** into the **Composition Layer** for efficient support.
- Enhancing event matching and state management through Shared Entity Filtration, Flow Contraction, and Sibling Entity Sharing Enforcement to reduce search space and state explosion.
- Port the prototype from Java to Rust for better memory control and locality.

An overview of **IPMES+** is illustrated in the below figure.

![IPMES Flow Chart](images/flowchart-IPMES+.png)

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

This section describes how to reproduce the experiment results in our paper.

### Preparation (5 compute-minutes)

#### Experiment environment

We use a single Python script `experiments.py` to automate all the experiments. The experiment environment requires:

- RAM >= 75 GB
    - Running experiments on SPADE requires <= 10 GB.
    - Running experiments on DARPA requires <= 75 GB.
- Unix-like environment (tested on Ubuntu 18.04 and 22.04)
    - GNU bash >= 4.4.20
- Python >= 3.6.9 (with pip installed)
    - The Python packages listed in `./requirements.txt`.

For Ubuntu/Debian:

```shell
sudo apt-get update
sudo apt-get install python3 python3-pip
cd ipmes-rust
pip3 install -r requirements.txt
```

It is recommended to install packages in python virtual environments like [conda](https://docs.anaconda.com/free/miniconda/index.html), [venv](https://docs.python.org/3/library/venv.html) or [virtualenv](https://virtualenv.pypa.io/en/latest/) to avoid package collisions.

To conduct experiments involving **IPMES** and **Siddhi**, it additionally requires:

- Java (JDK) >= 11
- Apache Maven >= 3.6.0

To conduct experiments involving **Timing**, it additionally requires:
- GNU Make >= 4.3
- `g++` >= 11.4.0

The RAM requirement is high because when matching the behavioral pattern `DP5` on the data graph `dd4`, there would be an exploding number of matched instances. Except that particular combination of behavioral pattern and data graph, the other experiments should be runnable on a personal computer with 32 GB of memory.

#### Source codes and data

All the source codes and data (including data graphs and behavioral patterns) needed for the experiments are contained in the zip file `IPMES_PLUS_EXP.zip`. Before starting the experiments, please run

```shell
unzip IPMES_PLUS_EXP.zip
cd IPMES_PLUS_EXP
```

#### Regarding `experiments.py`

The script `experiments.py` supports to run an experiment multiple times, and also supports pre-run experiments for CPU warm-up. For more information, please refer to

```
python experiments.py -h
```


### Effectiveness of frequency-type event patterns (Sec. IV.B, Table III) (3 compute-minutes)

This experiment demonstrate the necessity of **frequency-based** event patterns across different patterns on different data graphs.

The following command conducts the experiment automatically. The script will output a table similar to **Table III** in the paper. For convenience, the script will also print out the command it is currently running.

```shell
python experiments.py --freq
```

Example output:

```
*** Building applications... ***
*** Building finished. ***
Running: `IPMES_PLUS/target/release/ipmes-rust IPMES_PLUS/data/universal_patterns/SP2_regex.json data_graphs/attack_raw.csv -w 1800 --silent`
Run 1 / 1 ...
Total number of matches: 1058
CPU time elapsed: 0.932406222 secs
Peak memory usage: 68608 kB

Running: `IPMES_PLUS/target/release/ipmes-rust IPMES_PLUS/data/freq_patterns/SP2_regex.json data_graphs/attack_raw.csv -w 1800 --silent`
Run 1 / 1 ...
...
  Pattern  Found Ins.  CPU Time (sec)  Peak Memory (MB)
      SP2        1058        0.932406         67.000000
      SP3      195000       77.508169      59713.730469
      SP4           0        1.212346        155.000000
      SP6           9        0.954729         67.000000
      SP7       53218        2.302216         67.000000
     SP10         993        0.969664         67.000000
     SP11          36        0.936476         67.000000
 SP2_freq          25        0.886327         67.000000
 SP3_freq           1        1.037059         67.000000
 SP4_freq           0        0.979137         67.000000
 SP6_freq           4        0.953199         67.000000
 SP7_freq         419        1.043491         67.000000
SP10_freq           9        0.932143         67.000000
SP11_freq           9        0.935679         67.000000
```

### Effectiveness of flow-type event patterns (Sec. IV.C, Table IV) (1 compute-minutes)

This experiment demonstrate the necessity of **flow-based** event patterns across different patterns on different data graphs.

The following command conducts the experiment automatically. The script will output a table similar to **Table IV** in the paper. For convenience, the script will also print out the command it is currently running.

```shell
python experiments.py --flow
```

Example output:

```
*** Building applications... ***
*** Building finished. ***
Running: `IPMES_PLUS/target/release/ipmes-rust IPMES_PLUS/data/universal_patterns/SP3.json modified_data_graphs/attack.csv -w 1800 --silent`
Run 1 / 1 ...
Total number of matches: 0
CPU time elapsed: 0.837500926 secs
Peak memory usage: 68608 kB

Running: `IPMES_PLUS/target/release/ipmes-rust IPMES_PLUS/data/flow_patterns/SP3.json modified_data_graphs/attack.csv -w 1800 --silent`
Run 1 / 1 ...
Total number of matches: 2
CPU time elapsed: 2.37848033 secs
Peak memory usage: 72004 kB

Running: `IPMES_PLUS/target/release/ipmes-rust IPMES_PLUS/data/universal_patterns/DP3.json modified_data_graphs/dd3.csv -w 1000 --silent`
Run 1 / 1 ...
Total number of matches: 0
CPU time elapsed: 3.822604907 secs
Peak memory usage: 68608 kB

Running: `IPMES_PLUS/target/release/ipmes-rust IPMES_PLUS/data/flow_patterns/DP3.json modified_data_graphs/dd3.csv -w 1000 --silent`
Run 1 / 1 ...
Total number of matches: 4
CPU time elapsed: 10.012387877 secs
Peak memory usage: 249008 kB

 Pattern  Found Ins.  CPU Time (sec)  Peak Memory (MB)
     SP3           0        0.837501         67.000000
     DP3           0        3.822605         67.000000
SP3_flow           2        2.378480         70.316406
DP3_flow           4       10.012388        243.171875
```

### Efficiency of matching low-level attack patterns (Sec. IV.D, Figure 8, Figure 9) (6 compute-days)

This experiment compares the efficiency of matching patterns without flow and frequency semantics across **IPMES+**, **IPMES**, **Timing**, and **Siddhi**.

The following command conducts the experiment automatically. The script will output a table that corresponds to **Figure 8** and **Figure 9** in the paper. For convenience, the script will also print out the command it is currently running.

```shell
python experiments.py
```

Example output:

```
*** Building applications... ***
*** Building finished. ***
Running: `IPMES_PLUS/target/release/ipmes-rust IPMES_PLUS/data/universal_patterns/SP1_regex.json data_graphs/attack.csv -w 1800 --silent`
Run 1 / 1 ...
...
CPU Time (sec)
Dataset   ipmes+     ipmes      timing    siddhi
 attack 0.827590  9.579167   55.618900 15.400833
    mix 1.169838 10.596667 1403.702995 21.330000
 benign 0.305944  6.814167   98.533342 33.084167
 ...
Memory (MB)
Dataset  ipmes+       ipmes     timing  siddhi
 attack    89.0 2064.000000 560.333333  1620.0
    mix    89.0 2064.000000 771.416667  1756.0
 benign    89.0 2194.666667 205.750000  1580.0
 ...
```

To closely inspect the performance of some particular applications, you can apply the `-a` option with a list of application ids as the parameter. Similarly, to specify only a subset of data graphs, you can apply the `-g` option with a list of data graphs as the parameter. For example, the following command runs **IPMES+** and **Timing** with data graphs **attack** and **dd2**:

```
python experiments.py -a "0,2" -g "attack,dd2"
```

which outputs:

```
*** Building applications... ***
*** Building finished. ***
Running: `IPMES_PLUS/target/release/ipmes-rust IPMES_PLUS/data/universal_patterns/SP1_regex.json data_graphs/attack.csv -w 1800 --silent`
Run 1 / 1 ...
...
Running: `timingsubg/rdf/bin/tirdf old_data_graphs/dd2.csv IPMES/data/universal_patterns/DP5_regex.json 1000 1 /dev/null timingsubg/rdf/data/universal_patterns/subpatterns/DP5_regex.json`
Run 1 / 1 ...
...
CPU Time (sec)
Dataset   ipmes+      timing
 attack 0.805636   55.572467
    dd2 3.956605 1420.066625
Memory (MB)
Dataset     ipmes+      timing
 attack  67.000000  560.333333
    dd2 176.110938 2031.250000
```

For more information regarding the options, please refer to

```
python experiments.py -h
```

### Join layer optimization (Sec. IV.E, Table V) (1 compute-minutes)

This experiment highlights the effectiveness of **sibling entity
sharing enforcement** for **IPMES+**. We have synthesized worst-case provenance graphs exclusively containing pattern instances for this experiment.

The following command conducts the experiment automatically. The script will output a table that corresponds to **Figure 8** and **Figure 9** in the paper. For convenience, the script will also print out the command it is currently running.

```shell
python experiments.py
```

Example output:

```
*** Building applications... ***
*** Building finished. ***
patching file mod.rs
Running: `./target/release/ipmes-rust ./data/universal_patterns/SP6_regex.json data/synthesized_graphs/DW10.csv -w 1800 --silent`
Run 1 / 1 ...
...
Running: `./target/release/ipmes-rust ./data/universal_patterns/SP6_regex.json data/synthesized_graphs/DW50.csv -w 1800 --silent`
Run 1 / 1 ...
Total number of matches: 50
CPU time elapsed: 0.005099256 secs
Peak memory usage: 68608 kB

Before optimization:
Synthesized Graph  Num Results  CPU Time (sec)  Peak Memory (MB)
             DW10           10        0.001232              67.0
             DW20           20        0.004339              67.0
             DW30           30        0.012072              67.0
             DW40           40        0.027526              67.0
             DW50           50        0.046382              67.0
After optimization:
Synthesized Graph  Num Results  CPU Time (sec)  Peak Memory (MB)
             DW10           10        0.000789              67.0
             DW20           20        0.001508              67.0
             DW30           30        0.002457              67.0
             DW40           40        0.003806              67.0
             DW50           50        0.005099              67.0
```

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
  <PATTERN_FILE>  The path to the pattern file in json format, e.g. ../data/universal_patterns/SP12.json
  <DATA_GRAPH>    The path to the preprocessed data graph (provenance graph) in csv format

Options:
  -w, --window-size <WINDOW_SIZE>  Window size (sec) [default: 1800]
  -s, --silent                     Enable silent mode will not print individual pattern matches
  -h, --help                       Print help
  -V, --version                    Print version
```

#### Example
- `./target/release/ipmes-rust -w 3600 ../data/universal_patterns/SP12.json ../data/preprocessed/attack.csv`
  - `-w 3600`: Set the window size to be `3600` seconds.
  - `SP12.json`: A pattern for the SPADE dataset. See [data/README.md](data/README.md) for more information.
  - `preprocessed/attack.csv`: Input data graph which is either generated from real-world log data, or is manually synthesized. See [data/README.md](data/README.md) for more information.

### Input Format
See [data/README.md](data/README.md) for more information.

### Output Format
The program output for the [above example](#example) is shown below:

```bash
Pattern Match: <1637229824.000, 1637229824.000>[7109945, 7106314, 7119497, 5109127]
Pattern Match: <1637226496.000, 1637226496.000>[7109521, 7106121, 7119501, 5109128]
Pattern Match: <1637233280.000, 1637233280.000>[7110213, 7106605, 7119505, 5109124]
Pattern Match: <1637244160.000, 1637244160.000>[7111035, 7107489, 7119499, 5109126]
Pattern Match: <1637254912.000, 1637254912.000>[7111777, 7108182, 7119498, 5109122]
Pattern Match: <1637251328.000, 1637251328.000>[7111529, 7107938, 7119500, 5109129]
Pattern Match: <1637262208.000, 1637262208.000>[7112244, 7108669, 7119496, 5109123]
Pattern Match: <1637258496.000, 1637258496.000>[7112003, 7108470, 7119503, 5109121]
Total number of matches: 8
CPU time elapsed: 0.728816059 secs
Peak memory usage: 4096 kB
```

The output message means:

- **Pattern Match**: Each entry of `Pattern Match` denotes a matched instance of the pattern such that they are in the following format: `<StartTime, EndTime>[list of MatchIDs]`, where
    - **StartTime**: The timestamp of the earliest event of this match instance.
    - **EndTime**: The timestamp of the latest event of this match instance.
    - **MatchIDs**: The IDs of the matched input events, whose index in this array corresponds to the pattern event they are matched to. In this example, the first `Pattern Match` entry contains an input event `7109945`, which is located at index 0, and hence it matches the pattern event with ID 0.
- **Total number of matches**: The number of matched instances of the pattern on the data graph.
- **CPU time elapsed**: The CPU time spent for pattern matching.
- **Peak memory usage**: The maximum heap allocation size in kilobytes.

## Authors (1 human-minute)

- Hong-Wei Li (Research Center for Information Technology Innovation, Academia Sinica, Taiwan) <g6_7893000@hotmail.com>
- Ping-Ting Liu (Department of Computer Science, National Yang Ming Chiao Tung University, Taiwan) <xyfc128@gmail.com>
- Bo-Wei Lin (Department of Computer Science, National Yang Ming Chiao Tung University, Taiwan) <0800680274united@gmail.com>
- Yennun Huang (Research Center for Information Technology Innovation, Academia Sinica, Taiwan) <yennunhuang@citi.sinica.edu.tw>