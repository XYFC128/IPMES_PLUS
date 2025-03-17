import typing as t
import subprocess
from subprocess import Popen, PIPE
import re
import os
import pandas as pd
import json
import argparse
import asyncio

IPMES_PLUS = "IPMES_PLUS/"
TIMING = "timingsubg/rdf/"
IPMES = "IPMES/"

DATA_GRAPH_DIR = "data_graphs/"
OLD_DATA_GRAPH_DIR = "old_data_graphs/"
SYNTH_GRAPH_DIR = "synthesized_graphs/"
FLOW_DATA_GRAPH_DIR = "modified_data_graphs/"

PATTERN_DIR = os.path.join(IPMES_PLUS, "data/universal_patterns/")
OLD_PATTERN_DIR = os.path.join(IPMES, "data/universal_patterns/")
OLD_SUBPATTERN_DIR = os.path.join(TIMING, "data/universal_patterns/subpatterns/")


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


def build_timing():
    cwd = os.getcwd()
    os.chdir(TIMING)
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
    pattern_file: str, data_graph: str, window_size: int, pre_run=0, re_run=1
) -> t.Union[t.Tuple[int, float, float], None]:
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
    peak_mem = parse_peak_mem_result(peak_mem_result)
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
    flow_configs = [("SP3", "attack.csv", 1800), ("DP3", "dd3.csv", 1000)]

    original_result = []
    flow_result = []

    for pattern, data_graph, window_size in flow_configs:
        data_graph = os.path.join(FLOW_DATA_GRAPH_DIR, data_graph)
        original_pattern = os.path.join(
            IPMES_PLUS, "data/universal_patterns/", pattern + ".json"
        )
        original_res = run_ipmes_plus(original_pattern, data_graph, window_size)
        if not original_res is None:
            num_match, cpu_time, peak_mem = original_res
            original_result.append([pattern, num_match, cpu_time, peak_mem / 2**20])

        flow_pattern = os.path.join(
            IPMES_PLUS, "data/flow_patterns/", pattern + ".json"
        )
        flow_res = run_ipmes_plus(flow_pattern, data_graph, window_size)
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


def select_app_graph(
    args: argparse.Namespace,
) -> t.Tuple[t.List[str], t.List[str], t.List[str]]:
    apps_table = {"0": "ipmes+", "1": "ipmes", "2": "timing", "3": "siddhi"}
    apps = []
    apps_id = args.match_app_list.split(",")

    for id in apps_id:
        if id in apps_table:
            apps.append(apps_table[id])

    spade_graphs = ["attack", "mix", "benign"]
    darpa_graphs = ["dd1", "dd2", "dd3", "dd4"]
    spade = []
    darpa = []
    graphs_id = args.match_graph_list.split(",")

    for graph in graphs_id:
        if graph in spade_graphs:
            spade.append(graph)
        elif graph in darpa_graphs:
            darpa.append(graph)

    return apps, spade, darpa


def exp_matching_efficiency(
    args: argparse.Namespace, parallel_jobs=1
) -> t.Tuple[pd.DataFrame, pd.DataFrame]:
    spade_patterns = [f"SP{i}" for i in range(1, 13)]
    darpa_patterns = [f"DP{i}" for i in range(1, 6)]

    apps, spade_graphs, darpa_graphs = select_app_graph(args)

    job_sem = asyncio.BoundedSemaphore(parallel_jobs)

    all_results = []

    def run_dataset(dataset, patterns):
        for graph in dataset:
            for app in apps:
                all_results.append(
                    run_all_patterns(
                        app, graph, patterns, job_sem, args.pre_run, args.re_run
                    )
                )

    async def await_results():
        return await asyncio.gather(*all_results)

    if not args.no_spade:
        run_dataset(spade_graphs, spade_patterns)

    if not args.no_darpa:
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
    args: argparse.Namespace, num_instaces: list[int] = [10, 20, 30, 40, 50]
):
    with open("patches/forward.patch", "r") as f:
        subprocess.run(["patch"], check=True, stdout=None, stderr=None, stdin=f, cwd=IPMES_PLUS + "src/process_layers/join_layer/")

    build_ipmes_plus()

    run_result = []
    for n_ins in num_instaces:
        pattern = os.path.join(PATTERN_DIR, f"SP6_regex.json")
        data_graph = os.path.join(SYNTH_GRAPH_DIR, f"DW{n_ins}.csv")
        res = run_ipmes_plus(pattern, data_graph, 1800, args.pre_run, args.re_run)
        if not res is None:
            num_match, cpu_time, peak_mem = res
            run_result.append([f"DW{n_ins}", num_match, cpu_time, peak_mem / 2**20])

    df = pd.DataFrame(
        data=run_result,
        columns=[
            "Synthesized Graph",
            "Num Results",
            "CPU Time (sec)",
            "Peak Memory (MB)",
        ],
    )

    with open("patches/backward.patch", "r") as f:
        subprocess.run(["patch"], check=True, stdout=None, stderr=None, stdin=f, cwd=IPMES_PLUS + "src/process_layers/join_layer/")

    build_ipmes_plus()

    optimized_run_result = []
    for n_ins in num_instaces:
        pattern = os.path.join(PATTERN_DIR, f"SP6_regex.json")
        data_graph = os.path.join(SYNTH_GRAPH_DIR, f"DW{n_ins}.csv")
        res = run_ipmes_plus(pattern, data_graph, 1800, args.pre_run, args.re_run)
        if not res is None:
            num_match, cpu_time, peak_mem = res
            optimized_run_result.append(
                [f"DW{n_ins}", num_match, cpu_time, peak_mem / 2**20]
            )

    df_optimized = pd.DataFrame(
        data=optimized_run_result,
        columns=[
            "Synthesized Graph",
            "Num Results",
            "CPU Time (sec)",
            "Peak Memory (MB)",
        ],
    )

    return df, df_optimized


def main():
    parser = parser = argparse.ArgumentParser(
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
    parser.add_argument(
        "--no-darpa",
        default=False,
        action="store_true",
        help="Do not run on DARPA data graphs (for the Matching Efficiency experiment).",
    )
    parser.add_argument(
        "--no-spade",
        default=False,
        action="store_true",
        help="Do not run on SPADE data graphs (for the Matching Efficiency experiment).",
    )
    parser.add_argument(
        "--freq",
        default=False,
        action="store_true",
        help='Run experiment: "Effectiveness of Frequency-type Event Patterns".',
    )
    parser.add_argument(
        "--flow",
        default=False,
        action="store_true",
        help='Run experiment: "Effectiveness of Flow-type Event Patterns".',
    )
    parser.add_argument(
        "--join",
        default=False,
        action="store_true",
        help='Run experiment: "Join Layer Optimization".',
    )
    parser.add_argument(
        "-a",
        "--match-app-list",
        default="0,1,2,3",
        type=str,
        help="""Comma separated list of application(s) to run, where
            [0: IPMES+, 1: IPMES, 2: Timing, 3: Siddhi].
            For example, ``-a 0,2'' means running the Mathcing Efficiency experiment with IPMES+ and Timing.
            """,
    )
    parser.add_argument(
        "-g",
        "--match-graph-list",
        default="attack,mix,benign,dd1,dd2,dd3,dd4",
        type=str,
        help="""Comma separated list of graph to run on.
            For example, "-g attack,dd3" means running the Mathcing Efficiency experiment on data graphs "attack" and "dd3".
            """,
    )
    args = parser.parse_args()

    print("*** Building applications... ***")
    build_ipmes()
    build_ipmes_plus()
    build_timing()
    print("*** Building finished. ***")

    if args.freq:
        print(exp_freq_effectivess().to_string(index=False))

    if args.flow:
        print(exp_flow_effectivess().to_string(index=False))

    if args.join:
        df, df_optimized = exp_join_layer_optimization(args)

        print("Before optimization:")
        print(df.to_string(index=False))
        print("After optimization:")
        print(df_optimized.to_string(index=False))

    if not (args.freq or args.flow or args.join):
        # The experiment "Efficiency of Matching Low-level Attack Patterns" is run by default,
        # if no other experiments have been conducted.
        cpu_df, mem_df = exp_matching_efficiency(args, 4)

        print("CPU Time (sec)")
        print(cpu_df.to_string(index=False))
        print("Memory (MB)")
        print(mem_df.to_string(index=False))


if __name__ == "__main__":
    exit(main())
