from argparse import ArgumentParser
import json
import random
from re import sub

def parse_pattern(pattern: dict) -> tuple[dict, dict]:
    entities = {}
    events = {}
    
    for entity in pattern['Entities']:
        entities[entity['ID']] = entity

    for event in pattern['Events']:
        events[event['ID']] = event
    
    return entities, events


def random_topo_sort(in_degree: dict[int, int], adj_list: dict[int, list[int]]) -> list[int]:
    queue = []
    for (id, degree) in in_degree.items():
        if degree == 0:
            queue.append(id)

    topo_sorted = []
    while not len(queue) == 0:
        id = queue.pop(random.randrange(len(queue)))
        topo_sorted.append(id)
        for child in adj_list[id]:
            in_degree[child] -= 1
            if in_degree[child] == 0:
                queue.append(child)

    return topo_sorted


if __name__ == '__main__':
    parser = ArgumentParser()
    parser.description = 'A script to generate random input graph from the given pattern'
    parser.add_argument('-o', '--output-file', default='', type=str,
                        help='The output folder of the upgrade result, left empty to use the same name as pattern')
    parser.add_argument('-n', default=1, type=int,
                        help='Number of output subgraphs')
    parser.add_argument('-s', '--seed', default=None, type=int,
                        help='Random seed')
    parser.add_argument('pattern_file')

    args = parser.parse_args()

    if not args.seed is None:
        random.seed(args.seed)

    input_file: str = args.pattern_file
    output_file: str = args.output_file
    if len(output_file) == 0:
        output_file = input_file.removesuffix('.json') + '.csv'

    with open(input_file, 'r') as f:
        pattern = json.load(f)
    entities, events = parse_pattern(pattern)
    
    in_degree = {}
    adj_list = {}
    for event in events.values():
        event_id = event['ID']
        in_degree[event_id] = len(event['Parents'])
        for parent in event['Parents']:
            if not parent in adj_list:
                adj_list[parent] = []
            adj_list[parent].append(event_id)
        if not event_id in adj_list:
            adj_list[event_id] = []

    entity_id_window = max(entities.keys()) + 1
    time = 1.0
    for i in range(args.n):
        topo_sorted = random_topo_sort(in_degree.copy(), adj_list)
        for idx, pattern_event_id in enumerate(topo_sorted):
            time_str = f'{time:.3f}'
            time = random.choice([time, time + 1])
            event = events[pattern_event_id]
            event_id = i * len(events) + idx
            subject = entities[event['SubjectID']]
            subject_id = i * entity_id_window + subject['ID']
            object = entities[event['ObjectID']]
            object_id = i * entity_id_window + object['ID']
            print(','.join([
                time_str, time_str,
                str(event_id), event['Signature'],
                str(subject_id), subject['Signature'],
                str(object_id), object['Signature'],
            ]))
