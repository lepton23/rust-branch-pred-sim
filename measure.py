#!/usr/bin/env python3

import csv
import os
import re
import subprocess
import sys
from collections import defaultdict

from tqdm import tqdm

REPO_DIR = os.path.dirname(os.path.abspath(__file__))
BINARY = os.path.join(REPO_DIR, "target", "release", "p2-bp")
TRACES_DIR = os.path.join(REPO_DIR, "traces")

SIZES = ["512", "1024", "2048", "4096"]

# (algorithm, size-or-None)
PREDICTORS = [("always", None)] + [
    (alg, size)
    for alg in ("twobit", "gshare", "profiled")
    for size in SIZES
]

MISPREDICT_RE = re.compile(r"(\d+) mispredicts over (\d+) conditional branches")
RATE_RE = re.compile(r"([\d.]+)% mispredict rate")


def build():
    print("Building release binary...")
    result = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=REPO_DIR,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print("Build failed:\n", result.stderr)
        sys.exit(1)
    print("Build succeeded.\n")


def run_predictor(trace_path: str, alg: str, size: str | None) -> dict:
    cmd = [BINARY, trace_path, alg]
    if size is not None:
        cmd.append(size)

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(
            f"Predictor run failed ({alg} {size}):\n{result.stderr}"
        )

    output = result.stdout
    m_counts = MISPREDICT_RE.search(output)
    m_rate = RATE_RE.search(output)

    if not m_counts or not m_rate:
        raise RuntimeError(
            f"Could not parse output for {alg} {size}:\n{output}"
        )

    return {
        "mispredicts": int(m_counts.group(1)),
        "total_branches": int(m_counts.group(2)),
        "mispredict_rate": float(m_rate.group(1)),
    }


def main():
    output_dir = "results"
    os.makedirs(output_dir, exist_ok=True)

    build()

    trace_files = sorted(
        f for f in os.listdir(TRACES_DIR) if f.endswith(".out")
    )
    if not trace_files:
        print(f"No .out trace files found in {TRACES_DIR}")
        sys.exit(1)

    jobs = [
        (trace, alg, size)
        for trace in trace_files
        for alg, size in PREDICTORS
    ]

    # rows keyed by algorithm name
    rows: dict[str, list[dict]] = defaultdict(list)
    errors = []

    for trace, alg, size in tqdm(jobs, desc="Running predictors", unit="run"):
        trace_path = os.path.join(TRACES_DIR, trace)
        label = f"{alg}({size})" if size else alg
        try:
            metrics = run_predictor(trace_path, alg, size)
            row = {
                "trace": trace,
                "table_size": size if size else "N/A",
                "mispredicts": metrics["mispredicts"],
                "total_branches": metrics["total_branches"],
                "mispredict_rate_%": metrics["mispredict_rate"],
            }
            rows[alg].append(row)
        except RuntimeError as e:
            tqdm.write(f"ERROR: {trace} / {label}: {e}")
            errors.append((trace, label, str(e)))

    fieldnames = [
        "trace",
        "table_size",
        "mispredicts",
        "total_branches",
        "mispredict_rate_%",
    ]

    for alg, alg_rows in rows.items():
        out_path = os.path.join(output_dir, f"results_{alg}.csv")
        with open(out_path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(alg_rows)
        print(f"Wrote {len(alg_rows)} rows to {out_path}")

    if errors:
        print(f"\n{len(errors)} run(s) failed:")
        for trace, label, msg in errors:
            print(f"  {trace} / {label}: {msg}")


if __name__ == "__main__":
    main()
