# Paper Experiments

This document describes how to reproduce the experiment results in our paper.

## Experiment Setup

### Requirements

We use Python scripts to automate the experiments. The experiment environment requires:

- Unix-like environment (tested on Ubuntu 18.04 and 22.04)
- Latest stable release of rust: To run IPMES+.
- Python >= 3.6.9: To run experiment automation script.
    - with pip installed
- Java (JDK) >= 11: To run IPMES and Siddhi.
- Apache Maven >= 3.6.0: To build IPMES an Siddhi.
- GNU Make >= 4.3: To build timingsubg
- `g++` >= 11.4.0: To build timingsubg

### Data Preparation

You can download the all the source codes and data (including souce code of compared tools, data graphs and behavioral patterns) needed for the experiments at TODO.

Extract the experiment data:

```shell
unzip IPMES_PLUS_EXP.zip
cd IPMES_PLUS_EXP
```

### Environment Setup

Install the python packages listed in `requirements.txt` in the provided archive. For Ubuntu/Debian:

```
sudo apt-get update
sudo apt-get install python3 python3-pip
pip3 install -r requirements.txt
```

It is recommended to install packages in python virtual environments like [conda](https://docs.anaconda.com/free/miniconda/index.html), [venv](https://docs.python.org/3/library/venv.html) or [virtualenv](https://virtualenv.pypa.io/en/latest/) to avoid package collisions.


## Experiment 1: Effectiveness of Frequency-type Event Patterns

This experiment demonstrate the necessity of **frequency-based** event patterns across different patterns on different data graphs.

Resource usage of this experiment:

- Peak Memory Usage: 60 GB
- Estimated CPU Time: 100 seconds

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

## Experiment 2: Effectiveness of Flow-type Event Patterns

This experiment demonstrate the necessity of **flow-based** event patterns across different patterns on different data graphs.

Resource usage of this experiment:

- Peak Memory Usage: 250 MB
- Estimated CPU Time: 15 seconds

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

## Experiment 3: Efficiency of Matching Low-level Attack Patterns

This experiment compares the efficiency of matching patterns without flow and frequency semantics across **IPMES+**, **IPMES**, **Timing**, and **Siddhi**.

Resource usage of this experiment:

- Peak Memory Usage: 76 GB
- Estimated CPU Time: 340 hours (some data points runs significantly slower than others, see the description below)

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

Some data points requires significantly more time or memory to collect. These data points are:

- timing on DARPA dataset (dd1-dd4) runs for 140 hours in total
- siddhi on DARPA dataset (dd1-dd4) runs for 192 hours in total
- dd4 dataset requires 76 GB of memory for timing to run

To avoid running those data points, you can apply the `-a` option with a list of application ids as the parameter. Similarly, to specify only a subset of data graphs, you can apply the `-g` option with a list of data graphs as the parameter. For example, the following command runs **IPMES+** and **Timing** with data graphs **attack** and **dd2**:

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

## Experiment 4: Join Layer Optimization

This experiment highlights the effectiveness of **sibling entity
sharing enforcement** for **IPMES+**. We have synthesized worst-case provenance graphs exclusively containing pattern instances for this experiment.

Resource usage of this experiment:

- Peak Memory Usage: 70 MB
- Estimated CPU Time: 1 seconds

The following command conducts the experiment automatically. The script will output a table that corresponds to **Figure 8** and **Figure 9** in the paper. For convenience, the script will also print out the command it is currently running.

```shell
python experiments.py --join
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
