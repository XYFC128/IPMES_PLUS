import typing as t
import subprocess
from subprocess import Popen, PIPE
import re
import os
import pandas as pd
import json
import argparse

IPMES_PLUS = "IPMES_PLUS/"
TIMING = "timingsubg/rdf"
IPMES = "IPMES/"

DATA_GRAPH_DIR = "data_graphs/"
OLD_DATA_GRAPH_DIR = "old_data_graphs/"

PATTERN_DIR = IPMES_PLUS + "data/universal_patterns/"
OLD_PATTERN_DIR = TIMING + "data/universal_patterns/"
OLD_SUBPATTERN_DIR = TIMING + "data/universal_patterns/subpatterns/"

FLOW_DATA_GRAPH_DIR = 'modified_data_graphs/'

def build_ipmes_plus():
    cwd = os.getcwd()
    os.chdir(IPMES_PLUS)
    subprocess.run(
        ["cargo", "build", "--release"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
    )
    os.chdir(cwd)


def build_timing():
    cwd = os.getcwd()
    os.chdir(TIMING)
    subprocess.run(["make", "clean"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    subprocess.run(["make", "-j"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    os.chdir(cwd)


def build_ipmes():
    cwd = os.getcwd()
    os.chdir(IPMES)
    subprocess.run(["mvn", "compile"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    os.chdir(cwd)


def run_ipmes_plus(
    pattern_file: str, data_graph: str, window_size: int, pre_run=0, re_run=1
) -> t.Union[t.Tuple[int, float, int], None]:
    binary = os.path.join(IPMES_PLUS, "target/release/ipmes-rust")
    run_cmd = [binary, pattern_file, data_graph, "-w", str(window_size), "--silent"]
    print("Running: `{}`".format(" ".join(run_cmd)))

    for _ in range(pre_run):
        proc = Popen(run_cmd, stdout=None, stderr=None, encoding="utf-8")
        proc.wait()

    num_match = "0"
    peak_mem_result = None
    total_cpu_time = 0.0
    for i in range(re_run):
        print(f"Run {i + 1} / {re_run} ...")
        proc = Popen(run_cmd, stdout=PIPE, stderr=PIPE, encoding="utf-8")
        outs, errs = proc.communicate()
        if proc.wait() != 0:
            print(f"Run failed:\n{errs}")
            return None

        print(outs)

        num_match = re.search(r"Total number of matches: (\d+)", outs).group(1)
        cpu_time = re.search(r"CPU time elapsed: (\d+\.\d+) secs", outs).group(1)
        total_cpu_time += float(cpu_time)
        peak_mem_result = re.search(r"Peak memory usage: (\d+) (.)B", outs)

    avg_cpu_time = total_cpu_time / re_run
    num_match = int(num_match)
    if peak_mem_result is not None:
        peak_mem = peak_mem_result.group(1)
        peak_mem_unit = peak_mem_result.group(2)

        multiplier = 1
        if peak_mem_unit == "k":
            multiplier = 2**10
        elif peak_mem_unit == "M":
            multiplier = 2**20
        elif peak_mem_unit == "G":
            multiplier = 2**30
        else:
            print(f"Encounter unknown memory unit: {peak_mem_unit}")

        peak_mem = int(peak_mem) * multiplier
    else:
        peak_mem = 0

    return num_match, avg_cpu_time, peak_mem


def run_timing(
    data_graph: str,
    pattern_file: str,
    window_size: int,
    max_thread_num: int,
    runtime_record: str,
    subpattern_file: str,
    pre_run=0,
    re_run=1,
) -> t.Union[t.Tuple[int, float, int], None]:
    binary = os.path.join(TIMING, "bin/tirdf")
    run_cmd = [
        binary,
        data_graph,
        pattern_file,
        str(window_size),
        str(max_thread_num),
        runtime_record,
        subpattern_file,
    ]
    print("Running: `{}`".format(" ".join(run_cmd)))

    for _ in range(pre_run):
        proc = Popen(run_cmd, stdout=None, stderr=None, encoding="utf-8")
        proc.wait()

    num_match = "0"
    peak_mem_result = None
    total_cpu_time = 0.0
    for i in range(re_run):
        print(f"Run {i + 1} / {re_run} ...")
        proc = Popen(run_cmd, stdout=PIPE, stderr=PIPE, encoding="utf-8")
        outs, errs = proc.communicate()
        if proc.wait() != 0:
            print(f"Run failed:\n{errs}")
            return None

        print(outs)

        num_match = re.search(r"Total number of matches: (\d+)", outs).group(1)
        cpu_time = re.search(r"CPU time elapsed: (\d+\.\d+) secs", outs).group(1)
        total_cpu_time += float(cpu_time)
        peak_mem_result = re.search(r"Peak memory usage: (\d+) (.)B", outs)

    avg_cpu_time = total_cpu_time / re_run
    num_match = int(num_match)
    if peak_mem_result is not None:
        peak_mem = peak_mem_result.group(1)
        peak_mem_unit = peak_mem_result.group(2)

        multiplier = 1
        if peak_mem_unit == "k":
            multiplier = 2**10
        elif peak_mem_unit == "M":
            multiplier = 2**20
        elif peak_mem_unit == "G":
            multiplier = 2**30
        else:
            print(f"Encounter unknown memory unit: {peak_mem_unit}")

        peak_mem = int(peak_mem) * multiplier
    else:
        peak_mem = 0

    return num_match, avg_cpu_time, peak_mem


def run_ipmes(
    pattern_path: str,
    graph_path: str,
    window_size: int,
    options: str = "",
    pre_run=0,
    re_run=1,
) -> t.Union[t.Tuple[int, float, int], None]:

    def parse_cpu_time(stderr: str) -> float:
        lines = stderr.strip().split("\n")
        user_time = float(lines[-2].split()[1])
        sys_time = float(lines[-1].split()[1])
        return user_time + sys_time

    run_cmd = [
        "bash",
        "-c",
        f'time -p -- mvn -q exec:java -Dexec.args="-w {window_size} {pattern_path} {graph_path} {options}"',
    ]
    if re_run > 1:
        print(f"Running ({re_run} times):", " ".join(run_cmd))
    else:
        print("Running:", " ".join(run_cmd))
        if re_run < 1:
            return 0, 0, 0

    for _ in range(pre_run):
        proc = Popen(run_cmd, stdout=None, stderr=None, encoding="utf-8")
        proc.wait()

    total_cpu_time = 0
    total_mem_usage = 0
    for _ in range(re_run):
        proc = Popen(run_cmd, stdout=PIPE, stderr=PIPE, encoding="utf-8")
        outs, errs = proc.communicate()

        cpu_time = parse_cpu_time(errs)
        output = json.loads(outs)
        mem_usage = float(output["PeakHeapSize"]) / 2**20  # convert to MB
        num_result = output["NumResults"]

        total_cpu_time += cpu_time
        total_mem_usage += mem_usage

    return num_result, total_cpu_time / re_run, total_mem_usage / re_run


def get_pattern_number(pattern_name: str):
    pattern_name = pattern_name.removesuffix(".json").removesuffix("_regex")
    return int(pattern_name[2:])


def exp_freq_effectivess() -> pd.DataFrame:
    freq_patterns = os.listdir(os.path.join(IPMES_PLUS, "data/freq_patterns/"))
    freq_patterns.sort(key=lambda p: get_pattern_number(p))

    original_result = []
    freq_result = []

    data_graph = os.path.join(DATA_GRAPH_DIR, "attack_raw.csv")
    for pattern in freq_patterns:
        pattern_name = pattern.removesuffix(".json").removesuffix("_regex")

        original_pattern = os.path.join(IPMES_PLUS, "data/universal_patterns/", pattern)
        original_res = run_ipmes_plus(original_pattern, data_graph, 1800)
        if not original_res is None:
            num_match, cpu_time, peak_mem = original_res
            original_result.append(
                [pattern_name, num_match, cpu_time, peak_mem / 2**20]
            )

        freq_pattern = os.path.join(IPMES_PLUS, "data/freq_patterns/", pattern)
        freq_res = run_ipmes_plus(freq_pattern, data_graph, 1800)
        if not freq_res is None:
            num_match, cpu_time, peak_mem = freq_res
            freq_result.append(
                [pattern_name + "_freq", num_match, cpu_time, peak_mem / 2**20]
            )

    run_result = original_result + freq_result
    return pd.DataFrame(
        data=run_result,
        columns=["Pattern", "Found Ins.", "CPU Time (sec)", "Peak Memory (MB)"],
    )


def exp_flow_effectivess() -> pd.DataFrame:
    flow_configs = [('SP3', 'attack.csv', 1800), ('DP3', 'dd3.csv', 1000)]

    original_result = []
    flow_result = []

    for pattern, data_graph, window_size in flow_configs:
        data_graph = os.path.join(FLOW_DATA_GRAPH_DIR, data_graph)
        original_pattern = os.path.join(IPMES_PLUS, 'data/universal_patterns/', pattern + '.json')
        original_res = run_ipmes_plus(original_pattern, data_graph, window_size)
        if not original_res is None:
            num_match, cpu_time, peak_mem = original_res
            original_result.append([pattern, num_match, cpu_time, peak_mem / 2**20])

        flow_pattern = os.path.join(IPMES_PLUS, 'data/flow_patterns/', pattern + '.json')
        flow_res = run_ipmes_plus(flow_pattern, data_graph, window_size)
        if not flow_res is None:
            num_match, cpu_time, peak_mem = flow_res
            flow_result.append([pattern + '_flow', num_match, cpu_time, peak_mem / 2**20])

    run_result = original_result + flow_result
    return pd.DataFrame(
        data=run_result,
        columns=["Pattern", "Found Ins.", "CPU Time (sec)", "Peak Memory (MB)"],
    )


def exp_matching_efficiency(apps: list[str], args: argparse.Namespace):
    spade_graphs = ["attack", "mix", "benign"]
    darpa_graphs = ["dd1", "dd2", "dd3", "dd4"]

    if not args.no_spade:
        window_size = 1800
        for i in range(1, 13):
            for graph in spade_graphs:
                pattern = os.path.join(PATTERN_DIR, f"SP{i}_regex.json")
                old_pattern = os.path.join(OLD_PATTERN_DIR, f"SP{i}_regex.json")
                old_subpattern = os.path.join(OLD_SUBPATTERN_DIR, f"SP{i}_regex.json")

                data_graph = os.path.join(DATA_GRAPH_DIR, graph + ".csv")
                old_data_graph = os.path.join(OLD_DATA_GRAPH_DIR, graph + ".csv")

                for app in apps:
                    if app == "ipmes+":
                        res = run_ipmes_plus(
                            pattern, data_graph, window_size, args.pre_run, args.re_run
                        )
                    elif app == "timing":
                        res = run_timing(
                            data_graph,
                            pattern,
                            window_size,
                            1,
                            "/dev/null",
                            old_subpattern,
                            args.pre_run,
                            args.re_run,
                        )
                    elif app == "ipmes":
                        

                res = run(pattern, data_graph, 1800, args.pre_run, args.re_run)
                if not res is None:
                    num_match, cpu_time, peak_mem = res
                    run_result.append(
                        [f"SP{i}", graph, num_match, cpu_time, peak_mem / 2**20]
                    )

    if not args.no_darpa:
        for i in range(1, 6):
            for graph in darpa_graphs:
                pattern = os.path.join(args.pattern_dir, f"DP{i}_regex.json")
                data_graph = os.path.join(args.data_graph, graph + ".csv")
                res = run(pattern, data_graph, 1000, args.pre_run, args.re_run)
                if not res is None:
                    num_match, cpu_time, peak_mem = res
                    run_result.append(
                        [f"DP{i}", graph, num_match, cpu_time, peak_mem / 2**20]
                    )

    df = pd.DataFrame(
        data=run_result,
        columns=[
            "Pattern",
            "Data Graph",
            "Num Results",
            "CPU Time (sec)",
            "Peak Memory (MB)",
        ],
    )
    print(df.to_string(index=False))
    out_file = os.path.join(args.out_dir, "run_result.csv")
    df.to_csv(out_file, index=False)
    print(f"This table is saved to {out_file}")


def main():
    parser = parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
        description="Run experiments",
    )

    args = parser.parse_args()

    build_ipmes_plus()
    build_timing()
    print(exp_freq_effectivess().to_string(index=False))


if __name__ == "__main__":
    exit(main())
