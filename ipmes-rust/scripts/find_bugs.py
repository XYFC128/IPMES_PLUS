import os
import re
import tempfile
import subprocess

pattern_name_mapping = {
    'SP1': 'TTP1',
    'SP2': 'TTP2',
    'SP3': 'TTP3',
    'SP4': 'TTP4',
    'SP5': 'TTP5',
    'SP6': 'TTP6',
    'SP7': 'TTP7',
    'SP8': 'TTP8',
    'SP9': 'TTP9',
    'SP10': 'TTP9-2',
    'SP11': 'TTP10',
    'SP12': 'TTP11',
    'DP1': 'TTP1-1',
    'DP2': 'TTP1-2',
    'DP3': 'TTP2',
    'DP4': 'TTP3',
    'DP5': 'TTP4',
}

data_name_mapping = {
    'attack': '12hour_attack_tmp',
    'benign': '12hour_background',
    'mix': '12hour_mix',
    # TODO: DDx
}

def to_ans_file(pattern_name: str) -> str:
    key = pattern_name.removesuffix('_regex')
    old_pattern_name = pattern_name_mapping[key]
    complete_pattern_name = pattern_name.replace(key, old_pattern_name)
    default_window = '1800s' if pattern.startswith('SP') else '1000s'
    return f'{complete_pattern_name}_{default_window}.txt'

def get_ans_file(ans_dir: str, pattern: str, data_graph: str) -> str:
    return os.path.join(ans_dir, data_name_mapping[data_graph], to_ans_file(pattern))

def parse_ans_file(file_path: str) -> list[list[str]]:
    with open(file_path) as f:
        lines = f.readlines()
    
    ans_pattern = re.compile('<.*>\[([0-9,]*)\]')
    answers = []
    for line in lines:
        result = ans_pattern.search(line)
        if result is None:
            continue
        ans = result.group(1)
        answers.append(ans.split(','))
    return answers

def index_data_graph(data_graph: str) -> dict[tuple[int, str]]:
    with open(data_graph) as f:
        lines = f.readlines()
    
    data_edges = {}
    for ln, line in enumerate(lines):
        fields = line.split(',')
        if len(fields) < 4:
            continue
        event_id = fields[3]
        data_edges[event_id] = (ln, line)

    return data_edges

def get_num_results_from_stdout(stdout: bytes) -> int:
    stdout = stdout.decode()
    match_result = re.search('Total number of matches: (\d+)', stdout)
    if match_result is None:
        return 0
    
    return int(match_result.group(1))

def get_match_results_from_stderr(stderr: bytes):
    lines = stderr.decode().split('\n')
    ans_pattern = re.compile('Pattern Match: \[([0-9,\s]+)\]')
    match_results = []
    for line in lines:
        found = ans_pattern.search(line)
        if found is None:
            continue
        ids = found.group(1).split(', ')
        match_results.append(ids)

    return match_results

def find_wrong_answers(answers: list[list[str]], match_results: list[list[str]]):
    ans_dict = {}
    for ans_ids in answers:
        ans_dict[','.join(ans_ids)] = 0
    
    false_positive = []
    for match_ids in match_results:
        key = ','.join(match_ids)
        if key not in ans_dict:
            false_positive.append(key)
            continue
        ans_dict[key] += 1
    
    true_negative = []
    for key, val in ans_dict.items():
        if val == 0:
            true_negative.append(key)
        elif val > 1:
            print(f'The match result {key} appears {val} times')
    
    return false_positive, true_negative

def reassign_event_id(event_string: str) -> str:
    lines = event_string.split('\n')
    output = ''
    node_id_map = {}
    edge_id_map = {}
    for line in lines:
        fields = line.split(',')
        if len(fields) < 6:
            continue
        
        edge_id_map.setdefault(fields[3], str(len(edge_id_map) + 1))
        fields[3] = edge_id_map[fields[3]]

        node_id_map.setdefault(fields[4], str(len(node_id_map) + 1))
        node_id_map.setdefault(fields[5], str(len(node_id_map) + 1))
        fields[4] = node_id_map[fields[4]]
        fields[5] = node_id_map[fields[5]]
        output += ','.join(fields) + '\n'

    return output

def gen_wrong_answer(input_events, event_ids, out_file, expect_num):
    print(f'Generating small input graph expecting {expect_num} results to {out_file}')
    event_list = []
    for id in event_ids:
        event_list.append(input_events[id])
    event_list.sort()
    event_string = ''.join(x[1] for x in event_list)
    event_string = reassign_event_id(event_string)

    with open(out_file, 'w') as f:
        f.write(event_string)

if __name__ == '__main__':
    import argparse
    parser = parser = argparse.ArgumentParser(
                formatter_class=argparse.ArgumentDefaultsHelpFormatter,
                description='Automatic bug finder')
    
    ans_folder = '../data/answer'
    data_folder = '../data/preprocessed'
    out_dir = '../data/temp'
    pattern = 'SP9_regex'
    data = 'attack'

    os.makedirs(out_dir, exist_ok=True)

    ans_file = get_ans_file(ans_folder, pattern, data)
    answers = parse_ans_file(ans_file)
    
    print('Indexing data graph...')
    input_events = index_data_graph(os.path.join(data_folder, f'{data}.csv'))

    run_args = ['cargo', 'run', '--', f'../data/universal_patterns/{pattern}.json', os.path.join(data_folder, f'{data}.csv')]
    print(run_args)
    os.environ['RUST_LOG'] = 'info'
    run = subprocess.run(run_args, capture_output=True)
    os.environ['RUST_LOG'] = ''

    match_results = get_match_results_from_stderr(run.stderr)
    false_positive, true_negative = find_wrong_answers(answers, match_results)

    if len(false_positive) == 0 and len(true_negative) == 0:
        print('The match result is the same as the answer')
        exit(0)

    print('Among {} match results, there are {} results are not in answer, and {} answers not found in the results'
          .format(len(match_results), len(false_positive), len(true_negative)))
    
    if len(false_positive) > 0:
        ids = false_positive[0].split(',')
        gen_wrong_answer(input_events, ids, os.path.join(out_dir, f'expect_no_{pattern}.csv'), 0)
    else:
        ids = true_negative[0].split(',')
        gen_wrong_answer(input_events, ids, os.path.join(out_dir, f'expect_{1}_{pattern}.csv'), 0)