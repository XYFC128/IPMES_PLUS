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

if __name__ == '__main__':
    ans_folder = '../data/answer'
    data_folder = '../data/preprocessed'
    out_dir = '../data/temp'
    pattern = 'SP8_regex'
    data = 'attack'

    os.makedirs(out_dir, exist_ok=True)

    ans_file = get_ans_file(ans_folder, pattern, data)
    answers = parse_ans_file(ans_file)
    
    input_events = index_data_graph(os.path.join(data_folder, f'{data}.csv'))
    
    for ans_num, ans_ids in enumerate(answers, start=1):
        event_list = []
        for id in ans_ids:
            event_list.append(input_events[id])
        event_list.sort()
        event_string = ''.join(x[1] for x in event_list)
        
        with tempfile.NamedTemporaryFile(mode='w+') as fp:
            fp.write(event_string)
            fp.close()

            run = subprocess.run(['cargo', 'run', '--', f'../data/universal_patterns/{pattern}.json', os.path.join(data_folder, f'{data}.csv')], capture_output=True)
        
        num_result = get_num_results_from_stdout(run.stdout)
        if num_result != 1:
            print(f'The number of matches should be 1, but found {num_result} instead')
            print(f'In {ans_file}:')
            print(f'The #{ans_num} answer:', ','.join(ans_ids))

            out_file = os.path.join(out_dir, f'expect_one_{pattern}.csv')
            with open(out_file, 'w') as f:
                f.write(event_string)

            print(f'The corresponding input has been written to {out_file}')
            break
            
    print('All tests passed')