import sys
import os
import pprint
import json
from typing import Literal

from preprocess import extract_edge_signature as extract_spade_edge_signature, extract_node_signature as extract_spade_node_signature
from preprocess_darpa import extract_edge_signature as extract_darpa_edge_signature, extract_node_signature as extract_darpa_node_signature


spade_pattern_file = [
    ('SP1', 'TTP1'),
    ('SP2', 'TTP2'),
    ('SP3', 'TTP3'),
    ('SP4', 'TTP4'),
    ('SP5', 'TTP5'),
    ('SP6', 'TTP6'),
    ('SP7', 'TTP7'),
    ('SP8', 'TTP8'),
    ('SP9', 'TTP9'),
    ('SP10', 'TTP9-2'),
    ('SP11', 'TTP10'),
    ('SP12', 'TTP11'),
]

darpa_pattern_file = [
    ('DP1', 'TTP1-1'),
    ('DP2', 'TTP1-2'),
    ('DP3', 'TTP2'),
    ('DP4', 'TTP3'),
    ('DP5', 'TTP4'),
]

darpa_dir = '../data/darpa_patterns/'
spade_dir = '../data/patterns/'
output_dir = '../data/universal_patterns/'

def extract_edge_signature(edge_obj: dict, kind: Literal['darpa', 'spade']) -> str:
    if kind == 'spade':
        return extract_spade_edge_signature(edge_obj)
    elif kind == 'darpa':
        return extract_darpa_edge_signature(edge_obj)

def extract_node_signature(node_obj: dict, kind: Literal['darpa', 'spade']) -> str:
    if kind == 'spade':
        return extract_spade_node_signature(node_obj)
    elif kind == 'darpa':
        return extract_darpa_node_signature(node_obj)


def parse(node_file: str, edge_file: str, orels_file: str, kind: Literal['darpa', 'spade']) -> list[dict]:
    node_raw = open(node_file).readlines()
    edge_raw = open(edge_file).readlines()
    node_signatures: list[str] = []
    edges: list[dict] = []

    id_convert: dict[int, int] = {}

    for i, line in enumerate(node_raw):
        obj = json.loads(line)
        raw_id = int(obj['node']['id'])
        id_convert[raw_id] = i
        sig = extract_node_signature(obj['node'], kind)
        node_signatures.append(sig)
    
    for i, line in enumerate(edge_raw):
        obj = json.loads(line)
        raw_start = int(obj['edge']['start']['id'])
        start_id = id_convert[raw_start]
        start_sig = node_signatures[start_id]

        raw_end = int(obj['edge']['end']['id'])
        end_id = id_convert[raw_end]
        end_sig = node_signatures[end_id]
        
        sig = extract_edge_signature(obj['edge'], kind)
        edges.append({
            'ID': i,
            'Signature': f'{sig}#{start_sig}#{end_sig}',
            'SubjectID': start_id,
            'ObjectID': end_id,
        })

    orels:dict = json.load(open(orels_file))
    for key, val in orels.items():
        if key == 'root':
            continue
        id = int(key)
        parents = val['parents']
        while 'root' in parents:
            parents.remove('root')
        edges[id]['Parents'] = parents
    
    return edges

def convert(input_prefix: str, output_path: str, regex: bool, kind: Literal['darpa', 'spade']):
    print(f'Converting {input_prefix} to {output_path}')
    node_file = input_prefix + '_node.json'
    edge_file = input_prefix + '_edge.json'
    orels_file = input_prefix.removesuffix('_regex') + '_oRels.json'

    edges = parse(node_file, edge_file, orels_file, kind)

    output = {
        'Version': '0.1',
        'UseRegex': regex,
        'Events': edges
    }
    json.dump(output, open(output_path, 'w'), indent=4)

if __name__ == '__main__':
    for name, prefix in spade_pattern_file:
        convert(os.path.join(spade_dir, prefix), os.path.join(output_dir, f'{name}.json'), False, 'spade')
        convert(os.path.join(spade_dir, prefix + '_regex'), os.path.join(output_dir, f'{name}_regex.json'), True, 'spade')

    for name, prefix in darpa_pattern_file:
        convert(os.path.join(darpa_dir, prefix), os.path.join(output_dir, f'{name}.json'), False, 'darpa')
        convert(os.path.join(darpa_dir, prefix + '_regex'), os.path.join(output_dir, f'{name}_regex.json'), True, 'darpa')