from PySiddhi.core.SiddhiManager import SiddhiManager
import sys
from time import sleep
import argparse

from preprocess import extract_fields
from gen_siddhi_query import gen_siddhi_app

parser = argparse.ArgumentParser()
parser.description = """
                    Given a pattern, this app will automatically generate the
                    Siddhi query and use Siddhi runtime to process the input
                    event stream to find the pattern.

                    The input stream is read from stdin line by line, and the matched
                    node id and edge id will be logged to stdout.
                    """
parser.add_argument('--pattern_prefix',
                    required=True,
                    type=str,
                    help='The path prefix of pattern\'s files, e.g. pattern/TTP11')

args = parser.parse_args()
regex = args.pattern_prefix.endswith('_regex')
orels = args.pattern_prefix + '_oRels.json' if not regex else args.pattern_prefix[:-6] + '_oRels.json'
siddhiApp = gen_siddhi_app(args.pattern_prefix + '_node.json',
                           args.pattern_prefix + '_edge.json',
                           orels,
                           regex
                           )
siddhiManager = SiddhiManager()
siddhiAppRuntime = siddhiManager.createSiddhiAppRuntime(siddhiApp)

# Retrieving input handler to push events into Siddhi
inputHandler = siddhiAppRuntime.getInputHandler("InputStream")
unorderedInputHandler = siddhiAppRuntime.getInputHandler("UnorderedInputStream")

# Starting event processing
siddhiAppRuntime.start()

time_buffer = []
last_time = ''
for line in sys.stdin:
    fields = extract_fields(line)
    ts = fields[0]
    event = fields[1:]
    if ts == last_time:
        time_buffer.append(event)
        continue
    
    if len(time_buffer) > 1:
        for e in time_buffer:
            unorderedInputHandler.send(e)
    
    time_buffer = [event]
    last_time = ts
    inputHandler.send(event)

# Wait for response
print('Processed all input')
try:
    while True:
        sleep(1)
finally:
    siddhiManager.shutdown()