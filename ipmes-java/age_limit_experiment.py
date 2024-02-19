#!/usr/bin/env python
import subprocess
from subprocess import Popen
import os
import json
import argparse
import sys
import pickle
from results import parse_cputime

parser = parser = argparse.ArgumentParser(
                formatter_class=argparse.ArgumentDefaultsHelpFormatter,
                description='Age limit experiment')
parser.add_argument('-l', '--ws-list',
                    default='',
                    type=str,
                    help='comma seperated list of window sizes')
parser.add_argument('--start',
                    default=1,
                    type=int,
                    help='start of the window size')
parser.add_argument('--stop',
                    default=11,
                    type=int,
                    help='stop of the window size')
parser.add_argument('--stop-unchanged-poolsize',
                    default=True,
                    choices=[True, False],
                    type=bool,
                    help='stop when the peak pool size is unchanged')
parser.add_argument('--step',
                    default=1,
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
parser.add_argument('--dry',
                    action='store_true',
                    help='use the results saved from previous run if exist')
parser.add_argument('--columns',
                    default='all',
                    type=str,
                    help='comma seperated list of output columns')
parser.add_argument('-p', '--pattern',
                    default='../data/universal_patterns/SP7_regex.json',
                    type=str,
                    help='pattern prefix')
parser.add_argument('-d', '--data',
                    default='../data/preprocessed/mix.csv',
                    type=str,
                    help='data graph')
parser.add_argument('-o', '--log-dir',
                    default='../results/agelimits/',
                    type=str,
                    help='the output log dir')
args = parser.parse_args()


subprocess.run(['mvn', 'compile'], check=True)

log_dir = args.log_dir

print('log dir:', log_dir, file=sys.stderr)
os.makedirs(log_dir, exist_ok=True)

os.environ['MAVEN_OPTS'] = '-Xmx100G'

output_table: list[dict] = []

def count_clusters(results: list[dict]) -> int:
    clusters = {}
    for res in results:
        clusters.setdefault(res['StartTime'], 0)
        clusters[res['StartTime']] += 1
    return len(clusters)

def run(window_size):
    sub_proc_args = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="-w {window_size} --dump-results {args.pattern} {args.data}"']

    cpu_time = 0
    out_file = os.path.join(log_dir, f'{window_size}s.out')
    metadata_file = os.path.join(log_dir, f'{window_size}s_metadata.pkl')

    if not args.dry or not os.path.exists(out_file) or not os.path.exists(metadata_file):
        print(' '.join(sub_proc_args), file=sys.stderr)
        for _ in range(args.re_run):
            out_redirect = open(out_file, 'w')
            proc = Popen(sub_proc_args, stdout=out_redirect, stderr=subprocess.PIPE)
            proc.wait()
            stderr = proc.stderr.read().decode()
            out_redirect.close()

            cpu_time += parse_cputime(stderr)

        cpu_time /= args.re_run

        metadata = {}
        metadata['Avg CPU Time'] = cpu_time
        pickle.dump(metadata, open(metadata_file, 'wb'))
    else:
        metadata = pickle.load(open(metadata_file, 'rb'))

    with open(out_file, 'r') as f:
        stdout = f.read()

        output = json.loads(stdout)
        poolsize = output['PeakPoolSize']
        num_reults = output['NumResults']
        num_clusters = count_clusters(output['MatchResults'])

    output_table.append({
        'WindowSize': window_size,
        'AvgCpuTime': metadata['Avg CPU Time'],
        'PoolSize': poolsize,
        'NumResults': num_reults,
        'NumClusters': num_clusters,
        })

    return cpu_time, poolsize

if len(args.ws_list) > 0:
    ws_list = [int(x) for x in args.ws_list.split(',')]
    for window_size in ws_list:
        run(window_size)
elif args.multiplier == 1:
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

if len(output_table) > 0:
    columns = output_table[0].keys()
    if args.columns != 'all':
        columns = args.columns.split(',')
    print(', '.join(columns))
    for row in output_table:
        data = [str(row[k]) for k in columns]
        print('\t '.join(data))

