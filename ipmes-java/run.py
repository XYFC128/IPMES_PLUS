#!/usr/bin/env python
import subprocess
from subprocess import Popen
import os
import sys
import re

dataset = 'darpa'
if len(sys.argv) > 1:
    dataset = sys.argv[1]

if dataset == 'darpa':
    from darpa import graphs
    pattern_filter = '^DP[0-9]+_regex\.json$'
    window_size = 1000
else:
    from spade import graphs
    pattern_filter = '^SP[0-9]+_regex\.json$'
    window_size = 1800

subprocess.run(['mvn', 'compile'], check=True)

os.makedirs('../results', exist_ok=True)

os.environ['MAVEN_OPTS'] = '-Xmx100G'

pattern_dir = '../data/universal_patterns'

filename_filter = re.compile(pattern_filter)
for pattern_file in os.listdir(pattern_dir):
    if not filename_filter.match(pattern_file):
        continue
    pattern_name = pattern_file.removesuffix('_regex.json')
    print(f'Running pattern {pattern_name}')
    pattern_path = os.path.join(pattern_dir, pattern_file)

    pdata: list[tuple] = []
    for graph in graphs:
        args = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="-w {window_size} {pattern_path} ../data/preprocessed/{graph}.csv"']
        print(args)
        out_file = open(f'../results/{pattern_name}_{graph}.out', 'w')
        err_file = open(f'../results/{pattern_name}_{graph}.err', 'w')
        pdata.append((Popen(args, stdout=out_file, stderr=err_file), out_file, err_file))
        
    for proc, out_file, err_file in pdata:
        proc.wait()
        out_file.close()
        err_file.close()

