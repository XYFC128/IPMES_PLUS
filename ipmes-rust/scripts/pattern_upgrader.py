from argparse import ArgumentParser
from typing import TextIO
import json
import os

def add_freq_and_flow(input_pattern: str) -> str:
    old_obj = json.loads(input_pattern)
    new_obj = {
        'Version': '0.2.0'
    }

    new_obj['UseRegex'] = old_obj['UseRegex']
    entities_map = {}
    events = []
    
    for event in old_obj['Events']:
        signatures = event['Signature'].split('#')
        if len(signatures) != 3:
            raise 'Signature format error in event {}'.format(event['ID'])
        
        event_sig, sub_sig, obj_sig = signatures
        new_event = {
            'ID': event['ID'],
            'Signature': event_sig,
            'SubjectID': event['SubjectID'],
            'ObjectID': event['ObjectID'],
            'Parents': event['Parents']
        }
        events.append(new_event)

        sub_id = event['SubjectID']
        obj_id = event['ObjectID']

        if not sub_id in entities_map:
            entities_map[sub_id] = {
                'ID': sub_id,
                'Signature': sub_sig
            }

        if not obj_id in entities_map:
            entities_map[obj_id] = {
                'ID': obj_id,
                'Signature': obj_sig
            }

    entities = list(entities_map.values())
    entities.sort(key=lambda x: x['ID'])

    new_obj['Entities'] = entities
    new_obj['Events'] = events

    return json.dumps(new_obj, indent=2)

upgrade_map = {
    '0.1.0': (add_freq_and_flow, '0.2.0')
}

def upgrade_file(pattern_file: TextIO, target_version: str) -> str:
    content = pattern_file.read()

    version = json.loads(content)['Version']

    upgrader_chain = []
    while version != target_version:
        if version not in upgrade_map:
            raise f'No available method to upgrade to {target_version}'
        upgrader, new_version = upgrade_map[version]
        upgrader_chain.append(upgrader)
        version = new_version

    for upgrader in upgrader_chain:
        content = upgrader(content)

    return content

latest_version = '0.2.0'

if __name__ == '__main__':
    parser = ArgumentParser()
    parser.add_argument('-t', '--target-version', default=latest_version, type=str)
    parser.add_argument('-o', '--output-dir', default='', type=str,
                        help='The output folder of the upgrade result, left empty (default) if you intend to modify input files inplace')
    parser.add_argument('pattern_files', nargs='+', default=[])

    args = parser.parse_args()

    output_dir = ''
    if len(args.output_dir) > 0:
        os.makedirs(args.output_dir, exist_ok=True)
        output_dir = args.output_dir
    
    for file_path in args.pattern_files:
        try:
            with open(file_path) as f:
                new_content = upgrade_file(f, args.target_version)

            out_path = file_path
            if len(output_dir) > 0:
                basename = os.path.basename(file_path)
                out_path = os.path.join(output_dir, basename)
            
            with open(out_path, 'w') as f:
                f.write(new_content)
            
        except Exception as e:
            print(f'Failed to upgrade pattern file {file_path}: {e}')