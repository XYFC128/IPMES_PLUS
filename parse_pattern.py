import json

from preprocess import extract_node_signature

class PatternEdge:
    def __init__(self, id: int, op: str, start: int, end: int):
        self.id = id
        self.signature = op
        self.start = start
        self.end = end


class PatternNode:
    def __init__(self, id: int, node_obj: dict):
        self.id = id
        self.signature = extract_node_signature(node_obj)


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
            nodes.append(PatternNode(i, obj['node']))
        
        for i, line in enumerate(edge_raw):
            obj = json.loads(line)
            op = obj['edge']['properties']['operation']
            raw_start = int(obj['edge']['start']['id'])
            raw_end = int(obj['edge']['end']['id'])
            edges.append(PatternEdge(i, op, id_convert[raw_start], id_convert[raw_end]))

        self.nodes = nodes
        self.edges = edges

    
    def get_endpoints(self, eid: int) -> tuple[PatternNode, PatternNode]:
        """Returns the endpoints of the given edge in the order: (start_node, end_node)"""

        edge = self.edges[eid]
        return self.nodes[edge.start], self.nodes[edge.end]
