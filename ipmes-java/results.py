import argparse
import json
import os

def parse_cputime(stderr: str) -> float:
    lines = stderr.strip().split('\n')
    user_time = float(lines[-2].split()[1])
    sys_time = float(lines[-1].split()[1])
    return user_time + sys_time

if __name__ == '__main__':
    parser = parser = argparse.ArgumentParser(
                    description='Convert the run results to csv')
    parser.add_argument('-i', '--in-dir',
                default='../results/ipmes-java/',
                type=str,
                help='the folder containing the run results')
    parser.add_argument('-o', '--out-file',
                default='../results/ipmes-java/run_result.csv',
                type=str,
                help='the output file')
    args = parser.parse_args()

    in_dir = args.in_dir

    if args.dataset == 'darpa':
        from darpa import graphs, pattern_file
    else:
        from spade import graphs, pattern_file

    print('Results: [NumResults, PeakPoolSize, PeakHeapSize]')
    run_result = []
    for pattern_name, _ in pattern_file:
        cputime = []
        pattern_usage_count: dict[int, int] = {}
        for graph in graphs:
            stdout = open(os.path.join(in_dir, f'{pattern_name}_{graph}.out'), 'r').read()
            stderr = open(os.path.join(in_dir, f'{pattern_name}_{graph}.err'), 'r').read()
            cpu_time = parse_cputime(stderr)

            output = json.loads(stdout)
            peak_heap = float(output['PeakHeapSize']) / 2**20
            run_result.append([pattern_name, graph, output['NumResults'], cpu_time, output['PeakPoolSize'], peak_heap])

    import pandas as pd
    df = pd.DataFrame(data=run_result, columns=['Pattern', 'Data Graph', 'Num Results', 'CPU Time (sec)', 'Peak Pool Size', 'Peak Heap Size (MB)'])
    print(df.to_string(index=False))
    df.to_csv(args.out_file, index=False)
