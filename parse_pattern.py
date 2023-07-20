import json

from preprocess import extract_node_signature, extract_edge_signature

class PatternEdge:
    def __init__(self, id: int, sig: str, start_id: int, end_id: int):
        self.id = id
        self.signature = sig
        self.start = start_id
        self.end = end_id


class PatternNode:
    def __init__(self, id: int, sig: dict):
        self.id = id
        self.signature = sig


class PatternGraph:
    def __init__(self, node_file: str, edge_file: str):
        node_raw = open(node_file).readlines()
        edge_raw = open(edge_file).readlines()
        nodes: list[PatternNode] = []
        edges: list[PatternEdge] = []

        id_convert: dict[int, int] = {}

        for i, line in enumerate(node_raw):
            obj = json.loads(line)
            raw_id = int(obj['node']['id'])
            id_convert[raw_id] = i
            sig = extract_node_signature(obj['node'])
            nodes.append(PatternNode(i, sig))
        
        for i, line in enumerate(edge_raw):
            obj = json.loads(line)
            raw_start = int(obj['edge']['start']['id'])
            raw_end = int(obj['edge']['end']['id'])
            sig = extract_edge_signature(obj['edge'])
            edges.append(PatternEdge(i, sig, id_convert[raw_start], id_convert[raw_end]))

        self.nodes = nodes
        self.edges = edges

    
    def get_endpoints(self, eid: int) -> tuple[PatternNode, PatternNode]:
        """Returns the endpoints of the given edge in the order: (start_node, end_node)"""

        edge = self.edges[eid]
        return self.nodes[edge.start], self.nodes[edge.end]


if __name__ == '__main__':
    import sys
    import pprint
    g = PatternGraph(sys.argv[1], sys.argv[2])
    print('nodes: ')
    for nd in g.nodes:
        pprint.pprint(nd.__dict__)
    print('edges: ')
    for e in g.edges:
        pprint.pprint(e.__dict__)