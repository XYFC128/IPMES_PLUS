import json
import argparse

class Edge:
    def __init__(self, id: int, op: str, start: int, end: int):
        self.id = id
        self.op = op
        self.start = start
        self.end = end

class Node:
    def __init__(self, id: int, type: str):
        self.id = id
        self.type = type


def parse_graph(node_file: str, edge_file: str) -> tuple[dict[int, Node], list[Edge]]:
    '''
    Parse the input json file into internal structure
    '''
    node_raw = open(node_file).readlines()
    edge_raw = open(edge_file).readlines()
    nodes = {}
    edges = []
    
    for line in node_raw:
        root = json.loads(line)
        id = int(root['node']['id'])
        nodes[id] = Node(id, root['node']['properties']['type'])

    for line in edge_raw:
        root = json.loads(line)
        id = int(root['edge']['id'])
        op = root['edge']['properties']['operation']
        start = int(root['edge']['start']['id'])
        end = int(root['edge']['end']['id'])
        edges.append(Edge(id, op, start, end))
    
    return nodes, edges


def gen_header(num_node: int, num_edge: int) -> str:
    '''
    Generate stream and table definition
    '''
    header = '@App:name("SiddhiApp")\n'
    header += 'define Stream InputStream (eid string, op string, start_id string, start_type string, end_id string, end_type string);\n'

    param = '('
    output_condition = ''
    for i in range(0, num_node):
        param += 'n{}_id string, '.format(i)
        output_condition += f'n{i}_id != "null" and '

    for i in range(0, num_edge):
        param += f'e{i}_id string'
        output_condition += f'e{i}_id != "null"'
        if i != num_edge - 1:
            param += ', '
            output_condition += ' and '
        else:
            param += ');\n'
    
    header += 'define Stream CandidateStream ' + param
    header += 'define Table CandidateTable ' + param
    header += '@sink(type="log")\ndefine Stream OutputStream ' + param
    header += f'''
from CandidateStream
select *
insert into CandidateTable;

from CandidateStream[{output_condition}]
select *
insert into OutputStream;
'''
    return header


def gen_edge_rules(nodes: dict[int, Node], edges: list[Edge]) -> str:
    '''
    Generate rule for every edge
    '''
    node_field = {}
    for _, nd in nodes.items():
        ith = len(node_field)
        node_field[nd.id] = 'n{}_id'.format(ith)

    rule = ''

    for ith, edge in enumerate(edges):
        edge_condition = 'op == "{}" and start_type == "{}" and end_type == "{}"'.format(
            edge.op,
            nodes[edge.start].type,
            nodes[edge.end].type,
        )
        start_field = node_field[edge.start]
        end_field = node_field[edge.end]
        edge_field = 'e{}_id'.format(ith)

        select_expr = ''
        for i in range(len(nodes)):
            cur_field = f'n{i}_id'
            if cur_field == start_field:
                select_expr += f'start_id as {cur_field}'
            elif cur_field == end_field:
                select_expr += f'end_id as {cur_field}'
            else:
                select_expr += f'"null" as {cur_field}'
            select_expr += ', '
        for i in range(len(edges)):
            if i == ith:
                select_expr += f'eid as e{i}_id'
            else:
                select_expr += f'"null" as e{i}_id'
            select_expr += ', ' if i != len(edges) - 1 else ''
        
        rule += f'''
from InputStream[{edge_condition}]
select {select_expr}
insert into CandidateStream;
'''
        select_expr = ''
        for i in range(len(nodes)):
            cur_field = f'n{i}_id'
            if cur_field == start_field:
                select_expr += f's.start_id as {cur_field}'
            elif cur_field == end_field:
                select_expr += f's.end_id as {cur_field}'
            else:
                select_expr += f't.{cur_field}'
            select_expr += ', '
        for i in range(len(edges)):
            if i == ith:
                select_expr += f's.eid as e{i}_id'
            else:
                select_expr += f't.e{i}_id'
            select_expr += ', ' if i != len(edges) - 1 else ''

        rule += f'''
from InputStream[{edge_condition}] as s join
    CandidateTable as t
    on t.{edge_field} == "null" and
        ((t.{start_field} == s.start_id and t.{end_field} == s.end_id) or
        (t.{start_field} == s.start_id and t.{end_field} == "null") or
        (t.{start_field} == "null" and t.{end_field} == s.end_id))
select {select_expr}
insert into CandidateStream;
'''
    return rule
    

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--node',
                        required=True,
                        help='node file path')
    parser.add_argument('--edge',
                        required=True,
                        help='edge file path')

    args = parser.parse_args()
    nodes, edges = parse_graph(args.node, args.edge)
    print(gen_header(len(nodes), len(edges)))
    print(gen_edge_rules(nodes, edges))