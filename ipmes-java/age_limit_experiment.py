#!/usr/bin/env python
import subprocess
from subprocess import Popen
import os
from results import parse_cputime


subprocess.run(['mvn', 'compile'], check=True)

if not os.path.exists('../results'):
    os.mkdir('../results')

os.environ['MAVEN_OPTS'] = '-Xmx100G'

print('window size, CPU time')
for window_size in range(100, 3700, 100):
    args = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="-w {window_size} ../data/patterns/TTP7_regex ../data/preprocessed/mix.csv"']
    # print(args)
    proc = Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    proc.wait()
    stderr = proc.stderr.read().decode()
    cpu_time = parse_cputime(stderr)
    print('{}\t {}'.format(window_size, cpu_time))
