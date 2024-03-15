import fileinput

if __name__ == '__main__':
    for line in fileinput.input(inplace=True):
        if len(line) == 0:
            continue

        columns = line.strip().split(',')
        if len(columns) != 6:
            print(line, end='')
            continue

        t1, t2, sig, event_id, subject_id, object_id = columns
        event_sig, subject_sig, object_sig = sig.split('#')
        print(','.join([t1, t2, event_id, event_sig, subject_id, subject_sig, object_id, object_sig]), end='\n' if line.endswith('\n') else '')