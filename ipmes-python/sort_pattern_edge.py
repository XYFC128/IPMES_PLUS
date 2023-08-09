import json
from sys import argv
import argparse


parser = argparse.ArgumentParser()
parser.description = "sort the pattern's edge file"
parser.add_argument('edge_file',
                    type=str,
                    help='The path to the edge file you want to sort')
parser.add_argument('--darpa',
                    action='store_true',
                    default=False,
                    help='The edge file is in DARPA dataset format')

args = parser.parse_args()

input_file = args.edge_file
lines = open(input_file).readlines()
jsons = [json.loads(line) for line in lines]

if args.darpa:
    key_func = lambda obj : (obj['edge']['properties']['earliest'], obj['edge']['properties']['lastest'])
else:
    key_func = lambda obj : float(obj['edge']['properties']['time'])
output = sorted(jsons, key=key_func)
for obj in output:
    print(json.dumps(obj))
