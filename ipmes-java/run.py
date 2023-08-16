#!/usr/bin/env python
import subprocess
from subprocess import Popen
import os
import sys

dataset = 'darpa'
if len(sys.argv) > 1:
    dataset = sys.argv[1]

if dataset == 'darpa':
    from darpa import graphs, pattern_file
else:
    from spade import graphs, pattern_file

subprocess.run(['mvn', 'compile'], check=True)

if not os.path.exists('../results'):
    os.mkdir('../results')

os.environ['MAVEN_OPTS'] = '-Xmx100G'

for pattern_name, file_prefix in pattern_file:
    print(f'Running pattern {pattern_name}')
    pdata: list[tuple] = []
    for graph in graphs:
        if dataset == 'darpa':
            args = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="-w 1000 --darpa ../data/darpa_patterns/{file_prefix} ../data/preprocessed/{graph}.csv"']
        else:
            args = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="-w 1800 ../data/patterns/{file_prefix} ../data/preprocessed/{graph}.csv"']
        # args = ['bash', '-c', 'time -p -- mvn -q exec:java -Dexec.args="../data/patterns/TTP11 ../data/preprocessed/interval.csv"']
        print(args)
        out_file = open(f'../results/{pattern_name}_{graph}.out', 'w')
        err_file = open(f'../results/{pattern_name}_{graph}.err', 'w')
        pdata.append((Popen(args, stdout=out_file, stderr=err_file), out_file, err_file))
        
    for proc, out_file, err_file in pdata:
        proc.wait()
        out_file.close()
        err_file.close()

