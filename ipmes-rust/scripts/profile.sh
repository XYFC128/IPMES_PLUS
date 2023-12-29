#!/bin/bash

cargo build
cd Flamegraph
for i in {0..1}; do
  perf record --freq=997 --call-graph dwarf -q "../../data/universal_patterns/SP${i}_regex.json" ../../data/preprocessed/attack.csv
  perf script | ./stackcollapse-perf.pl > out.perf-folded
  ./flamegraph.pl out.perf-folded > perf.svg
done
