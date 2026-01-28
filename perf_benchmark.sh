#!/usr/bin/env bash
set -euo pipefail
###############################################################################
# CONFIGURATION
###############################################################################
ROOT_DIR="../benchmarks-projet-automates-cellulaire"
PROJECTS=("trait_ac" "trait_ac_ui")
MAX_VERSION=7
SLEEP_BETWEEN_RUNS="1m"
LOG_FILE="benchmarks.out"
DIFF_DIR="benchmark_diffs"
###############################################################################
# HELPERS
###############################################################################
timestamp() {
    date '+%Y-%m-%d %H:%M:%S'
}

print_commit_info() {
    local version=$1
    echo ""
    echo "==================== CODE VERSION ${version} ===================="
    cd "perf-improve-${version}"
    git log -1 --pretty=format:"Commit: %h%nDate:   %ad%nMsg:    %s%n" --date=iso
    cd - >/dev/null
}

build_project() {
    local version=$1
    local project=$2
    echo "[$(timestamp)] BUILD perf-improve-${version}/${project}"
    cd "perf-improve-${version}/${project}"
    cargo build --release
    cd - >/dev/null
}

run_benchmark() {
    local version=$1
    local project=$2
    echo ""
    echo "========== BENCHMARK ${project} | VERSION ${version} =========="
    echo "[$(timestamp)] Starting"
    cd "perf-improve-${version}/${project}"
    cargo run --release
    cd - >/dev/null
    echo "[$(timestamp)] Finished"
    echo "Sleeping ${SLEEP_BETWEEN_RUNS}..."
    sleep "${SLEEP_BETWEEN_RUNS}"
}

generate_diffs() {
    local diff_base="${SCRIPT_DIR}/${DIFF_DIR}"
    
    echo ""
    echo "==================== GENERATING DIFFS ===================="
    echo "[$(timestamp)] Creating diff directory structure"
    
    # Check if diff directory already exists
    if [[ -d "${diff_base}" ]]; then
        echo "ERROR: ${diff_base} already exists."
        echo "Please remove or rename it manually before running this script."
        exit 1
    fi
    
    mkdir -p "${diff_base}"
    
    for v in $(seq 0 "${MAX_VERSION}"); do
        local version_path="perf-improve-${v}"
        local diff_file="${diff_base}/version_${v}.diff"
        local info_file="${diff_base}/version_${v}_info.txt"
        
        echo ""
        echo "--- Version ${v} ---"
        
        if [[ -d "${version_path}" ]]; then
            echo "[$(timestamp)] Generating diff for ${version_path}"
            
            cd "${version_path}"
            
            # Write commit info to info file
            {
                echo "Version: ${v}"
                echo "Path: ${version_path}"
                echo "Generated: $(timestamp)"
                echo ""
                echo "==================== CURRENT COMMIT (HEAD) ===================="
                git log -1 --pretty=format:"Commit: %H%nShort:  %h%nAuthor: %an <%ae>%nDate:   %ad%nMsg:    %s%n" --date=iso
                echo ""
                echo "==================== UNSTAGED CHANGES ===================="
                git diff --stat 2>/dev/null || echo "(No unstaged changes)"
            } > "${info_file}"
            
            # Generate the actual diff (unstaged changes vs HEAD)
            {
                echo "# Git diff for Version ${v}"
                echo "# Unstaged changes compared to HEAD"
                echo "# Generated: $(timestamp)"
                echo "# HEAD Commit: $(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
                echo ""
                git diff 2>/dev/null || echo "# No unstaged changes"
            } > "${diff_file}"
            
            cd "${ROOT_DIR}"
            
            echo "  -> Created: ${diff_file}"
            echo "  -> Created: ${info_file}"
        else
            echo "[$(timestamp)] WARNING: ${version_path} not found, skipping"
        fi
    done
    
    # Create a summary file
    local summary_file="${diff_base}/SUMMARY.md"
    {
        echo "# Benchmark Diffs Summary"
        echo ""
        echo "Generated: $(timestamp)"
        echo ""
        
        for v in $(seq 0 "${MAX_VERSION}"); do
            echo "## Version ${v}"
            echo ""
            
            # Embed info
            echo "### Commit Info"
            echo ""
            echo "<details>"
            echo "<summary>Click to expand version ${v} info</summary>"
            echo ""
            echo '```'
            cat "${diff_base}/version_${v}_info.txt" 2>/dev/null || echo "(Info file not found)"
            echo '```'
            echo ""
            echo "</details>"
            echo ""
            
            # Embed diff
            echo "### Code Changes"
            echo ""
            echo "<details>"
            echo "<summary>Click to expand version ${v} diff</summary>"
            echo ""
            echo '```diff'
            cat "${diff_base}/version_${v}.diff" 2>/dev/null || echo "(Diff file not found)"
            echo '```'
            echo ""
            echo "</details>"
            echo ""
            echo "---"
            echo ""
        done
    } > "${summary_file}"
    
    echo ""
    echo "[$(timestamp)] Diffs generated in: ${diff_base}"
    echo "[$(timestamp)] Summary file: ${summary_file}"
    echo "==================== DIFFS COMPLETE ===================="
}

###############################################################################
# MAIN
###############################################################################
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_PATH="${SCRIPT_DIR}/${LOG_FILE}"

# Convert ROOT_DIR to absolute path
cd "${ROOT_DIR}"
ROOT_DIR="$(pwd)"

{
    echo "==================== DIFF GENERATION PHASE ===================="
    generate_diffs
    
    echo ""
    echo "==================== BUILD PHASE ===================="
    echo "[$(timestamp)] Build started"
    echo ""
    
    for v in $(seq 0 "${MAX_VERSION}"); do
        [[ -d "perf-improve-${v}" ]] || {
            echo "ERROR: perf-improve-${v} not found"
            exit 1
        }
        for project in "${PROJECTS[@]}"; do
            build_project "${v}" "${project}"
        done
    done
    
    echo ""
    echo "==================== FINISHED BUILD ===================="
    echo ""
    echo "==================== BENCHMARK PHASE ===================="
    echo "[$(timestamp)] Benchmarks started"
    echo ""
    
    for v in $(seq 0 "${MAX_VERSION}"); do
        print_commit_info "${v}"
        for project in "${PROJECTS[@]}"; do
            echo ""
            run_benchmark "${v}" "${project}"
        done
    done
    
    echo ""
    echo "[$(timestamp)] Benchmark suite completed"
    echo "==================== FINISHED BENCHMARKS ===================="
} 2>&1 | tee "${LOG_PATH}"