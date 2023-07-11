import argparse
from textwrap import dedent

from parse_pattern import PatternGraph
from parse_oRels import DependencyGpraph

def gen_header(g: PatternGraph) -> str:
    """
    Generate stream and table definitions
    """

    header = '@App:name("SiddhiApp")\n'
    header += 'define Stream InputStream (eid string, op string, start_id string, start_type string, end_id string, end_type string);\n'

    fields = '('
    output_condition = ''
    for i in range(len(g.nodes)):
        fields += f'n{i}_id string, '
        output_condition += f'n{i}_id != "null" and '

    for i in range(len(g.edges)):
        fields += f'e{i}_id string'
        output_condition += f'e{i}_id != "null"'
        if i != len(g.edges) - 1:
            fields += ', '
            output_condition += ' and '
        else:
            fields += ')'
    
    return dedent(f'''
                  @App:name("SiddhiApp")
                  define Stream InputStream (eid string, op string, start_id string, start_type string, end_id string, end_type string);

                  define Stream CandidateStream {fields};
                  define Table CandidateTable {fields};

                  @sink(type="log")
                  define Stream OutputStream {fields};

                  from CandidateStream
                  select *
                  insert into CandidateTable;

                  from CandidateStream[{output_condition}]
                  select *
                  insert into OutputStream;
                  ''')


def gen_select_expr(g: PatternGraph, eid: int, fmt_node: tuple[str, str, str], fmt_edge: tuple[str, str]) -> str:
    """
    Generate the "select" exptrssion in siddhi query. The selection result will have the same fields as
    the CandidateTable, i.e. (n0_id, n1_id, ..., e0_id, e1_id, ...).

    This function takes 5 format strings to format the output on different scenarios:
    1. The current field is <start node>_id
    2. The current field is <end node>_id
    3. The other nodes
    4. The current field is e<eid>_id
    5. The other edges

    Each format string will be given a named index: field_name. For example, if we want the select expression
    be "start_id as n0_id" when n0 is the start node of the given edge, we can use the following format
    string: "start_id as {field_name}"

    Args:
        g: The pattern graph
        eid: The id of the edge we currently processing
        fmt_node: format strings for the first 3 scenarios
        fmt_edge: format strings for the last 2 scenarios
    Returns:
        complete select expression
    """

    select_expr = ''
    start_nd, end_nd = g.get_endpoints(eid)
    for i in range(len(g.nodes)):
        cur_field = f'n{i}_id'
        if i == start_nd.id:
            select_expr += fmt_node[0].format(field_name = cur_field)
        elif i == end_nd.id:
            select_expr += fmt_node[1].format(field_name = cur_field)
        else:
            select_expr += fmt_node[2].format(field_name = cur_field)
        select_expr += ', '
    
    for i in range(len(g.edges)):
        cur_field = f'e{i}_id'
        if i == eid:
            select_expr += fmt_edge[0].format(field_name = cur_field)
        else:
            select_expr += fmt_edge[1].format(field_name = cur_field)
        select_expr += ', ' if i != len(g.edges) - 1 else ''
    
    return select_expr


def gen_dependency_condition(dep_graph: DependencyGpraph, eid: int) -> str:
    deps = dep_graph.get_dependencies(eid)
    dep_cond = ''
    for dep in deps:
        if dep != 'root':
            dep_cond += f't.e{dep}_id != "null" and '
    return dep_cond


def gen_edge_rules(pat_graph: PatternGraph, dep_graph: DependencyGpraph) -> str:
    """
    Generate the query for all edges.
    
    Args:
        nodes: nodes in the pattern
        edges: edges in the pattern

    Returns:
        edge_rules: query string for all edges
    """

    rules = ''

    for edge in pat_graph.edges:
        start, end = pat_graph.get_endpoints(edge.id)
        start_field = f'n{start.id}_id'
        end_field = f'n{end.id}_id'
        edge_field = f'e{edge.id}_id'

        edge_condition = f'op == "{edge.op}" and start_type == "{start.type}" and end_type == "{end.type}"'
        dep_condition = gen_dependency_condition(dep_graph, edge.id)
        self_select_expr = gen_select_expr(
            pat_graph, edge.id,
            ('start_id as {field_name}', 'end_id as {field_name}', '"null" as {field_name}'),
            ('eid as {field_name}', '"null" as {field_name}')
        )
        merge_select_expr = gen_select_expr(
            pat_graph, edge.id,
            ('s.start_id as {field_name}', 's.end_id as {field_name}', 't.{field_name}'),
            ('s.eid as {field_name}', 't.{field_name}')
        )

        rules += dedent(f'''
                        from InputStream[{edge_condition}]
                        select {self_select_expr}
                        insert into CandidateStream;

                        from InputStream[{edge_condition}] as s join
                            CandidateTable as t
                            on t.{edge_field} == "null" and {dep_condition}
                                ((t.{start_field} == s.start_id and t.{end_field} == s.end_id) or
                                (t.{start_field} == s.start_id and t.{end_field} == "null") or
                                (t.{start_field} == "null" and t.{end_field} == s.end_id))
                        select {merge_select_expr}
                        insert into CandidateStream;
                        ''')
    return rules
    

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--node',
                        required=True,
                        help='node file path')
    parser.add_argument('--edge',
                        required=True,
                        help='edge file path')
    parser.add_argument('--orels',
                        required=True,
                        help='orels file path')

    args = parser.parse_args()
    pattern_graph = PatternGraph(args.node, args.edge)
    dependency_graph = DependencyGpraph(args.orels)
    print(gen_header(pattern_graph))
    print(gen_edge_rules(pattern_graph, dependency_graph))