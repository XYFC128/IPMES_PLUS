#!/usr/bin/env python
import subprocess
from subprocess import Popen, PIPE
import os
import re
import argparse
import pandas as pd
import json
import sys

def parse_cpu_time(stderr: str) -> float:
    lines = stderr.strip().split('\n')
    user_time = float(lines[-2].split()[1])
    sys_time = float(lines[-1].split()[1])
    return user_time + sys_time

def run(pattern_path: str, graph_path: str, window_size: int, options: str = '', re_run = 1) -> tuple[float, float]:
    '''
    Return CPU-time and peak memory usage (in MB)
    '''

    run_cmd = ['bash', '-c', f'time -p -- mvn -q exec:java -Dexec.args="-w {window_size} {pattern_path} {graph_path} {options}"']
    if re_run > 1:
        print(f'Running ({re_run} times):', ' '.join(run_cmd))
    else:
        print('Running:', ' '.join(run_cmd))
        if re_run < 1:
            return 0, 0, 0
    
    total_cpu_time = 0
    total_mem_usage = 0
    for _ in range(re_run):
        proc = Popen(run_cmd, stdout=PIPE, stderr=PIPE, encoding='utf-8')
        outs, errs = proc.communicate()

        cpu_time = parse_cpu_time(errs)
        output = json.loads(outs)
        mem_usage = float(output['PeakHeapSize']) / 2**20 # convert to MB
        num_result = output['NumResults']

        total_cpu_time += cpu_time
        total_mem_usage += mem_usage

    return num_result, total_cpu_time / re_run, total_mem_usage / re_run

class Runner:
    def __init__(self, pattern_dir: str, graph_dir: str, pattern_filter: str, graphs: list[str]):
        self.pattern_files = []
        filename_filter = re.compile(pattern_filter)
        for pattern_file in os.listdir(pattern_dir):
            if not filename_filter.match(pattern_file):
                continue

            pattern_path = os.path.join(pattern_dir, pattern_file)
            self.pattern_files.append(pattern_path)
        self.pattern_files.sort(key=lambda s: (len(s), s))

        self.graph_files = []
        for graph in graphs:
            self.graph_files.append(os.path.join(graph_dir, graph))

    def new_spade(pattern_dir: str, graph_dir: str):
        spade_graphs = ['attack.csv', 'mix.csv', 'benign.csv']
        pattern_filter = '^SP[0-9]+_regex\\.json$'
        return Runner(pattern_dir, graph_dir, pattern_filter, spade_graphs)
    

    def new_darpa(pattern_dir: str, graph_dir: str):
        darpa_graphs = ['dd1.csv', 'dd2.csv', 'dd3.csv', 'dd4.csv']
        pattern_filter = '^DP[0-9]+_regex\\.json$'
        return Runner(pattern_dir, graph_dir, pattern_filter, darpa_graphs)

    def run_all(self, window_size: int, re_run: int = 1):
        cpu_time_table = []
        mem_usage_table = []
        for pattern in self.pattern_files:
            pattern_name = os.path.basename(pattern).removesuffix('.json')
            cpu_time_row = [pattern_name]
            mem_usage_row = [pattern_name]
            for graph_path in self.graph_files:
                n1, cpu_time, mem_usage = run(pattern, graph_path, window_size, re_run=re_run)
                cpu_time_row.append(cpu_time)
                mem_usage_row.append(mem_usage)

                n2, cpu_time, mem_usage = run(pattern, graph_path, window_size, options='--naive-join', re_run=re_run)
                cpu_time_row.append(cpu_time)
                mem_usage_row.append(mem_usage)

                n3, cpu_time, mem_usage = run(pattern, graph_path, window_size, options='--cep', re_run=re_run)
                cpu_time_row.append(cpu_time)
                mem_usage_row.append(mem_usage)

                if not (n1 == n2 and n2 == n3):
                    print(f'Warning: NumResults differs: {n1}, {n2}, {n3}', file=sys.stderr)
            cpu_time_table.append(cpu_time_row)
            mem_usage_table.append(mem_usage_row)

        cpu_time_columns = ['pattern']
        mem_usage_columns = ['pattern']
        for graph_path in self.graph_files:
            graph_name = os.path.basename(graph_path).removesuffix('.csv')
            column_names = [f'{graph_name}', f'{graph_name}-naive', f'{graph_name}-cep']
            cpu_time_columns.extend(column_names)
            mem_usage_columns.extend(column_names)
        
        cpu_time_df = pd.DataFrame(cpu_time_table, columns=cpu_time_columns, )
        cpu_time_df = cpu_time_df.set_index('pattern')
        cpu_time_df.loc['Average'] = cpu_time_df.mean(numeric_only=True)

        mem_usage_df = pd.DataFrame(mem_usage_table, columns=mem_usage_columns)
        mem_usage_df = mem_usage_df.set_index('pattern')
        mem_usage_df.loc['Average'] = mem_usage_df.mean(numeric_only=True)
        return cpu_time_df, mem_usage_df
    
if __name__ == '__main__':
    parser = parser = argparse.ArgumentParser(
                    formatter_class=argparse.ArgumentDefaultsHelpFormatter,
                    description='Runner')
    parser.add_argument('-D', '--dataset',
                    default='darpa',
                    choices=['darpa', 'spade', 'all'],
                    type=str,
                    help='the dataset to run on')
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
    parser.add_argument('-r', '--re-run',
                    default=1,
                    type=int,
                    help='number of re-runs, cpu-time and memory usage will be the mean of re-runs')
    args = parser.parse_args()


    subprocess.run(['mvn', 'compile'], check=True)

    result_dir = args.out_dir
    os.makedirs(result_dir, exist_ok=True)

    os.environ['MAVEN_OPTS'] = '-Xmx100G'

    def run_and_save(dataset: str):
        if dataset == 'spade':
            runner = Runner.new_spade(args.pattern_dir, args.data_dir)
            window_size = 1800
        elif dataset == 'darpa':
            runner = Runner.new_darpa(args.pattern_dir, args.data_dir)
            window_size = 1000
        else:
            return
        
        cpu_time_df, mem_usage_df = runner.run_all(window_size, args.re_run)

        print(f'{dataset.upper()} CPU Time (sec)')
        print(cpu_time_df)
        cpu_time_save_path = os.path.join(result_dir, f'{dataset}_cpu_time.csv')
        cpu_time_df.to_csv(cpu_time_save_path)
        print(f'This table is saved to {cpu_time_save_path}')
        print()
        print(f'{dataset.upper()} Memory Usage (MB)')
        print(mem_usage_df)
        mem_usage_save_path = os.path.join(result_dir, f'{dataset}_mem_usage.csv')
        mem_usage_df.to_csv(mem_usage_save_path)
        print(f'This table is saved to {mem_usage_save_path}')
        print()

    run_spade = args.dataset in ['spade', 'all']
    run_darpa = args.dataset in ['darpa', 'all']

    if run_spade:
        run_and_save('spade')

    if run_darpa:
        run_and_save('darpa')
