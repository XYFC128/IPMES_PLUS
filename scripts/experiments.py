import typing as t
import subprocess
from subprocess import Popen, PIPE
import re
import os
import pandas as pd 

IPMES_PLUS = 'IPMES_PLUS/'
DATA_GRAPH_DIR = 'data_graphs/'

def build_ipmes_plus():
    cwd = os.getcwd()
    os.chdir(IPMES_PLUS)
    subprocess.run(['cargo', 'build', '--release'], check=True, stdout=None, stderr=None)
    os.chdir(cwd)


def run_ipmes_plus(pattern_file: str, data_graph: str, window_size: int, pre_run=0, re_run=1) -> t.Union[t.Tuple[int, float, int], None]:
    binary = os.path.join(IPMES_PLUS, 'target/release/ipmes-rust')
    run_cmd = [binary, pattern_file, data_graph, '-w', str(window_size), '--silent']
    print('Running: `{}`'.format(' '.join(run_cmd)))

    for _ in range(pre_run):
        proc = Popen(run_cmd, stdout=None, stderr=None, encoding='utf-8')
        proc.wait()

    num_match = '0'
    peak_mem_result = None
    total_cpu_time = 0.0
    for i in range(re_run):
        print(f'Run {i + 1} / {re_run} ...')
        proc = Popen(run_cmd, stdout=PIPE, stderr=PIPE, encoding='utf-8')
        outs, errs = proc.communicate()
        if proc.wait() != 0:
            print(f'Run failed:\n{errs}')
            return None

        print(outs)

        num_match = re.search(r'Total number of matches: (\d+)', outs).group(1)
        cpu_time = re.search(r'CPU time elapsed: (\d+\.\d+) secs', outs).group(1)
        total_cpu_time += float(cpu_time)
        peak_mem_result = re.search(r'Peak memory usage: (\d+) (.)B', outs)

    avg_cpu_time = total_cpu_time / re_run
    num_match = int(num_match)
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
        peak_mem = 0
    
    return num_match, avg_cpu_time, peak_mem


def get_pattern_number(pattern_name: str):
    pattern_name = pattern_name.removesuffix('.json').removesuffix('_regex')
    return int(pattern_name[2:])


def exp_freq_effectivess() -> pd.DataFrame:
    freq_patterns = os.listdir(os.path.join(IPMES_PLUS, 'data/freq_patterns/'))
    freq_patterns.sort(key=lambda p: get_pattern_number(p))

    original_result = []
    freq_result = []
    
    data_graph = os.path.join(DATA_GRAPH_DIR, 'attack_raw.csv')
    for pattern in freq_patterns:
        pattern_name = pattern.removesuffix('.json').removesuffix('_regex')

        original_pattern = os.path.join(IPMES_PLUS, 'data/universal_patterns/', pattern)
        original_res = run_ipmes_plus(original_pattern, data_graph, 1800)
        if not original_res is None:
            num_match, cpu_time, peak_mem = original_res
            original_result.append([pattern_name, num_match, cpu_time, peak_mem / 2**20])

        freq_pattern = os.path.join(IPMES_PLUS, 'data/freq_patterns/', pattern)
        freq_res = run_ipmes_plus(freq_pattern, data_graph, 1800)
        if not freq_res is None:
            num_match, cpu_time, peak_mem = freq_res
            freq_result.append([pattern_name + '_freq', num_match, cpu_time, peak_mem / 2**20])

    run_result = original_result + freq_result
    return pd.DataFrame(data=run_result, columns=['Pattern', 'Found Ins.', 'CPU Time (sec)', 'Peak Memory (MB)'])


def exp_flow_effectivess() -> pd.DataFrame:
    freq_patterns = os.listdir(os.path.join(IPMES_PLUS, 'data/freq_patterns/'))
    freq_patterns.sort(key=lambda p: get_pattern_number(p))

    original_result = []
    freq_result = []
    
    data_graph = os.path.join(DATA_GRAPH_DIR, 'attack_raw.csv')
    for pattern in freq_patterns:
        pattern_name = pattern.removesuffix('.json').removesuffix('_regex')

        original_pattern = os.path.join(IPMES_PLUS, 'data/universal_patterns/', pattern)
        original_res = run_ipmes_plus(original_pattern, data_graph, 1800)
        if not original_res is None:
            num_match, cpu_time, peak_mem = original_res
            original_result.append([pattern_name, num_match, cpu_time, peak_mem / 2**20])

        freq_pattern = os.path.join(IPMES_PLUS, 'data/freq_patterns/', pattern)
        freq_res = run_ipmes_plus(freq_pattern, data_graph, 1800)
        if not freq_res is None:
            num_match, cpu_time, peak_mem = freq_res
            freq_result.append([pattern_name + '_freq', num_match, cpu_time, peak_mem / 2**20])

    run_result = original_result + freq_result
    return pd.DataFrame(data=run_result, columns=['Pattern', 'Found Ins.', 'CPU Time (sec)', 'Peak Memory (MB)'])


def main():
    build_ipmes_plus()
    print(exp_freq_effectivess().to_string(index=False))


if __name__ == '__main__':
    exit(main())
