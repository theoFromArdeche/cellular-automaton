import re
from dataclasses import dataclass
from typing import List


RAW_DATA = r"""
========== Perf improve 0 (base) : ==========
    Finished `release` profile [optimized] target(s) in 0.14s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: Conway)

Execution time: 292.110316095s
Performance: 0.34 timesteps/sec
Cells/sec: 1.37M



========== Perf improve 1: ==========
    Finished `release` profile [optimized] target(s) in 0.14s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: Conway)

Execution time: 205.216862182s
Performance: 0.49 timesteps/sec
Cells/sec: 1.95M



========== Perf improve 2: ==========
    Finished `release` profile [optimized] target(s) in 0.14s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: conway)

Execution time: 15.889493512s
Performance: 6.29 timesteps/sec
Cells/sec: 25.17M



========== Perf improve 3: ==========
    Finished `release` profile [optimized] target(s) in 0.08s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: conway)

Execution time: 15.994609017s
Performance: 6.25 timesteps/sec
Cells/sec: 25.01M



========== Perf improve 4: ==========
    Finished `release` profile [optimized] target(s) in 0.08s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: conway)

Execution time: 8.760514853s
Performance: 11.41 timesteps/sec
Cells/sec: 45.66M



========== Perf improve 5: ==========
    Finished `release` profile [optimized + debuginfo] target(s) in 0.14s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: conway optimized)

Execution time: 2.76714677s
Performance: 36.14 timesteps/sec
Cells/sec: 144.55M



========== Perf improve 6: ==========
    Finished `release` profile [optimized + debuginfo] target(s) in 0.08s
     Running `target/release/trait_ac_ui`
✓ GPU renderer initialized
Simulation completed at timestep 101. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 101
  Active traits:
    0: Energy (rule: conway optimized)

Execution time: 1.320260187s
Performance: 76.50 timesteps/sec
Cells/sec: 306.00M
"""


@dataclass
class Run:
    label: str
    exec_time: float
    timesteps_per_sec: float
    cells_per_sec_m: float


def parse_runs(text: str) -> List[Run]:
    runs = []

    # Split on each performance section
    blocks = re.split(r"=+\s*Perf improve", text)

    for block in blocks:
        if "Execution time:" not in block:
            continue

        # Extract improvement label
        label_match = re.search(r"^\s*(\d+[^:=]*)", block)
        time_match = re.search(r"Execution time:\s*([\d.]+)s", block)
        ts_match = re.search(r"Performance:\s*([\d.]+)\s*timesteps/sec", block)
        cells_match = re.search(r"Cells/sec:\s*([\d.]+)M", block)

        if not all([label_match, time_match, ts_match, cells_match]):
            print("⚠️ Warning: failed to parse a block")
            continue

        runs.append(
            Run(
                label=f"Perf improve {label_match.group(1).strip()}",
                exec_time=float(time_match.group(1)),
                timesteps_per_sec=float(ts_match.group(1)),
                cells_per_sec_m=float(cells_match.group(1)),
            )
        )

    return runs


def print_report(runs):
    if not runs:
        raise RuntimeError("No performance runs parsed.")

    base = runs[0]

    print("\n=== PERFORMANCE IMPROVEMENT REPORT ===\n")
    print(
        f"{'Version':<21}"
        f"{'Time (s)':>10}"
        f"{'Timesteps/s':>15}"
        f"{'Cells/s (M)':>15}"
        f"{'From base':>22}"
        f"{'From prev':>22}"
    )
    print("-" * 120)

    prev = None
    for r in runs:
        base_x = base.exec_time / r.exec_time
        base_pct = (base_x - 1) * 100

        if prev is None:
            prev_x = 1.0
            prev_pct = 0.0
        else:
            prev_x = prev.exec_time / r.exec_time
            prev_pct = (prev_x - 1) * 100

        base_str = f"{base_pct:7.0f}% ({base_x:5.2f}×)"
        prev_str = f"{prev_pct:7.0f}% ({prev_x:5.2f}×)"

        print(
            f"{r.label:<21}"
            f"{r.exec_time:>10.3f}"
            f"{r.timesteps_per_sec:>15.2f}"
            f"{r.cells_per_sec_m:>15.2f}"
            f"{base_str:>22}"
            f"{prev_str:>22}"
        )

        prev = r

    final = runs[-1]
    total_x = base.exec_time / final.exec_time
    total_pct = (total_x - 1) * 100

    print("\n=== SUMMARY ===")
    print(
        f"Final version: {total_pct:.1f}% faster "
        f"({total_x:.1f}× speedup) vs baseline"
    )


if __name__ == "__main__":
    runs = parse_runs(RAW_DATA)
    runs.sort(key=lambda r: r.label)
    print_report(runs)
