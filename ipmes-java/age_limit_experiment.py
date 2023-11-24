#!/usr/bin/env python
import subprocess
from subprocess import Popen
import os
import json
import argparse
from results import parse_cputime

parser = parser = argparse.ArgumentParser(
                description='Age limit experiment')
parser.add_argument('--darpa',
                    action='store_true',
                    help='run on darpa')
parser.add_argument('--start',
                    default=100,
                    type=int,
                    help='start of the window size')
parser.add_argument('--stop',
                    default=3700,
                    type=int,
                    help='stop of the window size')
parser.add_argument('--stop-unchanged-poolsize',
                    default=True,
                    type=bool,
                    help='stop when the peak pool size is unchanged')
parser.add_argument('--step',
                    default=100,
                    type=int,
                    help='step of the window size')
parser.add_argument('--multiplier',
                    default=1,
                    type=int,
                    help='if != 1, grow the age limit exponentialy by the multiplier')
parser.add_argument('-r', '--re-run',
                    default=3,
                    type=int,
                    help='re-run times')
args = parser.parse_args()


subprocess.run(['mvn', 'compile'], check=True)

if not os.path.exists('../results'):
    os.mkdir('../results')

os.environ['MAVEN_OPTS'] = '-Xmx100G'

def run(window_size):
    if args.darpa:
        sub_proc_args = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="--darpa --regex -w {window_size} ../data/darpa_patterns/TTP1-1_regex ../data/preprocessed/dd1.csv"']
    else:
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

    return cpu_time, poolsize

print('Window size, CPU time, Peak pool size')
if args.multiplier == 1:
    for window_size in range(args.start, args.stop, args.step):
        run(window_size)
else:
    window_size = args.start
    mul = args.multiplier
    stop = args.stop if args.stop > args.start else float('inf')
    last_pool_size = -1
    while window_size < stop:
        cpu_time, pool_size = run(window_size)
        if pool_size == last_pool_size and args.stop_unchanged_poolsize:
            break
        last_pool_size = pool_size
        window_size *= mul
