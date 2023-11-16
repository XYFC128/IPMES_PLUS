#!/usr/bin/env python
import subprocess
from subprocess import Popen
import os
import json
import argparse
from results import parse_cputime

parser = parser = argparse.ArgumentParser(
                description='Age limit experiment')
parser.add_argument('--start',
                    default=100,
                    help='start of the window size')
parser.add_argument('--stop',
                    default=3700,
                    help='stop of the window size')
parser.add_argument('--step',
                    default=100,
                    help='step of the window size')
parser.add_argument('-r', '--re-run',
                    default=3,
                    help='re-run times')
args = parser.parse_args()


subprocess.run(['mvn', 'compile'], check=True)

if not os.path.exists('../results'):
    os.mkdir('../results')

os.environ['MAVEN_OPTS'] = '-Xmx100G'

print('Window size, CPU time, Peak pool size')
for window_size in range(args.start, args.stop, args.step):
    sub_proc_args = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="-w {window_size} ../data/patterns/TTP7_regex ../data/preprocessed/mix.csv"']
    # print(args)
    cpu_time = 0
    for _ in range(args.re_run):
        proc = Popen(sub_proc_args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        proc.wait()
        stderr = proc.stderr.read().decode()
        stdout = proc.stdout.read().decode()

        cpu_time += parse_cputime(stderr)
    cpu_time /= args.re_run
    output = json.loads(stdout)
    poolsize = output['PeakPoolSize']

    print('{}\t {}\t {}'.format(window_size, cpu_time, poolsize))
