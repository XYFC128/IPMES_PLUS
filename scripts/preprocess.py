from argparse import ArgumentParser, RawDescriptionHelpFormatter
import json

def extract_node_signature(node_obj: dict) -> str:
    properties = node_obj['properties']
    type: str = properties['type']
    signature = f'{type}::'
    if type == 'Process':
        signature += properties['name']
    elif type == 'Artifact':
        subtype = properties['subtype']
        signature += f'{subtype}::'
        if subtype == 'file' or subtype == 'directory':
            signature += properties['path']
        elif subtype == 'network socket':
            signature += '{}:{}'.format(
                properties['remote address'],
                properties['remote port']
                )
    return signature


def extract_edge_signature(edge_obj: dict) -> str:
    return edge_obj['properties']['operation']


def extract_timestamps(edge_obj: dict) -> tuple[str, str]:
    prop: dict = edge_obj['properties']
    if 'earliest' in prop:
        return prop['earliest'], prop['lastest']
    else:
        return prop['time'], prop['time']


def extract_fields(inp: str, reverse=False) -> list[str]:
    """
    Convert the original attack graph input into a list of fields.

    Args:
        inp: a json string
    
    Returns:
        A list of the extracted fields
    """

    inp_obj = json.loads(inp)
    start_time, end_time = extract_timestamps(inp_obj['r'])
    start_label, end_label = ('m', 'n') if not reverse else ('n', 'm')
    return [
        start_time, end_time,
        inp_obj['r']['id'],
        extract_edge_signature(inp_obj['r']),
        inp_obj[start_label]['id'],
        extract_node_signature(inp_obj[start_label]),
        inp_obj[end_label]['id'],
        extract_node_signature(inp_obj[end_label]),
    ]

if __name__ == '__main__':
    parser = ArgumentParser(description='test', formatter_class=RawDescriptionHelpFormatter)
    parser.description = """
This program treats each line in stdin as a JSON object of an event.
It outputs the preprocessed event in csv format to stdout.

Example usage:
    python preprocess.py < 12hour_attack_08_18.json > output.csv

The fields in the output csv:
    start_time, end_time, eid, event_sig, start_id, start_sig, end_id, end_sig:
    - start_time: the event start time
    - end_time:   the event end time
    - eid:        edge id
    - event_sig:  event signature
    - start_id:   id of the start node
    - start_sig:  start node signature
    - end_id:     id of the end node
    - end_sig:    end node signature
"""
    parser.add_argument('--data_graph', '-G',
                        type=str,
                        default='-',
                        help='The path to the data graph (.json file)')
    parser.add_argument('--reverse', '-r',
                        action='store_true',
                        default=False,
                        help='Reverse the direction of the arcs')
    args = parser.parse_args()

    import fileinput
    for line in fileinput.input(args.data_graph):
        print(','.join(extract_fields(line, args.reverse)))
