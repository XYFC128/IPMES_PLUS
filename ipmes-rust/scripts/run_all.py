import argparse
import subprocess
from subprocess import Popen, PIPE
import os
import re
import pandas as pd 

def run(pattern_file: str, data_graph: str, window_size: int):
    run_cmd = ['./target/release/ipmes-rust', pattern_file, data_graph, '-w', str(window_size)]
    print('Running: `{}`'.format(' '.join(run_cmd)))

    proc = Popen(run_cmd, stdout=PIPE, stderr=PIPE, encoding='utf-8')
    outs, errs = proc.communicate()

    print(outs)

    num_match = re.search('Total number of matches: (\d+)', outs).group(1)
    num_match = int(num_match)
    cpu_time = re.search('CPU time elapsed: (\d+\.\d+) secs', outs).group(1)
    cpu_time = float(cpu_time)

    peak_mem_result = re.search('Peak memory usage: (\d+) (.)B', outs)
    if peak_mem_result is not None:
        peak_mem = peak_mem_result.group(1)
        peak_mem_unit = peak_mem_result.group(2)

        multiplier = 1
        if peak_mem_unit == 'k':
            multiplier = 2**10
        elif peak_mem_unit == 'M':
            multiplier = 2**20
        elif peak_mem_unit == 'G':
            multiplier = 2**30
        else:
            print(f'Encounter unknown memory unit: {peak_mem_unit}')
        
        peak_mem = int(peak_mem) * multiplier
    else:
        peak_mem = None
    
    return num_match, cpu_time, peak_mem

if __name__ == '__main__':
    parser = parser = argparse.ArgumentParser(
                formatter_class=argparse.ArgumentDefaultsHelpFormatter,
                description='Run all pattern on all graph')
    parser.add_argument('-d', '--data-graph',
                    default='../data/preprocessed/',
                    type=str,
                    help='the folder of data graphs')
    parser.add_argument('-p', '--pattern-dir',
                    default='../data/universal_patterns/',
                    type=str,
                    help='the folder of patterns')
    parser.add_argument('-o', '--out-dir',
                    default='../results/ipmes-rust/',
                    type=str,
                    help='the output folder')
    args = parser.parse_args()

    
    if os.getcwd().endswith('scripts'):
        os.chdir('..')

    subprocess.run(['cargo', 'build', '--release'], check=True)

    os.makedirs(args.out_dir, exist_ok=True)
    
    darpa_graphs = ['dd1', 'dd2', 'dd3', 'dd4']
    spade_graphs = ['attack', 'mix', 'benign']

    run_result = []

    for i in range(1, 13):
        for graph in spade_graphs:
            pattern = os.path.join(args.pattern_dir, f'SP{i}_regex.json')
            data_graph = os.path.join(args.data_graph, graph + '.csv')
            num_match, cpu_time, peak_mem = run(pattern, data_graph, 1800)
            run_result.append([f'SP{i}', graph, num_match, cpu_time, peak_mem / 2**20])

    for i in range(1, 6):
        for graph in darpa_graphs:
            pattern = os.path.join(args.pattern_dir, f'DP{i}_regex.json')
            data_graph = os.path.join(args.data_graph, graph + '.csv')
            num_match, cpu_time, peak_mem = run(pattern, data_graph, 1000)
            run_result.append([f'DP{i}', graph, num_match, cpu_time, peak_mem / 2**20])

    df = pd.DataFrame(data=run_result, columns=['Pattern', 'Data Graph', 'Num Results', 'CPU Time (sec)', 'Peak Memory (MB)'])
    print(df.to_string(index=False))
    df.to_csv(os.path.join(args.out_dir, 'run_result.csv'), index=False)

