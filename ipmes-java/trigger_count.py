import darpa
import spade
import json


def get_trigger_counts(pattern_file, graphs) -> list:
    trigger_counts: list = []
    for pattern_name, _ in pattern_file:
        for graph in graphs:
            stdout = open(f'../results/{pattern_name}_{graph}.out', 'r').read()
            output = json.loads(stdout)
            trigger_counts += output['TriggerCounts']
    return trigger_counts

trigger_counts: list[list[int]] = []
trigger_counts += get_trigger_counts(darpa.pattern_file, darpa.graphs)
trigger_counts += get_trigger_counts(spade.pattern_file, darpa.spade)

for counts in trigger_counts:
    max_count = max(counts)
    print(' \t'.join([str(c / max_count) for c in counts]))