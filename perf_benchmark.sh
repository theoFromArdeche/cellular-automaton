#!/usr/bin/env bash
set -e

###############################################################################
# All versions need to have the test mode implemented and enabled
# I modified some versions slightly to add a test mode
# You need to have all the versions in the same dir as this version
###############################################################################

echo "==================== BUILDING ===================="
cd ../perf-improve-0/trait_ac_ui
cargo build --release
cd ../trait_ac
cargo build --release

cd ../../perf-improve-1/trait_ac_ui
cargo build --release
cd ../trait_ac
cargo build --release

cd ../../perf-improve-2/trait_ac_ui
cargo build --release
cd ../trait_ac
cargo build --release

cd ../../perf-improve-3/trait_ac_ui
cargo build --release
cd ../trait_ac
cargo build --release

cd ../../perf-improve-4/trait_ac_ui
cargo build --release
cd ../trait_ac
cargo build --release

cd ../../perf-improve-5/trait_ac_ui
cargo build --release
cd ../trait_ac
cargo build --release

cd ../../perf-improve-6/trait_ac_ui
cargo build --release
cd ../trait_ac
cargo build --release

echo "==================== FINISHED BUILDING ===================="

###############################################################################
# BENCHMARKS (captured into benchmarks.out with live feed via tee)
###############################################################################

# Function to run benchmark with progress indicator
run_benchmark() {
    local version=$1
    local project=$2
    local desc=$3
    
    echo ""
    echo "========== ${desc} =========="
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Running ${project} version ${version}..."
    
    cd "../../perf-improve-${version}/${project}"
    cargo run --release
    
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Completed ${project} version ${version}"
    echo "Waiting 1 minute before next benchmark..."
    sleep 1m
}

{
    echo "==================== STARTING BENCHMARKS ===================="
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Benchmark suite started"
    echo ""
    
    echo "==================== trait_ac (library) ===================="
    
    run_benchmark 0 "trait_ac" "Perf improve 0 (base)"
    run_benchmark 1 "trait_ac" "Perf improve 1"
    run_benchmark 2 "trait_ac" "Perf improve 2"
    run_benchmark 3 "trait_ac" "Perf improve 3"
    run_benchmark 4 "trait_ac" "Perf improve 4"
    run_benchmark 5 "trait_ac" "Perf improve 5"
    run_benchmark 6 "trait_ac" "Perf improve 6"
    
    echo ""
    echo "==================== trait_ac_ui (graphical interface) ===================="
    
    run_benchmark 0 "trait_ac_ui" "Perf improve 0 (base)"
    run_benchmark 1 "trait_ac_ui" "Perf improve 1"
    run_benchmark 2 "trait_ac_ui" "Perf improve 2"
    run_benchmark 3 "trait_ac_ui" "Perf improve 3"
    run_benchmark 4 "trait_ac_ui" "Perf improve 4"
    run_benchmark 5 "trait_ac_ui" "Perf improve 5"
    run_benchmark 6 "trait_ac_ui" "Perf improve 6"
    
    echo ""
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Benchmark suite completed"
    cd ../../projet-automates-cellulaires
    echo "==================== FINISHED BENCHMARKS ===================="
} 2>&1 | tee ../../projet-automates-cellulaires/benchmarks.out