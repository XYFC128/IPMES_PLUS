def get_attr(output: str, attr_name: str) -> str:
    pattern = attr_name + ' = '
    start = output.find(pattern) + len(pattern)
    if start == -1:
        return ''
    end = output.find('\n', start)
    return output[start:end]

from spade import pattern_file
graphs = ['12hour_attack_tmp', '12hour_mix', '12hour_background']
for _, prefix in pattern_file:
    cputime = []
    pattern_usage_count: dict[int, int] = {}
    for graph in graphs:
        ans = open(f'../data/memory overhead/{graph}/{prefix}_1800s.txt', 'r').read()
        memory = get_attr(ans, 'memory(current, peak in byte)')
        print(memory.split()[1][:-1])

