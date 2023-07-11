'''
The oRels file reprensents the temporal dependencies of the edges in the pattern.
The file format is a json object like this:
{
    "root": {
        "parents": [],
        "children": [
            0,
            3,
            6
        ]
    },
    "0": {
        "parents": ["root"],
        "children": [2]
    }, ...
}

The dependency graph is a connected DAG with a root. The root is a virtual vertex,
not representing any edge. The other vertices in the graph have a number lable n,
corresponding to the n-th edge in the edge file, n starting from 0.

In the dependency graph, the parents are the dependencies of it's childs, meaning
the occurrence of a child must after all of it's parent.
'''

import json

class DependencyGpraph:
    def __init__(self, orels_file: str):
        self.graph = json.load(open(orels_file))

    def get_start_events(self) -> list[int]:
        return self.graph['root']['children']
    
    def get_dependencies(self, eid: int) -> list[int]:
        return self.graph[str(eid)]['parents']
    
    def get_references(self, eid: int) -> list[int]:
        return self.graph[str(eid)]['children']



if __name__ == '__main__':
    import sys
    g = DependencyGpraph(sys.argv[1])