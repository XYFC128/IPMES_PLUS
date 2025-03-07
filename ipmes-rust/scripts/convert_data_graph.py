import fileinput
import re
import sys

pattern = re.compile(r'(.+)#(\w+::.+)#(\w+::.+)')

if __name__ == '__main__':
    for line in fileinput.input(inplace=True):
        if len(line) == 0:
            continue

        columns = line.strip().split(',')
        if len(columns) != 6:
            print(line, end='')
            continue

        t1, t2, sig, event_id, subject_id, object_id = columns
        split = sig.split('#')
        if len(split) == 3:
            event_sig, subject_sig, object_sig = split
        else:
            found = pattern.search(sig)
            if found is None:
                print('Bad record:', line.strip(), file=sys.stderr)
                continue
            event_sig = found.group(1)
            subject_sig = found.group(2)
            object_sig = found.group(3)

        print(','.join([t1, t2, event_id, event_sig, subject_id, subject_sig, object_id, object_sig]), end='\n' if line.endswith('\n') else '')
