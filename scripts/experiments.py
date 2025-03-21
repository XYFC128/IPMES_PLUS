import typing as t
import subprocess
from subprocess import Popen, PIPE
import re
import os
import pandas as pd
import json
import argparse
import asyncio

# IPMES_PLUS = "IPMES_PLUS/"
IPMES_PLUS = "./"
TIMING = "timingsubg/rdf/"
IPMES = "IPMES/"

DATA_GRAPH_DIR = "data/data_graphs/"
OLD_DATA_GRAPH_DIR = "data/old_data_graphs/"
SYNTH_GRAPH_DIR = "data/synthesized_graphs/"
FLOW_DATA_GRAPH_DIR = "data/modified_data_graphs/"

PATTERN_DIR = os.path.join(IPMES_PLUS, "data/universal_patterns/")
OLD_PATTERN_DIR = os.path.join(IPMES, "data/universal_patterns/")
OLD_SUBPATTERN_DIR = os.path.join(TIMING, "data/universal_patterns/subpatterns/")

OUT_DIR = "results/"


def build_ipmes_plus():
    cwd = os.getcwd()
    os.chdir(IPMES_PLUS)
    subprocess.run(
        ["cargo", "build", "--release"],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    os.chdir(cwd)


def build_timing(clean=False):
    cwd = os.getcwd()
    os.chdir(TIMING)
    if clean:
        subprocess.run(
            ["make", "clean"],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    subprocess.run(
        ["make", "-j"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
    )
    os.chdir(cwd)


def build_ipmes():
    cwd = os.getcwd()
    os.chdir(IPMES + "/ipmes-java/")
    subprocess.run(
        ["mvn", "compile"],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    os.chdir(cwd)


def parse_peak_mem_result(peak_mem_result: re.Match[str] | None) -> int:
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

        return int(peak_mem) * multiplier
    else:
        return 0


def run_ipmes_plus(
    pattern_file: str, data_graph: str, window_size: int, pre_run=0, re_run=1, log_level: str=""
) -> t.Union[t.Tuple[int, float, float], None]:
    os.environ["RUST_LOG"] = log_level

    binary = os.path.join(IPMES_PLUS, "target/release/ipmes-rust")
    run_cmd = [binary, pattern_file, data_graph, "-w", str(window_size)]
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

        if log_level != "":
            total_num_state = re.search(r"Total number of states: (\d+)", errs).group(1)

    avg_cpu_time = total_cpu_time / re_run
    num_match = int(num_match)
    peak_mem = parse_peak_mem_result(peak_mem_result)

    if log_level != "":
        return num_match, avg_cpu_time, peak_mem, total_num_state

    return num_match, avg_cpu_time, peak_mem


def run_timing(
    pattern_file: str,
    data_graph: str,
    window_size: int,
    pre_run=0,
    re_run=1,
    max_thread_num: int = 1,
    runtime_record: str = "/dev/null",
) -> t.Union[t.Tuple[int, float, float], None]:
    binary = os.path.join(TIMING, "bin/tirdf")
    subpattern_file = os.path.join(OLD_SUBPATTERN_DIR, os.path.basename(pattern_file))
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
    peak_mem = parse_peak_mem_result(peak_mem_result)
    return num_match, avg_cpu_time, peak_mem


def run_ipmes(
    pattern_path: str,
    graph_path: str,
    window_size: int,
    pre_run=0,
    re_run=1,
    options: str = "",
) -> t.Union[t.Tuple[int, float, float], None]:

    def parse_cpu_time(stderr: str) -> float:
        lines = stderr.strip().split("\n")
        user_time = float(lines[-2].split()[1])
        sys_time = float(lines[-1].split()[1])
        return user_time + sys_time

    ipmes_pom = os.path.join(IPMES, "ipmes-java/pom.xml")
    run_cmd = [
        "bash",
        "-c",
        f'time -p -- mvn -f {ipmes_pom} -q exec:java -Dexec.args="-w {window_size} {pattern_path} {graph_path} {options}"',
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

    num_result = 0
    total_cpu_time = 0
    total_mem_usage = 0
    for _ in range(re_run):
        proc = Popen(run_cmd, stdout=PIPE, stderr=PIPE, encoding="utf-8")
        outs, errs = proc.communicate()
        if proc.wait() != 0:
            print(f"Run failed:\n{errs}")
            return None

        print(outs)

        cpu_time = parse_cpu_time(errs)
        output = json.loads(outs)
        mem_usage = int(output["PeakHeapSize"])
        num_result = int(output["NumResults"])

        total_cpu_time += cpu_time
        total_mem_usage += mem_usage

    return num_result, total_cpu_time / re_run, total_mem_usage / re_run


def run_siddhi(
    pattern_path: str,
    graph_path: str,
    window_size: int,
    pre_run=0,
    re_run=1,
) -> t.Union[t.Tuple[int, float, float], None]:
    return run_ipmes(
        pattern_path,
        graph_path,
        window_size,
        options="--cep",
        pre_run=pre_run,
        re_run=re_run,
    )


def get_pattern_number(pattern_name: str):
    pattern_name = pattern_name.removesuffix(".json").removesuffix("_regex")
    return int(pattern_name[2:])


def exp_freq_effectivess(pre_run=0, re_run=1) -> pd.DataFrame:
    freq_patterns = os.listdir(os.path.join(IPMES_PLUS, "data/freq_patterns/"))
    freq_patterns.sort(key=lambda p: get_pattern_number(p))

    original_result = []
    freq_result = []

    data_graph = os.path.join(DATA_GRAPH_DIR, "attack_raw.csv")
    for pattern in freq_patterns:
        pattern_name = pattern.removesuffix(".json").removesuffix("_regex")

        original_pattern = os.path.join(IPMES_PLUS, "data/universal_patterns/", pattern)
        original_res = run_ipmes_plus(original_pattern, data_graph, 1800, pre_run, re_run)
        if not original_res is None:
            num_match, cpu_time, peak_mem = original_res
            original_result.append(
                [pattern_name, num_match, cpu_time, peak_mem / 2**20]
            )

        freq_pattern = os.path.join(IPMES_PLUS, "data/freq_patterns/", pattern)
        freq_res = run_ipmes_plus(freq_pattern, data_graph, 1800, pre_run, re_run)
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


def exp_flow_effectivess(pre_run=0, re_run=1) -> pd.DataFrame:
    flow_configs = [("SP3", "attack.csv", 1800), ("DP3", "dd3.csv", 1000)]

    original_result = []
    flow_result = []

    for pattern, data_graph, window_size in flow_configs:
        data_graph = os.path.join(FLOW_DATA_GRAPH_DIR, data_graph)
        original_pattern = os.path.join(
            IPMES_PLUS, "data/universal_patterns/", pattern + ".json"
        )
        original_res = run_ipmes_plus(original_pattern, data_graph, window_size, pre_run, re_run)
        if not original_res is None:
            num_match, cpu_time, peak_mem = original_res
            original_result.append([pattern, num_match, cpu_time, peak_mem / 2**20])

        flow_pattern = os.path.join(
            IPMES_PLUS, "data/flow_patterns/", pattern + ".json"
        )
        flow_res = run_ipmes_plus(flow_pattern, data_graph, window_size, pre_run, re_run)
        if not flow_res is None:
            num_match, cpu_time, peak_mem = flow_res
            flow_result.append(
                [pattern + "_flow", num_match, cpu_time, peak_mem / 2**20]
            )

    run_result = original_result + flow_result
    return pd.DataFrame(
        data=run_result,
        columns=["Pattern", "Found Ins.", "CPU Time (sec)", "Peak Memory (MB)"],
    )


async def run_all_patterns(
    app: str,
    data_graph: str,
    patterns: list[str],
    job_sem: asyncio.BoundedSemaphore,
    pre_run=0,
    re_run=1,
) -> t.Tuple[float, float]:
    run_function_map = {
        "ipmes": run_ipmes,
        "ipmes+": run_ipmes_plus,
        "siddhi": run_siddhi,
        "timing": run_timing,
    }

    data_graph_dir = OLD_DATA_GRAPH_DIR
    if app == "ipmes+":
        data_graph_dir = DATA_GRAPH_DIR
    data_graph = os.path.join(data_graph_dir, data_graph + ".csv")

    pattern_dir = OLD_PATTERN_DIR
    if app == "ipmes+":
        pattern_dir = PATTERN_DIR

    async def run(pattern):
        async with job_sem:
            pattern_file = os.path.join(pattern_dir, pattern + "_regex.json")
            window_size = 1800 if pattern.startswith("SP") else 1000
            thread = asyncio.to_thread(
                run_function_map[app],
                pattern_file,
                data_graph,
                window_size,
                pre_run,
                re_run,
            )
            return await thread

    results = []
    for pattern in patterns:
        if app == "timing" and pattern == "DP1":
            continue  # a know bug of timing: it failed to run DP1 pattern on all graph
        results.append(run(pattern))

    total_cpu_time = 0.0
    total_mem_usage = 0.0
    success_runs = 0
    for res in await asyncio.gather(*results):
        if not res is None:
            num_match, cpu_time, peak_mem = res
            total_cpu_time += cpu_time
            total_mem_usage += peak_mem / 2**20
            success_runs += 1

    return total_cpu_time / success_runs, total_mem_usage / success_runs


def exp_matching_efficiency(
    apps: list[str],
    graphs: list[str],
    pre_run=0,
    re_run=1,
    parallel_jobs=1
) -> t.Tuple[pd.DataFrame, pd.DataFrame]:
    spade_graphs = ["attack", "mix", "benign"]
    spade_patterns = [f"SP{i}" for i in range(1, 13)]
    darpa_graphs = ["dd1", "dd2", "dd3", "dd4"]
    darpa_patterns = [f"DP{i}" for i in range(1, 6)]

    job_sem = asyncio.BoundedSemaphore(parallel_jobs)

    all_results = []

    def run_dataset(dataset, patterns):
        for graph in dataset:
            if not graph in graphs:
                continue
            for app in apps:
                all_results.append(
                    run_all_patterns(
                        app, graph, patterns, job_sem, pre_run, re_run
                    )
                )

    async def await_results():
        return await asyncio.gather(*all_results)

    run_dataset(spade_graphs, spade_patterns)
    run_dataset(darpa_graphs, darpa_patterns)

    datasets = spade_graphs + darpa_graphs
    cpu_result = []
    mem_result = []
    for i, (cpu_time, peak_mem) in enumerate(asyncio.run(await_results())):
        if i % len(apps) == 0:
            dataset_id = i // len(apps)
            cpu_result.append([datasets[dataset_id]])
            mem_result.append([datasets[dataset_id]])
        cpu_result[-1].append(cpu_time)
        mem_result[-1].append(peak_mem)

    cpu_df = pd.DataFrame(
        data=cpu_result,
        columns=["Dataset", *apps],
    )
    mem_df = pd.DataFrame(
        data=mem_result,
        columns=["Dataset", *apps],
    )
    return cpu_df, mem_df


def exp_join_layer_optimization(
    pre_run=0,
    re_run=1,
    num_instaces: list[int] = [10, 20, 30, 40, 50]
):
    print(os.getcwd())
    with open("patches/forward.patch", "r") as f:
        subprocess.run(["patch", "-p0"], check=True, stdout=None, stderr=None, stdin=f, cwd=IPMES_PLUS + "src/process_layers/")

    build_ipmes_plus()

    run_result = []
    for n_ins in num_instaces:
        pattern = os.path.join(PATTERN_DIR, f"SP6_regex.json")
        data_graph = os.path.join(SYNTH_GRAPH_DIR, f"DW{n_ins}.csv")
        res = run_ipmes_plus(pattern, data_graph, 1800, pre_run, re_run, "info")
        if not res is None:
            num_match, cpu_time, peak_mem, total_num_state = res
            run_result.append([f"DW{n_ins}", num_match, total_num_state, cpu_time, peak_mem / 2**20])

    df = pd.DataFrame(
        data=run_result,
        columns=[
            "Synthesized Graph",
            "Num Results",
            "Num States",
            "CPU Time (sec)",
            "Peak Memory (MB)",
        ],
    )

    with open("patches/backward.patch", "r") as f:
        # subprocess.run(["patch", "-p0"], check=True, stdout=None, stderr=None, stdin=f, cwd=IPMES_PLUS + "src/process_layers/join_layer/")
        subprocess.run(["patch", "-p0"], check=True, stdout=None, stderr=None, stdin=f, cwd=IPMES_PLUS + "src/process_layers/")


    build_ipmes_plus()

    optimized_run_result = []
    for n_ins in num_instaces:
        pattern = os.path.join(PATTERN_DIR, f"SP6_regex.json")
        data_graph = os.path.join(SYNTH_GRAPH_DIR, f"DW{n_ins}.csv")
        res = run_ipmes_plus(pattern, data_graph, 1800, pre_run, re_run, "info")
        if not res is None:
            num_match, cpu_time, peak_mem, total_num_state = res
            optimized_run_result.append([f"DW{n_ins}", num_match, total_num_state, cpu_time, peak_mem / 2**20])


    df_optimized = pd.DataFrame(
        data=optimized_run_result,
        columns=[
            "Synthesized Graph",
            "Num Results",
            "Num States",
            "CPU Time (sec)",
            "Peak Memory (MB)",
        ],
    )

    return df, df_optimized


def select_list(title: str, msg: str, choices: list[str]) -> list[str]:
    if len(choices) == 0:
        return []

    print("\n" + title)
    for i, ch in enumerate(choices, start=1):
        print(f'[{i}]: {ch}')

    def to_idx(s: str) -> int:
        if s.isdigit():
            idx = int(s) - 1
            if 0 <= idx and idx < len(choices):
                return idx
        return -1

    chosen: list[str] = []
    while len(chosen) == 0:
        s = input(f'{msg} (eg: "1 2 3", "1 2-4", default: "1-{len(choices)}"): ')
        if len(s.strip()) == 0:
            chosen = choices
            break
        for token in s.strip().split():
            ids = token.split('-')
            if len(ids) == 1 and to_idx(ids[0]) >= 0:
                chosen.append(choices[to_idx(ids[0])])
            elif len(ids) == 2:
                start = to_idx(ids[0])
                end = to_idx(ids[1])
                if start < end and start >= 0:
                    chosen += choices[start : end + 1]
    return chosen


def save_table(df: pd.DataFrame, filename: str):
    path = os.path.join(OUT_DIR, filename)
    print(df.to_string(index=False))
    df.to_csv(path, index=False)
    print(f"This table is saved to {path}")


def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
        description="Run experiments",
    )
    parser.add_argument(
        "-r",
        "--re-run",
        default=1,
        type=int,
        help="Number of re-runs to measure CPU time.",
    )
    parser.add_argument(
        "--pre-run",
        default=0,
        type=int,
        help="Number of runs before actual measurement.",
    )
    subparsers = parser.add_subparsers(dest='exp_name', required=True, help='Experiment to run')

    subparsers.add_parser('freq', help='Effectiveness of Frequency-type Event Patterns')
    subparsers.add_parser('flow', help='Effectiveness of Flow-type Event Patterns')

    parser_effi = subparsers.add_parser(
        'efficiency',
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
        help='Efficiency of Matching Low-level Attack Patterns')
    parser_effi.add_argument(
        "-j", "--jobs",
        default=1,
        type=int,
        help="The number of parallel jobs"
    )
    all_apps = ['ipmes+', 'timing', 'ipmes', 'siddhi']
    all_datasets = ['attack', 'mix', 'benign', 'dd1', 'dd2', 'dd3', 'dd4']
    parser_effi.add_argument(
        "--apps",
        choices=all_apps,
        nargs='+',
        help="The apps to run"
    )
    parser_effi.add_argument(
        "--graphs",
        choices=all_datasets,
        nargs='+',
        help="The datasets to run"
    )

    subparsers.add_parser('join', help='Join Layer Optimization')

    args = parser.parse_args()

    print("*** Building applications... ***")
    # build_ipmes()
    build_ipmes_plus()
    # build_timing()
    print("*** Building finished. ***")

    os.makedirs(OUT_DIR, exist_ok=True)

    if args.exp_name == 'freq':
        save_table(exp_freq_effectivess(args.pre_run, args.re_run), "freq_result.csv")

    if args.exp_name == 'flow':
        save_table(exp_flow_effectivess(args.pre_run, args.re_run), "flow_result.csv")

    if args.exp_name == 'join':
        df, df_optimized = exp_join_layer_optimization(args.pre_run, args.re_run)

        print("Before optimization:")
        save_table(df, "join_optim_before.csv")
        print("After optimization:")
        save_table(df_optimized, "join_optim_after.csv")

    if args.exp_name == 'efficiency':
        if args.apps is None:
            apps = select_list("Apps:", "Select apps to run", all_apps)
        else:
            apps = args.apps

        if args.graphs is None:
            graphs = select_list("Datasets:", "Select datasets to run", all_datasets)
        else:
            graphs = args.graphs

        cpu_df, mem_df = exp_matching_efficiency(
            apps,
            graphs,
            args.pre_run,
            args.re_run,
            args.jobs,
        )

        print("Average CPU Time (sec)")
        save_table(cpu_df, "efficiency_cpu.csv")
        print("Average Memory (MB)")
        save_table(mem_df, "efficiency_memory.csv")


if __name__ == "__main__":
    exit(main())
