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
    import fileinput

    for line in fileinput.input():
        print(convert(line))
