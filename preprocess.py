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


def convert(inp: str) -> str:
    """
    Convert the original attack graph input into csv format.

    Args:
        inp: a json string
    
    Returns:
        the extracted fields in csv format
    """

    inp_obj = json.loads(inp)
    eid = inp_obj['r']['id']
    esig = extract_edge_signature(inp_obj['r'])
    start_id = inp_obj['m']['id']
    start_sig = extract_node_signature(inp_obj['m'])
    end_id = inp_obj['n']['id']
    end_sig = extract_node_signature(inp_obj['n'])

    return f'{eid},{esig},{start_id},{start_sig},{end_id},{end_sig}'


if __name__ == '__main__':
    """
    This program treat each line in stdin as a JSON object of an event.
    It output the preprocessed event into a csv format to stdout.

    Example usage:
        python preprocess.py < 12hour_attack_08_18.json > output.csv
    
    The fields in the output csv:
        eid, esig, start_id, start_sig, end_id, end_sig:
        - eid:       edge id
        - esig:      edge signature
        - start_id:  id of the start node
        - start_sig: signature of the start node
        - end_id:    id of the end node
        - end_sig:   signature of the end node
    """

    import fileinput
    for line in fileinput.input():
        print(convert(line))
