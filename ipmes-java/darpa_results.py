from darpa import darpa_graphs, pattern_file
import argparse

def parse_cputime(stderr: str) -> float:
    lines = stderr.strip().split('\n')
    user_time = float(lines[-2].split()[1])
    sys_time = float(lines[-1].split()[1])
    return user_time + sys_time

def parse_results(stdout: str) -> int:
    result_start = stdout.find('Match Results:')
    return stdout[result_start:].count('[')

def read_results() -> tuple[list, list]:
    all_results = []
    all_cputime = []

    for pattern_name, _ in pattern_file:
        results = []
        cputime = []
        for graph in darpa_graphs:
            output = open(f'../results/{pattern_name}_{graph}.out', 'r').read()
            results.append(parse_results(output))
            cputime.append(parse_cputime(output))
        all_results.append(results)
        all_cputime.append(cputime)

    return all_results, all_cputime

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
    args = parser.parse_args()
    all_results, all_cputime = read_results()
    print('Results:')
    print_methods[args.results](all_results)

    print('CPU Time (sec):')
    print_methods[args.cpu_time](all_cputime)
