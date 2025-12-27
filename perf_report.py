import re
from dataclasses import dataclass
from typing import List


@dataclass
class Run:
    label: str
    project: str  # 'trait_ac' or 'trait_ac_ui'
    exec_time: float
    timesteps_per_sec: float
    cells_per_sec_m: float


def parse_runs(text: str) -> tuple[List[Run], List[Run]]:
    """Parse both trait_ac and trait_ac_ui runs from benchmark output."""
    trait_ac_runs = []
    trait_ac_ui_runs = []
    
    # Split into trait_ac and trait_ac_ui sections
    sections = text.split("==================== trait_ac_ui (graphical interface) ====================")
    
    if len(sections) == 2:
        trait_ac_section = sections[0]
        trait_ac_ui_section = sections[1]
    else:
        # Try to parse whatever we have
        trait_ac_section = text
        trait_ac_ui_section = ""
    
    # Parse trait_ac runs
    trait_ac_runs = parse_section(trait_ac_section, "trait_ac")
    
    # Parse trait_ac_ui runs
    if trait_ac_ui_section:
        trait_ac_ui_runs = parse_section(trait_ac_ui_section, "trait_ac_ui")
    
    return trait_ac_runs, trait_ac_ui_runs


def parse_section(text: str, project: str) -> List[Run]:
    """Parse runs from a specific project section."""
    runs = []
    
    # Split on each performance section
    blocks = re.split(r"=+\s*Perf improve", text)
    
    for block in blocks:
        if "Execution time:" not in block:
            continue
        
        # Extract improvement label
        label_match = re.search(r"^\s*(\d+[^:=]*)", block)
        # Match time in either seconds (s) or milliseconds (ms)
        time_match = re.search(r"Execution time:\s*([\d.]+)(m?s)", block)
        ts_match = re.search(r"Performance:\s*([\d.]+)\s*timesteps/sec", block)
        cells_match = re.search(r"Cells/sec:\s*([\d.]+)M", block)
        
        if not all([label_match, time_match, ts_match, cells_match]):
            continue
        
        label = label_match.group(1).strip()
        # Clean up label (remove extra colons, "base", etc.)
        label = label.replace(":", "").strip()
        if "(base)" in label:
            label = "0"
        
        # Convert milliseconds to seconds if needed
        exec_time = float(time_match.group(1))
        if time_match.group(2) == "ms":
            exec_time = exec_time / 1000.0
        
        runs.append(
            Run(
                label=f"Perf improve {label}",
                project=project,
                exec_time=exec_time,
                timesteps_per_sec=float(ts_match.group(1)),
                cells_per_sec_m=float(cells_match.group(1)),
            )
        )
    
    return runs


def print_report(runs: List[Run], project_name: str):
    """Print performance report for a set of runs."""
    if not runs:
        print(f"\n⚠️  No {project_name} runs found\n")
        return
    
    base = runs[0]
    
    print(f"\n{'='*120}")
    print(f"=== {project_name.upper()} PERFORMANCE IMPROVEMENT REPORT ===")
    print(f"{'='*120}\n")
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
    
    print("\n" + "="*50)
    print(f"SUMMARY for {project_name}:")
    print(
        f"  Final version is {total_pct:.1f}% faster "
        f"({total_x:.1f}× speedup) vs baseline"
    )
    print(f"  Time reduced from {base.exec_time:.2f}s to {final.exec_time:.2f}s")
    print("="*50 + "\n")


def print_ui_lib_breakdown(lib_runs: List[Run], ui_runs: List[Run]):
    """Print breakdown of UI vs Library time and pure UI improvements."""
    if not lib_runs or not ui_runs:
        print("\n⚠️  Cannot compute UI/Library breakdown - missing data\n")
        return
    
    print(f"\n{'='*140}")
    print("=== UI vs LIBRARY TIME BREAKDOWN ===")
    print(f"{'='*140}\n")
    print(
        f"{'Version':<18}"
        f"{'UI Total (s)':>13}"
        f"{'Lib Time (s)':>13}"
        f"{'Pure UI (s)':>13}"
        f"{'Lib %':>10}"
        f"{'UI %':>10}"
        f"{'Pure UI Δ base':>20}"
        f"{'Pure UI Δ prev':>20}"
    )
    print("-" * 140)
    
    # Create lookup for library times
    lib_times = {int(re.search(r'\d+', r.label).group()): r.exec_time for r in lib_runs}
    
    prev_ui_overhead = None
    base_ui_overhead = None
    
    for ui_run in ui_runs:
        version = int(re.search(r'\d+', ui_run.label).group())
        
        if version not in lib_times:
            print(f"⚠️  Warning: No library benchmark for version {version}")
            continue
        
        lib_time = lib_times[version]
        ui_total = ui_run.exec_time
        pure_ui = ui_total - lib_time
        
        lib_pct = (lib_time / ui_total) * 100
        ui_pct = (pure_ui / ui_total) * 100
        
        if base_ui_overhead is None:
            base_ui_overhead = pure_ui
            base_str = "      - (baseline)"
            prev_str = "      - (baseline)"
        else:
            # Improvement in pure UI overhead (speedup from base)
            base_x = base_ui_overhead / pure_ui if pure_ui > 0 else float('inf')
            base_improvement = (base_x - 1) * 100
            base_str = f"{base_improvement:6.1f}% ({base_x:4.2f}×)"
            
            if prev_ui_overhead is not None:
                prev_x = prev_ui_overhead / pure_ui if pure_ui > 0 else float('inf')
                prev_improvement = (prev_x - 1) * 100
                prev_str = f"{prev_improvement:6.1f}% ({prev_x:4.2f}×)"
            else:
                prev_str = "      -"
        
        print(
            f"{ui_run.label:<18}"
            f"{ui_total:>13.3f}"
            f"{lib_time:>13.3f}"
            f"{pure_ui:>13.3f}"
            f"{lib_pct:>9.1f}%"
            f"{ui_pct:>9.1f}%"
            f"{base_str:>20}"
            f"{prev_str:>20}"
        )
        
        prev_ui_overhead = pure_ui
    
    # Summary
    first_version = int(re.search(r'\d+', ui_runs[0].label).group())
    last_version = int(re.search(r'\d+', ui_runs[-1].label).group())
    
    first_ui_overhead = ui_runs[0].exec_time - lib_times[first_version]
    last_ui_overhead = ui_runs[-1].exec_time - lib_times[last_version]
    
    ui_overhead_speedup = first_ui_overhead / last_ui_overhead if last_ui_overhead > 0 else float('inf')
    ui_overhead_improvement = (ui_overhead_speedup - 1) * 100
    
    print("\n" + "="*70)
    print("PURE UI OVERHEAD IMPROVEMENT:")
    print(f"  Base UI overhead: {first_ui_overhead:.3f}s")
    print(f"  Final UI overhead: {last_ui_overhead:.3f}s")
    print(f"  Improvement: {ui_overhead_improvement:.1f}% ({ui_overhead_speedup:.1f}× speedup)")
    print("="*70 + "\n")


def main():
    # Read from benchmarks.out
    try:
        with open("benchmarks.out", "r") as f:
            raw_data = f.read()
    except FileNotFoundError:
        print("❌ Error: benchmarks.out not found")
        print("   Make sure you run this script from the same directory as benchmarks.out")
        return
    
    # Parse both project runs
    trait_ac_runs, trait_ac_ui_runs = parse_runs(raw_data)
    
    # Sort by version number
    trait_ac_runs.sort(key=lambda r: int(re.search(r'\d+', r.label).group()))
    trait_ac_ui_runs.sort(key=lambda r: int(re.search(r'\d+', r.label).group()))
    
    # Print reports
    print_report(trait_ac_runs, "trait_ac (library)")
    print_report(trait_ac_ui_runs, "trait_ac_ui (graphical interface)")
    
    # Print UI vs Library breakdown
    print_ui_lib_breakdown(trait_ac_runs, trait_ac_ui_runs)
    
    # Overall comparison
    if trait_ac_runs and trait_ac_ui_runs:
        print("\n" + "="*120)
        print("=== OVERALL COMPARISON ===")
        print("="*120)
        lib_speedup = trait_ac_runs[0].exec_time / trait_ac_runs[-1].exec_time
        ui_speedup = trait_ac_ui_runs[0].exec_time / trait_ac_ui_runs[-1].exec_time
        
        # Calculate pure UI speedup
        first_version = int(re.search(r'\d+', trait_ac_ui_runs[0].label).group())
        last_version = int(re.search(r'\d+', trait_ac_ui_runs[-1].label).group())
        lib_times = {int(re.search(r'\d+', r.label).group()): r.exec_time for r in trait_ac_runs}
        
        first_ui_overhead = trait_ac_ui_runs[0].exec_time - lib_times[first_version]
        last_ui_overhead = trait_ac_ui_runs[-1].exec_time - lib_times[last_version]
        pure_ui_speedup = first_ui_overhead / last_ui_overhead if last_ui_overhead > 0 else float('inf')
        
        print(f"\ntrait_ac (library)      - Final speedup: {lib_speedup:.1f}×")
        print(f"trait_ac_ui (GUI total) - Final speedup: {ui_speedup:.1f}×")
        print(f"trait_ac_ui (UI only)   - Final speedup: {pure_ui_speedup:.1f}×")
        print()


if __name__ == "__main__":
    main()