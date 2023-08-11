#!/usr/bin/env python
from io import TextIOWrapper
import subprocess
from subprocess import PIPE, Popen
import os

def parse_cputime(stderr: str) -> float:
    lines = stderr.strip().split('\n')
    user_time = float(lines[-2].split()[1])
    sys_time = float(lines[-1].split()[1])
    return user_time + sys_time

def parse_results(stdout: str) -> int:
    result_start = stdout.find('Match Results:')
    return stdout[result_start:].count('[')

subprocess.run(['mvn', 'compile'], check=True)
pattern_file = [
    ('DDP1', 'TTP1-1_regex'),
    ('DDP2', 'TTP1-2_regex'),
    ('DDP3', 'TTP2_regex'),
    ('DDP4', 'TTP3_regex'),
    ('DDP5', 'TTP4_regex'),
]
darpa_graphs = ['dd1', 'dd2', 'dd3', 'dd4']

all_results = []
all_cputime = []

if not os.path.exists('../results'):
    os.mkdir('../results')

os.environ['MAVEN_OPTS'] = '-Xmx100G'

for pattern_name, file_prefix in pattern_file:
    print(f'Running pattern {pattern_name}')
    pdata: list[tuple[Popen, TextIOWrapper]] = []
    for graph in darpa_graphs:
        # args = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="-w 1000 --darpa ../data/darpa_patterns/{file_prefix} ../data/preprocessed/{graph}.csv" &> {out_file}']
        args = ['bash', '-c', 'time -p -- mvn -q exec:java -Dexec.args="../data/patterns/TTP11 ../data/preprocessed/interval.csv"']
        print(args)
        out_file = open(f'../results/{pattern_name}_{graph}.out', 'w')
        pdata.append((Popen(args, stdout=out_file, stderr=out_file), out_file))
        
    results = []
    cputime = []
    for proc, out_file in pdata:
        proc.wait()
        out_file.close()
    for graph in darpa_graphs:
        output = open(f'../results/{pattern_name}_{graph}.out', 'r').read()
        results.append(parse_results(output))
        cputime.append(parse_cputime(output))
    all_results.append(results)
    all_cputime.append(cputime)

print('Results')
for (name, _), res in zip(pattern_file, all_results):
    print(f'{name}:', ' '.join(str(i) for i in res))

print('CPU Time (sec)')
for (name, _), res in zip(pattern_file, all_cputime):
    print(f'{name}:', ' '.join('{:.2f}'.format(i) for i in res))
