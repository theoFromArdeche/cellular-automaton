# Trait-AC: Multi-Trait Cellular Automata

A generalized approach for modeling complex multi-agent systems using cellular automata with multiple continuous traits per agent.

## Overview

Trait-AC extends classical cellular automata by allowing each agent to possess **multiple traits**, each represented by a floating-point value. These traits can evolve over time according to local rules, either independently or in a coupled manner, enabling the simulation of complex emergent behaviors.

**Key features:**
- **Multi-trait agents**: Each agent can have multiple continuous values
- **Configurable rules**: Easy-to-add update rules via Rust macros
- **Movement system**: Sophisticated 4-phase conflict resolution algorithm for agent movement
- **High performance**: Parallelized computation, cache-optimized data structures, GPU-accelerated rendering
- **Interactive UI**: Real-time visualization with scientific color palettes


## Project Structure

```
projet-automates-cellulaires/
├── trait_ac/           # Simulation library (core engine)
│   ├── src/
│   ├── config.toml     # Simulation library configuration (only if used with the main.rs of this library)
│   └── Cargo.toml
├── trait_ac_ui/        # Graphical user interface
│   ├── src/
│   ├── config.toml     # UI configuration
│   └── Cargo.toml
├── Technical Documentation.pdf
├── Benchmarks_report.pdf
└── README.md
```

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/)

### Running the Simulation with UI

```bash
cd trait_ac_ui
cargo run --release
```

### Running the Library Only (headless)

```bash
cd trait_ac
cargo run --release
```

### Configuration

Edit `config.toml` to customize:
- Grid dimensions and density
- Number and types of traits
- Update rules for each trait
- Movement behavior
- Neighborhoods for rules and movement

## Documentation

For comprehensive documentation including:
- Detailed architecture explanation
- Module descriptions (Grid, Neighborhood, Rules, Movement)
- Case study: Energy-Charge-Phase model
- Performance optimization tips
- UI features and controls

**See [Technical Documentation.pdf](Technical%20Documentation.pdf)**

⚠️ Links inside the PDF do not work in GitHub’s preview.
Please download the PDF to access clickable links.


## Performance

On a 3000×3000 grid (9 million cells) with Conway's Game of Life rules:
- **Library**: ~150 timesteps/sec (~1.3 billion cells/sec)
- **With UI**: ~110 timesteps/sec (~1 billion cells/sec)

## Disclaimer

This project began as a school assignment completed with a classmate. However, the
topic was open-ended regarding automata, and my contributions go well beyond what was
required. For this reason, this repository contains only my own work and does not
include my colleague’s contributions (since commit e1914c8), which I do not claim as my own. I consider this a
personal project rather than a school project.