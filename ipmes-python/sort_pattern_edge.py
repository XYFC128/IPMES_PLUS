import json
from sys import argv


input_file = argv[1]
lines = open(input_file).readlines()
jsons = [json.loads(line) for line in lines]
output = sorted(jsons, key=lambda obj : obj['edge']['properties']['time'])
for obj in output:
    print(json.dumps(obj))
