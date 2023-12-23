#!/bin/bash

for i in {0..12}; do
  cargo r -- "../../data/universal_patterns/SP${i}_regex.json" ../../data/preprocessed/attack.csv \
  | grep "Total number of matches" >> attack_results.txt
done