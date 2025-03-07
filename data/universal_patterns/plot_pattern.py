#/usr/bin/env python3
import graphviz
from argparse import ArgumentParser
import json
import os

if __name__ == '__main__':
    parser = ArgumentParser()
    parser.add_argument('-o', '--out-dir',
                        default='.', type=str, help='The output folder')
    parser.add_argument('pattern_file', help='The pattern file to plot')

    args = parser.parse_args()

    pattern = json.load(open(args.pattern_file))
    graph_name = os.path.basename(args.pattern_file).removesuffix('.json')

    dot = graphviz.Digraph(graph_name, format='pdf')  
    for entity in pattern['Entities']:
        id = str(entity['ID'])
        signature = r'ID:{}\n{}'.format(id, entity['Signature'])
        dot.node(id, signature)
    
    for event in pattern['Events']:
        subject = str(event['SubjectID'])
        object = str(event['ObjectID'])
        if 'Type' in event and event['Type'] == 'Flow':
            label = r'ID:{}'.format(event['ID'])
            dot.edge(subject, object, label=label, style='dashed')
        elif 'Frequency' in event:
            signature = event['Signature']
            label = r'ID:{}\n{}\nFrequency:{}'.format(event['ID'], signature, event['Frequency'])
            dot.edge(subject, object, label=label)
        else:
            signature = event['Signature']
            label = r'ID:{}\n{}'.format(event['ID'], signature)
            dot.edge(subject, object, label=label)

    dot.render(view=True, directory=args.out_dir)
