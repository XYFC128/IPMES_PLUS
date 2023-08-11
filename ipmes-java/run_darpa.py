#!#!/usr/bin/env python
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

subprocess.run(['mvn', 'compile'])
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

for pattern_name, file_prefix in pattern_file:
    print(f'Running pattern {pattern_name}')
    processes: list[Popen] = []
    for graph in darpa_graphs:
        args = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="-w 1000 --darpa ../data/darpa_patterns/{file_prefix} ../data/preprocessed/{graph}.csv"']
        # args = ['bash', '-c', 'time -p -- mvn -q exec:java -Dexec.args="../data/patterns/TTP11 ../data/preprocessed/interval.csv"']
        print(args)
        processes.append(Popen(args, stdout=PIPE, stderr=PIPE))
        
    results = []
    cputime = []
    for proc, graph in zip(processes, darpa_graphs):
        proc.wait()
        stderr = proc.stderr.read().decode()
        stdout = proc.stdout.read().decode()
        cputime.append(parse_cputime(stderr))
        results.append(parse_results(stdout))
        open(f'../results/{pattern_name}_{graph}.out', 'w').write(stderr + stdout)
    all_results.append(results)
    all_cputime.append(cputime)

print('Results')
for (name, _), res in zip(pattern_file, all_results):
    print(f'{name}:', ' '.join(str(i) for i in res))

print('CPU Time (sec)')
for (name, _), res in zip(pattern_file, all_cputime):
    print(f'{name}:', ' '.join('{:.2f}'.format(i) for i in res))
