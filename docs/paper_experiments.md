# Paper Experiments

This document describes how to reproduce the experiment results in our paper.

## Experiment Setup

### Requirements

We use Python scripts to automate the experiments. The experiment environment requires:

- 32 GB of RAM
- Python >= 3.6.9
    - with pip installed

### Environment Setup

Install the python packages listed in `scripts/requirements.txt`. For Ubuntu/Debian:

```
sudo apt-get update
sudo apt-get install python3 python3-pip
cd IPMES_PLUS/
pip3 install -r scripts/requirements.txt
```

It is recommended to install packages in python virtual environments like [conda](https://docs.anaconda.com/free/miniconda/index.html), [venv](https://docs.python.org/3/library/venv.html) or [virtualenv](https://virtualenv.pypa.io/en/latest/) to avoid package collisions.

### Data Preparation

Our experiments use the preprocessed data graph as the input to IPMES. You can download the preprocessed provenance graph for our experiment at TODO.

Extract the file to a location of your choice. In the following example, we assume that the location of preprocessed data graph is located at `<root of source files>/data/`.

## Experiment 1: Effectiveness of Frequency-type Event Patterns

TODO

## Experiment 2: Effectiveness of Flow-type Event Patterns

TODO

## Experiment 3: Efficiency of Matching Low-level Attack Patterns

TODO

## Experiment 4: Join Layer Optimization

TODO
