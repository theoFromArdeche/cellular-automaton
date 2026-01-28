#!/usr/bin/env python3
"""
Benchmark Analysis Script for Cellular Automata Project
Parses benchmark output and generates performance reports + LaTeX summary
"""

import re
import sys
from dataclasses import dataclass
from typing import List, Optional, Tuple
from datetime import datetime


@dataclass
class TraitConfig:
    name: str  # e.g., "Energy", "Charge" (None = any name accepted)
    allowed_rules: List[str]  # e.g., ["conway", "conway optimized"]


@dataclass
class BenchmarkConfig:
    grid_width: int
    grid_height: int
    timesteps: int
    traits: List[TraitConfig]  # Expected traits with their allowed rules


@dataclass
class ParsedTrait:
    index: int
    name: str
    rule: str


@dataclass
class Run:
    version: int
    commit: str
    commit_date: str
    commit_msg: str
    project: str  # 'trait_ac' or 'trait_ac_ui'
    exec_time: float
    timesteps_per_sec: float
    cells_per_sec_m: float
    grid_width: int
    grid_height: int
    timesteps: int
    traits: List[ParsedTrait]


def parse_time(time_str: str, unit: str) -> float:
    """Convert time string to seconds."""
    value = float(time_str)
    if unit == "ms":
        return value / 1000.0
    return value


def parse_benchmarks(text: str, expected_config: BenchmarkConfig) -> Tuple[List[Run], List[Run], List[str]]:
    """Parse benchmark output and return trait_ac runs, trait_ac_ui runs, and errors."""
    trait_ac_runs = []
    trait_ac_ui_runs = []
    errors = []
    
    # Split by CODE VERSION blocks
    version_blocks = re.split(r"={20,}\s*CODE VERSION\s+(\d+)\s*={20,}", text)
    
    # version_blocks[0] is preamble, then pairs of (version_num, content)
    for i in range(1, len(version_blocks), 2):
        if i + 1 >= len(version_blocks):
            break
            
        version_num = int(version_blocks[i])
        content = version_blocks[i + 1]
        
        # Extract commit info
        commit_match = re.search(r"Commit:\s*(\w+)", content)
        date_match = re.search(r"Date:\s*([\d\-\s:+]+)", content)
        msg_match = re.search(r"Msg:\s*(.+?)(?=\n\n|={10,})", content, re.DOTALL)
        
        commit = commit_match.group(1) if commit_match else "unknown"
        commit_date = date_match.group(1).strip() if date_match else "unknown"
        commit_msg = msg_match.group(1).strip() if msg_match else "unknown"
        
        # Split into trait_ac and trait_ac_ui sections
        benchmark_sections = re.split(r"={10,}\s*BENCHMARK\s+(trait_ac(?:_ui)?)\s*\|\s*VERSION\s*\d+\s*={10,}", content)
        
        for j in range(1, len(benchmark_sections), 2):
            if j + 1 >= len(benchmark_sections):
                break
                
            project = benchmark_sections[j]
            section_content = benchmark_sections[j + 1]
            
            # Parse grid and timesteps
            grid_match = re.search(r"Grid:\s*(\d+)x(\d+)", section_content)
            timesteps_match = re.search(r"Timesteps:\s*(\d+)", section_content)
            
            if not grid_match or not timesteps_match:
                errors.append(f"Version {version_num} ({project}): Could not parse grid/timesteps")
                continue
            
            grid_width = int(grid_match.group(1))
            grid_height = int(grid_match.group(2))
            timesteps = int(timesteps_match.group(1))
            
            # Parse all traits: "N: Name (rule: rulename)"
            trait_pattern = r"(\d+):\s*(\w+)\s*\(rule:\s*([^)]+)\)"
            trait_matches = re.findall(trait_pattern, section_content)
            
            parsed_traits = []
            for idx_str, name, rule in trait_matches:
                parsed_traits.append(ParsedTrait(
                    index=int(idx_str),
                    name=name.strip(),
                    rule=rule.strip().lower()
                ))
            
            if not parsed_traits:
                errors.append(f"Version {version_num} ({project}): Could not parse any traits")
                continue
            
            # Validate configuration
            if grid_width != expected_config.grid_width or grid_height != expected_config.grid_height:
                errors.append(f"Version {version_num} ({project}): Grid size mismatch - expected {expected_config.grid_width}x{expected_config.grid_height}, got {grid_width}x{grid_height}")
            
            if timesteps != expected_config.timesteps:
                errors.append(f"Version {version_num} ({project}): Timesteps mismatch - expected {expected_config.timesteps}, got {timesteps}")
            
            # Validate number of traits
            if len(parsed_traits) != len(expected_config.traits):
                errors.append(f"Version {version_num} ({project}): Trait count mismatch - expected {len(expected_config.traits)}, got {len(parsed_traits)}")
            else:
                # Validate each trait
                for idx, (parsed, expected) in enumerate(zip(parsed_traits, expected_config.traits)):
                    # Check trait name if specified (None or empty = any name accepted)
                    if expected.name and parsed.name.lower() != expected.name.lower():
                        errors.append(f"Version {version_num} ({project}): Trait {idx} name mismatch - expected '{expected.name}', got '{parsed.name}'")
                    
                    # Check rule is in allowed list
                    allowed_rules_lower = [r.lower() for r in expected.allowed_rules]
                    if parsed.rule not in allowed_rules_lower:
                        allowed_str = ", ".join(expected.allowed_rules)
                        errors.append(f"Version {version_num} ({project}): Trait {idx} ({parsed.name}) rule mismatch - expected one of [{allowed_str}], got '{parsed.rule}'")
            
            # Parse performance metrics
            time_match = re.search(r"Execution time:\s*([\d.]+)(m?s)", section_content)
            perf_match = re.search(r"Performance:\s*([\d.]+)\s*timesteps/sec", section_content)
            cells_match = re.search(r"Cells/sec:\s*([\d.]+)M", section_content)
            
            if not all([time_match, perf_match, cells_match]):
                errors.append(f"Version {version_num} ({project}): Could not parse performance metrics")
                continue
            
            exec_time = parse_time(time_match.group(1), time_match.group(2))
            
            run = Run(
                version=version_num,
                commit=commit,
                commit_date=commit_date,
                commit_msg=commit_msg,
                project=project,
                exec_time=exec_time,
                timesteps_per_sec=float(perf_match.group(1)),
                cells_per_sec_m=float(cells_match.group(1)),
                grid_width=grid_width,
                grid_height=grid_height,
                timesteps=timesteps,
                traits=parsed_traits
            )
            
            if project == "trait_ac":
                trait_ac_runs.append(run)
            else:
                trait_ac_ui_runs.append(run)
    
    # Sort by version
    trait_ac_runs.sort(key=lambda r: r.version)
    trait_ac_ui_runs.sort(key=lambda r: r.version)
    
    return trait_ac_runs, trait_ac_ui_runs, errors


def print_report(runs: List[Run], project_name: str):
    """Print performance report for a set of runs."""
    if not runs:
        print(f"\n⚠️  No {project_name} runs found\n")
        return
    
    base = runs[0]
    
    print(f"\n{'='*115}")
    print(f"=== {project_name.upper()} PERFORMANCE IMPROVEMENT REPORT ===")
    print(f"{'='*115}\n")
    print(
        f"{'Version':<10}"
        f"{'Date':<12}"
        f"{'Time (s)':>12}"
        f"{'Timesteps/s':>15}"
        f"{'Cells/s (M)':>15}"
        f"{'From base':>22}"
        f"{'From prev':>22}"
    )
    print("-" * 115)
    
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
        
        base_str = f"{base_pct:7.0f}% ({base_x:6.2f}×)" if r != base else "    - (baseline)"
        prev_str = f"{prev_pct:7.0f}% ({prev_x:6.2f}×)" if prev else "    - (baseline)"
        
        # Extract just the date (YYYY-MM-DD)
        date_str = r.commit_date[:10] if len(r.commit_date) >= 10 else r.commit_date
        
        print(
            f"v{r.version:<9}"
            f"{date_str:<12}"
            f"{r.exec_time:>12.3f}"
            f"{r.timesteps_per_sec:>15.2f}"
            f"{r.cells_per_sec_m:>15.2f}"
            f"{base_str:>22}"
            f"{prev_str:>22}"
        )
        
        prev = r
        prev = r
    
    final = runs[-1]
    total_x = base.exec_time / final.exec_time
    total_pct = (total_x - 1) * 100
    
    print("\n" + "="*60)
    print(f"SUMMARY for {project_name}:")
    print(f"  Final version is {total_pct:.1f}% faster ({total_x:.1f}× speedup) vs baseline")
    print(f"  Time reduced from {base.exec_time:.3f}s to {final.exec_time:.3f}s")
    print("="*60 + "\n")


def print_ui_lib_breakdown(lib_runs: List[Run], ui_runs: List[Run]):
    """Print breakdown of UI vs Library time and pure UI improvements."""
    if not lib_runs or not ui_runs:
        print("\n⚠️  Cannot compute UI/Library breakdown - missing data\n")
        return
    
    print(f"\n{'='*160}")
    print("=== UI vs LIBRARY TIME BREAKDOWN ===")
    print(f"{'='*160}\n")
    print(
        f"{'Version':<10}"
        f"{'Date':<12}"
        f"{'UI Total (s)':>14}"
        f"{'Lib Time (s)':>14}"
        f"{'Pure UI (s)':>14}"
        f"{'Lib %':>10}"
        f"{'UI %':>10}"
        f"{'Pure UI Δ base':>22}"
        f"{'Pure UI Δ prev':>22}"
    )
    print("-" * 160)
    
    # Create lookup for library times and dates
    lib_times = {r.version: r.exec_time for r in lib_runs}
    ui_dates = {r.version: r.commit_date[:10] if len(r.commit_date) >= 10 else r.commit_date for r in ui_runs}
    
    prev_ui_overhead = None
    base_ui_overhead = None
    
    for ui_run in ui_runs:
        version = ui_run.version
        
        if version not in lib_times:
            print(f"⚠️  Warning: No library benchmark for version {version}")
            continue
        
        lib_time = lib_times[version]
        ui_total = ui_run.exec_time
        pure_ui = ui_total - lib_time
        date_str = ui_dates.get(version, "")
        
        lib_pct = (lib_time / ui_total) * 100
        ui_pct = (pure_ui / ui_total) * 100
        
        if base_ui_overhead is None:
            base_ui_overhead = pure_ui
            base_str = "      - (baseline)"
            prev_str = "      - (baseline)"
        else:
            base_x = base_ui_overhead / pure_ui if pure_ui > 0 else float('inf')
            base_improvement = (base_x - 1) * 100
            base_str = f"{base_improvement:7.1f}% ({base_x:5.2f}×)"
            
            if prev_ui_overhead is not None:
                prev_x = prev_ui_overhead / pure_ui if pure_ui > 0 else float('inf')
                prev_improvement = (prev_x - 1) * 100
                prev_str = f"{prev_improvement:7.1f}% ({prev_x:5.2f}×)"
            else:
                prev_str = "      -"
        
        print(
            f"v{version:<9}"
            f"{date_str:<12}"
            f"{ui_total:>14.3f}"
            f"{lib_time:>14.3f}"
            f"{pure_ui:>14.3f}"
            f"{lib_pct:>9.1f}%"
            f"{ui_pct:>9.1f}%"
            f"{base_str:>22}"
            f"{prev_str:>22}"
        )
        
        prev_ui_overhead = pure_ui
    
    # Summary
    first_ui_overhead = ui_runs[0].exec_time - lib_times[ui_runs[0].version]
    last_ui_overhead = ui_runs[-1].exec_time - lib_times[ui_runs[-1].version]
    
    ui_overhead_speedup = first_ui_overhead / last_ui_overhead if last_ui_overhead > 0 else float('inf')
    ui_overhead_improvement = (ui_overhead_speedup - 1) * 100
    
    print("\n" + "="*70)
    print("PURE UI OVERHEAD IMPROVEMENT:")
    print(f"  Base UI overhead: {first_ui_overhead:.3f}s")
    print(f"  Final UI overhead: {last_ui_overhead:.3f}s")
    print(f"  Improvement: {ui_overhead_improvement:.1f}% ({ui_overhead_speedup:.1f}× speedup)")
    print("="*70 + "\n")


def generate_latex(lib_runs: List[Run], ui_runs: List[Run], errors: List[str], output_path: str, expected_config: BenchmarkConfig):
    """Generate LaTeX report."""
    
    lib_times = {r.version: r.exec_time for r in lib_runs}
    total_cells = expected_config.grid_width * expected_config.grid_height
    total_cells_formatted = f"{total_cells:,}"
    
    # Build traits description for LaTeX
    traits_latex = ""
    for i, trait in enumerate(expected_config.traits):
        name_str = trait.name if trait.name else "(any)"
        rules_str = ", ".join(trait.allowed_rules)
        traits_latex += f"        \\item Trait {i}: {name_str} $\\rightarrow$ allowed rules: {rules_str}\n"
    
    latex = r"""\documentclass[11pt,a4paper]{article}
\usepackage[utf8]{inputenc}
\usepackage[T1]{fontenc}
\usepackage{booktabs}
\usepackage{tabularx}
\usepackage{geometry}
\usepackage{xcolor}
\usepackage{hyperref}
\usepackage{graphicx}
\usepackage{siunitx}
\usepackage{float}

\geometry{margin=2cm}
\sisetup{group-separator={,}, group-minimum-digits=4}

\title{Cellular Automata Performance Benchmark Report}
\author{Auto-generated Analysis}
\date{""" + datetime.now().strftime("%B %d, %Y") + r"""}

\begin{document}
\maketitle

\section{Test Configuration}

All benchmarks were executed with the following configuration:
\begin{itemize}
    \item \textbf{Grid Size:} $""" + str(expected_config.grid_width) + r" \times " + str(expected_config.grid_height) + r"$ cells (" + total_cells_formatted + r""" total)
    \item \textbf{Timesteps:} """ + str(expected_config.timesteps) + r"""
    \item \textbf{Traits:} """ + str(len(expected_config.traits)) + r"""
    \begin{itemize}
""" + traits_latex + r"""    \end{itemize}
\end{itemize}

\textbf{Note:} The versions correspond to Git commits but have been modified for benchmarking purposes. 
The modifications (diffs) are available in the \texttt{benchmark\_diffs/} folder, with a summary in \texttt{benchmark\_diffs/SUMMARY.md}.

"""

    # Configuration validation
    if errors:
        latex += r"""
\subsection{Configuration Warnings}
\textcolor{red}{The following configuration mismatches were detected:}
\begin{itemize}
"""
        for error in errors:
            latex += f"    \\item {error.replace('_', r'\_')}\n"
        latex += r"""\end{itemize}
"""
    else:
        latex += r"""
\subsection{Configuration Validation}
\textcolor{green!50!black}{All versions conform to the expected test configuration.}
"""

    # Library performance table
    latex += r"""
\section{Library Performance (\texttt{trait\_ac})}

This section shows the performance of the core simulation library without graphical interface overhead.

\begin{table}[H]
\centering
\begin{tabular}{ccrrrrr}
\toprule
\textbf{Version} & \textbf{Date} & \textbf{Time (s)} & \textbf{Steps/s} & \textbf{MCells/s} & \textbf{vs Base} & \textbf{vs Prev} \\
\midrule
"""
    
    prev = None
    base = lib_runs[0] if lib_runs else None
    for r in lib_runs:
        base_x = base.exec_time / r.exec_time if base else 1.0
        prev_x = prev.exec_time / r.exec_time if prev else 1.0
        
        base_str = f"{base_x:.2f}$\\times$" if r != base else "---"
        prev_str = f"{prev_x:.2f}$\\times$" if prev else "---"
        
        date_str = r.commit_date[:10] if len(r.commit_date) >= 10 else r.commit_date
        
        latex += f"v{r.version} & {date_str} & {r.exec_time:.3f} & {r.timesteps_per_sec:.2f} & {r.cells_per_sec_m:.2f} & {base_str} & {prev_str} \\\\\n"
        prev = r
    
    latex += r"""\bottomrule
\end{tabular}
\caption{Library performance across versions}
\end{table}
"""
    
    # Summary stats for library
    if lib_runs:
        total_speedup = lib_runs[0].exec_time / lib_runs[-1].exec_time
        latex += f"""
\\textbf{{Summary:}} Final version achieves a \\textbf{{{total_speedup:.1f}$\\times$ speedup}} compared to baseline.
Time reduced from {lib_runs[0].exec_time:.3f}s to {lib_runs[-1].exec_time:.3f}s.
"""

    # UI performance table  
    latex += r"""
\section{Graphical Interface Performance (\texttt{trait\_ac\_ui})}

This section shows the performance including the graphical rendering overhead.

\begin{table}[H]
\centering
\begin{tabular}{ccrrrrr}
\toprule
\textbf{Version} & \textbf{Date} & \textbf{Time (s)} & \textbf{Steps/s} & \textbf{MCells/s} & \textbf{vs Base} & \textbf{vs Prev} \\
\midrule
"""
    
    prev = None
    base = ui_runs[0] if ui_runs else None
    for r in ui_runs:
        base_x = base.exec_time / r.exec_time if base else 1.0
        prev_x = prev.exec_time / r.exec_time if prev else 1.0
        
        base_str = f"{base_x:.2f}$\\times$" if r != base else "---"
        prev_str = f"{prev_x:.2f}$\\times$" if prev else "---"
        
        date_str = r.commit_date[:10] if len(r.commit_date) >= 10 else r.commit_date
        
        latex += f"v{r.version} & {date_str} & {r.exec_time:.3f} & {r.timesteps_per_sec:.2f} & {r.cells_per_sec_m:.2f} & {base_str} & {prev_str} \\\\\n"
        prev = r
    
    latex += r"""\bottomrule
\end{tabular}
\caption{GUI performance across versions}
\end{table}
"""
    
    # Summary stats for UI
    if ui_runs:
        total_speedup = ui_runs[0].exec_time / ui_runs[-1].exec_time
        latex += f"""
\\textbf{{Summary:}} Final version achieves a \\textbf{{{total_speedup:.1f}$\\times$ speedup}} compared to baseline.
Time reduced from {ui_runs[0].exec_time:.3f}s to {ui_runs[-1].exec_time:.3f}s.
"""

    # UI vs Library breakdown
    latex += r"""
\section{UI vs Library Time Breakdown}

This analysis separates the pure UI rendering overhead from the simulation computation time.

\begin{table}[H]
\centering
\begin{tabular}{ccrrrrrr}
\toprule
\textbf{Version} & \textbf{Date} & \textbf{UI Total (s)} & \textbf{Lib (s)} & \textbf{Pure UI (s)} & \textbf{Lib \%} & \textbf{UI \%} & \textbf{Pure UI Speedup} \\
\midrule
"""
    
    base_ui_overhead = None
    for ui_run in ui_runs:
        if ui_run.version not in lib_times:
            continue
            
        lib_time = lib_times[ui_run.version]
        ui_total = ui_run.exec_time
        pure_ui = ui_total - lib_time
        date_str = ui_run.commit_date[:10] if len(ui_run.commit_date) >= 10 else ui_run.commit_date
        
        lib_pct = (lib_time / ui_total) * 100
        ui_pct = (pure_ui / ui_total) * 100
        
        if base_ui_overhead is None:
            base_ui_overhead = pure_ui
            speedup_str = "---"
        else:
            speedup = base_ui_overhead / pure_ui if pure_ui > 0 else float('inf')
            speedup_str = f"{speedup:.2f}$\\times$"
        
        latex += f"v{ui_run.version} & {date_str} & {ui_total:.3f} & {lib_time:.3f} & {pure_ui:.3f} & {lib_pct:.1f}\\% & {ui_pct:.1f}\\% & {speedup_str} \\\\\n"
    
    latex += r"""\bottomrule
\end{tabular}
\caption{UI overhead breakdown by version}
\end{table}
"""

    # Pure UI overhead summary
    if ui_runs and lib_runs:
        first_ui_overhead = ui_runs[0].exec_time - lib_times.get(ui_runs[0].version, 0)
        last_ui_overhead = ui_runs[-1].exec_time - lib_times.get(ui_runs[-1].version, 0)
        if last_ui_overhead > 0:
            ui_speedup = first_ui_overhead / last_ui_overhead
            latex += f"""
\\textbf{{Pure UI Overhead Improvement:}} {ui_speedup:.1f}$\\times$ speedup (from {first_ui_overhead:.3f}s to {last_ui_overhead:.3f}s)
"""

    # Version history table
    latex += r"""
\section{Version History}

\begin{table}[H]
\centering
\begin{tabular}{clll}
\toprule
\textbf{Version} & \textbf{Commit} & \textbf{Date} & \textbf{Message} \\
\midrule
"""
    
    seen_versions = set()
    for r in lib_runs + ui_runs:
        if r.version in seen_versions:
            continue
        seen_versions.add(r.version)
        
        msg_escaped = r.commit_msg.replace('_', r'\_').replace('&', r'\&').replace('%', r'\%')
        if len(msg_escaped) > 50:
            msg_escaped = msg_escaped[:47] + "..."
        
        latex += f"v{r.version} & \\texttt{{{r.commit[:7]}}} & {r.commit_date[:10]} & {msg_escaped} \\\\\n"
    
    latex += r"""\bottomrule
\end{tabular}
\caption{Git commit history for each version}
\end{table}
"""

    # Overall summary
    latex += r"""
\section{Overall Summary}

\begin{center}
\begin{tabular}{lrr}
\toprule
\textbf{Component} & \textbf{Baseline Time} & \textbf{Final Speedup} \\
\midrule
"""
    
    if lib_runs:
        lib_speedup = lib_runs[0].exec_time / lib_runs[-1].exec_time
        latex += f"Library (\\texttt{{trait\\_ac}}) & {lib_runs[0].exec_time:.3f}s & {lib_speedup:.1f}$\\times$ \\\\\n"
    
    if ui_runs:
        ui_speedup = ui_runs[0].exec_time / ui_runs[-1].exec_time
        latex += f"GUI (\\texttt{{trait\\_ac\\_ui}}) & {ui_runs[0].exec_time:.3f}s & {ui_speedup:.1f}$\\times$ \\\\\n"
    
    if ui_runs and lib_runs:
        first_ui_overhead = ui_runs[0].exec_time - lib_times.get(ui_runs[0].version, 0)
        last_ui_overhead = ui_runs[-1].exec_time - lib_times.get(ui_runs[-1].version, 0)
        if last_ui_overhead > 0:
            pure_ui_speedup = first_ui_overhead / last_ui_overhead
            latex += f"Pure UI Overhead & {first_ui_overhead:.3f}s & {pure_ui_speedup:.1f}$\\times$ \\\\\n"
    
    latex += r"""\bottomrule
\end{tabular}
\end{center}

\vspace{1em}
"""

    if lib_runs:
        latex += f"""
The optimization effort resulted in a \\textbf{{{lib_runs[0].exec_time / lib_runs[-1].exec_time:.0f}$\\times$ speedup}} for the core library,
going from {lib_runs[0].cells_per_sec_m:.2f} million cells/second to {lib_runs[-1].cells_per_sec_m:.2f} million cells/second 
(\\textbf{{{lib_runs[-1].cells_per_sec_m/1000:.2f} billion cells/second}}).
"""

    latex += r"""
\end{document}
"""
    
    with open(output_path, 'w') as f:
        f.write(latex)
    
    print(f"✓ LaTeX report generated: {output_path}")


def main():
    # ============================================================
    # CONFIGURATION - Edit these values to match your test setup
    # ============================================================
    EXPECTED_GRID_WIDTH = 3000
    EXPECTED_GRID_HEIGHT = 3000
    EXPECTED_TIMESTEPS = 100
    
    # Define expected traits - each trait has a name (or None for any) and allowed rules
    # Example with 1 trait:
    EXPECTED_TRAITS = [
        TraitConfig(name=None, allowed_rules=["conway", "conway optimized"]),
    ]
    
    # Example with 2 traits:
    # EXPECTED_TRAITS = [
    #     TraitConfig(name="Energy", allowed_rules=["conway", "conway optimized"]),
    #     TraitConfig(name="Charge", allowed_rules=["conway", "conway optimized"]),
    # ]
    
    INPUT_FILE = "benchmarks.out"
    OUTPUT_FILE = None  # None = auto-generate from input file (e.g., benchmarks_report.tex)
    # ============================================================
    
    # Build expected configuration
    expected_config = BenchmarkConfig(
        grid_width=EXPECTED_GRID_WIDTH,
        grid_height=EXPECTED_GRID_HEIGHT,
        timesteps=EXPECTED_TIMESTEPS,
        traits=EXPECTED_TRAITS
    )
    
    # Read input file
    try:
        with open(INPUT_FILE, "r") as f:
            raw_data = f.read()
    except FileNotFoundError:
        print(f"❌ Error: {INPUT_FILE} not found")
        return 1
    
    # Parse benchmarks
    lib_runs, ui_runs, errors = parse_benchmarks(raw_data, expected_config)
    
    # Print test configuration
    total_cells = expected_config.grid_width * expected_config.grid_height
    print("\n" + "="*80)
    print("=== TEST CONFIGURATION ===")
    print("="*80)
    print(f"\nExpected configuration for all tests:")
    print(f"  • Grid: {expected_config.grid_width}×{expected_config.grid_height} ({total_cells:,} cells)")
    print(f"  • Timesteps: {expected_config.timesteps}")
    print(f"  • Traits: {len(expected_config.traits)}")
    for i, trait in enumerate(expected_config.traits):
        name_str = trait.name if trait.name else "(any)"
        rules_str = ", ".join(trait.allowed_rules)
        print(f"      {i}: {name_str} → allowed rules: [{rules_str}]")
    print("\nNote: Versions are from Git commits but have been modified.")
    print("      Modifications are documented in: benchmark_diffs/")
    print("      Summary available at: benchmark_diffs/SUMMARY.md")
    
    # Print validation results
    if errors:
        print("\n" + "="*80)
        print("⚠️  CONFIGURATION WARNINGS")
        print("="*80)
        for error in errors:
            print(f"  ❌ {error}")
    else:
        print("\n✓ All versions conform to expected test configuration.")
    
    # Print reports
    print_report(lib_runs, "trait_ac (library)")
    print_report(ui_runs, "trait_ac_ui (graphical interface)")
    print_ui_lib_breakdown(lib_runs, ui_runs)
    
    # Overall comparison
    if lib_runs and ui_runs:
        print("\n" + "="*120)
        print("=== OVERALL COMPARISON ===")
        print("="*120)
        
        lib_speedup = lib_runs[0].exec_time / lib_runs[-1].exec_time
        ui_speedup = ui_runs[0].exec_time / ui_runs[-1].exec_time
        
        lib_times = {r.version: r.exec_time for r in lib_runs}
        first_ui_overhead = ui_runs[0].exec_time - lib_times[ui_runs[0].version]
        last_ui_overhead = ui_runs[-1].exec_time - lib_times[ui_runs[-1].version]
        pure_ui_speedup = first_ui_overhead / last_ui_overhead if last_ui_overhead > 0 else float('inf')
        
        print(f"\ntrait_ac (library)      - Final speedup: {lib_speedup:.1f}×")
        print(f"trait_ac_ui (GUI total) - Final speedup: {ui_speedup:.1f}×")
        print(f"trait_ac_ui (UI only)   - Final speedup: {pure_ui_speedup:.1f}×")
        print(f"\nPeak throughput: {lib_runs[-1].cells_per_sec_m:.2f} million cells/sec ({lib_runs[-1].cells_per_sec_m/1000:.2f} billion cells/sec)")
        print()
    
    # Generate LaTeX
    if OUTPUT_FILE:
        latex_output = OUTPUT_FILE
    else:
        # Generate name as inputfile_summary.tex
        base_name = INPUT_FILE.rsplit('.', 1)[0] if '.' in INPUT_FILE else INPUT_FILE
        latex_output = base_name + "_report.tex"
    
    generate_latex(lib_runs, ui_runs, errors, latex_output, expected_config)
    
    return 0


if __name__ == "__main__":
    sys.exit(main())