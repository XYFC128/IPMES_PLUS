#!/usr/bin/env python
import subprocess
from subprocess import Popen
import os
import sys
import re
import argparse

parser = parser = argparse.ArgumentParser(
                formatter_class=argparse.ArgumentDefaultsHelpFormatter,
                description='Age limit experiment')
parser.add_argument('-D', '--dataset',
                default='darpa',
                type=str,
                help='darpa or spade')
parser.add_argument('-d', '--data-dir',
                default='../data/preprocessed/',
                type=str,
                help='the folder of data graphs')
parser.add_argument('-p', '--pattern-dir',
                default='../data/universal_patterns/',
                type=str,
                help='the folder of patterns')
parser.add_argument('-o', '--out-dir',
                default='../results/ipmes-java/',
                type=str,
                help='the output folder')
args = parser.parse_args()

data_dir = args.data_dir

if args.dataset == 'darpa':
    from darpa import graphs
    pattern_filter = '^DP[0-9]+_regex\.json$'
    window_size = 1000
else:
    from spade import graphs
    pattern_filter = '^SP[0-9]+_regex\.json$'
    window_size = 1800

subprocess.run(['mvn', 'compile'], check=True)

result_dir = args.out_dir
os.makedirs(result_dir, exist_ok=True)

os.environ['MAVEN_OPTS'] = '-Xmx100G'

pattern_dir = args.pattern_dir

filename_filter = re.compile(pattern_filter)
for pattern_file in os.listdir(pattern_dir):
    if not filename_filter.match(pattern_file):
        continue
    pattern_name = pattern_file.removesuffix('_regex.json')
    print(f'Running pattern {pattern_name}')
    pattern_path = os.path.join(pattern_dir, pattern_file)

    pdata: list[tuple] = []
    for graph in graphs:
        graph_path = os.path.join(data_dir, f'{graph}.csv')
        args = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="-w {window_size} {pattern_path} {graph_path}"']
        print(args)
        out_file = open(os.path.join(result_dir, f'{pattern_name}_{graph}.out'), 'w')
        err_file = open(os.path.join(result_dir, f'{pattern_name}_{graph}.err'), 'w')
        pdata.append((Popen(args, stdout=out_file, stderr=err_file), out_file, err_file))
        
    for proc, out_file, err_file in pdata:
        proc.wait()
        out_file.close()
        err_file.close()

