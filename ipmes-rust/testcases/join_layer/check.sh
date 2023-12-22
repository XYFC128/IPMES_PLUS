#!/bin/bash

file1="SP9_answer.txt"
file2="SP9_instances_processed.txt"

while IFS= read -r line; do
    if ! grep -qF "$line" "$file2"; then
        echo "Line '$line' from file1 is not found in file2."
    fi
done < "$file1"

