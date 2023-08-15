from darpa import darpa_graphs, pattern_file
import argparse
import json

def parse_cputime(stderr: str) -> float:
    lines = stderr.strip().split('\n')
    user_time = float(lines[-2].split()[1])
    sys_time = float(lines[-1].split()[1])
    return user_time + sys_time

def tabular_print(data: list[list]):
    for row in data:
        print('\t'.join(str(i) for i in row))

def flatten_print(data: list[list]):
    for row in data:
        print('\n'.join(str(i) for i in row))

print_methods = {
    'tabular': tabular_print,
    'flatten': flatten_print,
}

if __name__ == '__main__':
    parser = parser = argparse.ArgumentParser(
                    description='Print the DARPA run results')
    parser.add_argument('-r', '--results',
                        choices=print_methods.keys(),
                        default='tabular',
                        help='The method to print the number of match results')
    parser.add_argument('-t', '--cpu-time',
                        choices=print_methods.keys(),
                        default='flatten',
                        help='The method to print the measured CPU time')
    parser.add_argument('-p', '--pool-size',
                        choices=print_methods.keys(),
                        default='flatten',
                        help='The method to print the measured CPU time')
    args = parser.parse_args()


    all_cputime = []

    print('Results: [NumResults, PeekPoolSize]')
    for pattern_name, _ in pattern_file:
        cputime = []
        for graph in darpa_graphs:
            stdout = open(f'../results/{pattern_name}_{graph}.out', 'r').read()
            stderr = open(f'../results/{pattern_name}_{graph}.err', 'r').read()
            cputime.append(parse_cputime(stderr))

            output = json.loads(stdout)
            print('{}\t{}', output['NumResults'], output['PeekPoolSize'])
        all_cputime.append(cputime)

    print('CPU Time (sec):')
    print_methods[args.cpu_time](all_cputime)
