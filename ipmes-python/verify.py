def count_columns(line: str) -> int:
    return len(line.split(','))

if __name__ == '__main__':
    """
    This program verify the preprocessed csv to ensure the
    number of columns is the same in every line. The input
    is read from stdin and verification results will be
    print to stdout.
    """

    import fileinput
    normal_cols = 0
    for line_num, line in enumerate(fileinput.input()):
        cols = count_columns(line)
        if cols != normal_cols:
            print(f'{line_num}: has {cols} columns')
            normal_cols = cols
            