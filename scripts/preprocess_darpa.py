import json
from preprocess import extract_timestamps

def extract_node_signature(node_obj: dict) -> str:
    properties: dict = node_obj['properties']
    if 'NetFlowObject_baseObject_epoch' in properties:
        type = 'OBJECT_SOCKET'
    elif 'UnnamedPipeObject_baseObject_epoch' in properties:
        type = 'OBJECT_UNNAMEPIPE'
    elif 'type' in properties:
        type = properties['type']
    else:
        return 'OTHER'

    signature = f'{type}::'
    if type == 'SUBJECT_PROCESS' or type == 'SUBJECT_UNIT':
        signature += properties['Subject_properties_map_name']
    elif type == 'FILE_OBJECT_CHAR' or type == 'FILE_OBJECT_FILE' or type == 'FILE_OBJECT_DIR':
        signature += properties['path']
    elif type == 'OBJECT_SOCKET':
        signature += properties['NetFlowObject_remoteAddress']
        signature += ':'
        signature += str(properties['NetFlowObject_remotePort'])
    elif type == 'SRCSINK_UNKNOWN':
        signature += str(properties['SrcSinkObject_pid'])

    return signature


def extract_edge_signature(edge_obj: dict) -> str:
    return edge_obj['properties']['type']


def extract_fields(inp: str) -> list[str]:
    """
    Convert the original attack graph input into a list of fields.

    Args:
        inp: a json string
    
    Returns:
        A list of the extracted fields
    """

    inp_obj = json.loads(inp)
    start_time, end_time = extract_timestamps(inp_obj['r'])
    eid = inp_obj['r']['id']
    start_id = inp_obj['m']['id']
    end_id = inp_obj['n']['id']
    event_sig = '{}#{}#{}'.format(
        extract_edge_signature(inp_obj['r']),
        extract_node_signature(inp_obj['m']),
        extract_node_signature(inp_obj['n'])
    )
    
    time_unit = 1000000000
    return [str(start_time / time_unit), str(end_time / time_unit), event_sig, eid, start_id, end_id]


if __name__ == '__main__':
    """
    This program treats each line in stdin as a JSON object of an event.
    It outputs the preprocessed event in csv format to stdout.

    Example usage:
        python preprocess.py < 12hour_attack_08_18.json > output.csv
    
    The fields in the output csv:
        start_time, end_time, event_sig, eid, start_id, end_id:
        - start_time: the event start time
        - end_time:   the event end time
        - event_sig:  event signature
        - eid:        edge id
        - start_id:   id of the start node
        - end_id:     id of the end node
    """

    import fileinput
    for line in fileinput.input():
        print(','.join(extract_fields(line)))
