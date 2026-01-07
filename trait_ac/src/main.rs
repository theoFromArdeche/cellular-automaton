use trait_ac::neighborhood::Neighborhood;
use trait_ac::grid::Grid;
use trait_ac::rules::{RulesRegistry, Rule};
use trait_ac::movement::{MovementRegistry, Movement};
use trait_ac::utils::{print_separator, semantic_traits_names, print_active_traits}; // print_trait_array
use std::time::Instant;
use rayon::prelude::*;
use serde::Deserialize;
use std::fs;

fn deserialize_rules<'de, D>(deserializer: D) -> Result<Vec<Rule>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let names: Vec<String> = Vec::deserialize(deserializer)?;
    names
        .into_iter()
        .map(|name| {
            Rule::from_name(&name).ok_or_else(|| serde::de::Error::custom(format!("Unknown rule: {}", name)))
        })
        .collect()
}

fn deserialize_movement<'de, D>(deserializer: D) -> Result<Movement, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    Movement::from_name(&name)
        .ok_or_else(|| serde::de::Error::custom(format!("Unknown movement: {}", name)))
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    // Grid settings
    pub grid_height: usize,
    pub grid_width: usize,
    pub grid_density: f32,
    pub timesteps: usize,

    // Trait settings
    pub num_traits: usize,
    pub active_mask: Vec<u8>,
    pub initialisation_ranges: Vec<(f32, f32)>,

    // Rules & movement (with custom deserializers)
    #[serde(deserialize_with = "deserialize_rules")]
    pub rules: Vec<Rule>,
    #[serde(deserialize_with = "deserialize_movement")]
    pub movement: Movement,

    // Neighborhood masks
    pub neighborhood_traits_mask: Vec<Vec<u8>>,
    pub neighborhood_mvt_mask: Vec<Vec<u8>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grid_height: 1000,
            grid_width: 1000,
            grid_density: 1.0,
            timesteps: 100,
            num_traits: 1,
            active_mask: vec![
                1, 0, 0,
                0, 0, 0,
                0, 0, 0,
            ],
            initialisation_ranges: vec![
                (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
                (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
                (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
            ],
            rules: vec![
                Rule::ConwayOptimized, Rule::ConwayOptimized, Rule::ConwayOptimized,
                Rule::ConwayOptimized, Rule::ConwayOptimized, Rule::ConwayOptimized,
                Rule::ConwayOptimized, Rule::ConwayOptimized, Rule::ConwayOptimized,
            ],
            movement: Movement::Static,
            neighborhood_traits_mask: vec![
                vec![1, 1, 1],
                vec![1, 1, 1],
                vec![1, 1, 1],
            ],
            neighborhood_mvt_mask: vec![
                vec![1, 1, 1],
                vec![1, 1, 1],
                vec![1, 1, 1],
            ],
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.grid_width == 0 || self.grid_height == 0 {
            return Err("Grid dimensions must be > 0");
        }
        if !(0.0..=1.0).contains(&self.grid_density) {
            return Err("Density must be between 0.0 and 1.0");
        }
        if self.timesteps == 0 {
            return Err("Timesteps must be > 0");
        }
        if self.num_traits == 0 {
            return Err("num_traits must be > 0");
        }
        if self.active_mask.is_empty() {
            return Err("active_mask must not be empty");
        }
        if self.initialisation_ranges.is_empty() {
            return Err("initialisation_ranges must not be empty");
        }
        if self.rules.is_empty() {
            return Err("rules must not be empty");
        }
        Ok(())
    }
}

fn main() {
    println!("=== Modular Cellular Automata Simulation ===\n");

    let config = Config::load("config.toml").unwrap_or_else(|e| {
        eprintln!("Config error: {}", e);
        std::process::exit(1);
    });

    let trait_names = semantic_traits_names();

    let rules_registry = RulesRegistry::custom(config.rules);
    let mut movement_registry = MovementRegistry::custom(
        config.grid_width,
        config.grid_height,
        config.movement,
    );

    let mut grid = Grid::new_with_density(
        config.grid_width,
        config.grid_height,
        config.grid_density,
        config.num_traits,
        &config.initialisation_ranges,
    );

    let neighborhood_traits_height = config.neighborhood_traits_mask.len();
    let neighborhood_traits_width = config.neighborhood_traits_mask[0].len();
    let neighborhood_traits_center_row = (neighborhood_traits_height - 1) / 2;
    let neighborhood_traits_center_col = (neighborhood_traits_width - 1) / 2;

    let neighborhood_mvt_height = config.neighborhood_mvt_mask.len();
    let neighborhood_mvt_width = config.neighborhood_mvt_mask[0].len();
    let neighborhood_mvt_center_row = (neighborhood_mvt_height - 1) / 2;
    let neighborhood_mvt_center_col = (neighborhood_mvt_width - 1) / 2;

    let neighborhood_traits = Neighborhood::new(
        neighborhood_traits_width,
        neighborhood_traits_height,
        neighborhood_traits_center_row,
        neighborhood_traits_center_col,
        config.neighborhood_traits_mask,
    );

    let neighborhood_mvt = Neighborhood::new(
        neighborhood_mvt_width,
        neighborhood_mvt_height,
        neighborhood_mvt_center_row,
        neighborhood_mvt_center_col,
        config.neighborhood_mvt_mask,
    );

    println!("Configuration:");
    println!("  Grid: {}x{}", grid.width, grid.height);
    println!("  Timesteps: {}", config.timesteps);
    print_active_traits(config.num_traits, &config.active_mask, &trait_names, &rules_registry);

    // Pre-allocate next grid
    let mut next_grid = Grid {
        width: grid.width,
        height: grid.height,
        num_cells: grid.num_cells,
        num_traits: grid.num_traits,
        data: grid.data.clone(),
        is_empty: grid.is_empty.clone(),
    };

    // Collect active trait indices once
    let active_traits: Vec<usize> = config.active_mask
        .iter()
        .enumerate()
        .filter_map(|(i, &m)| if m != 0 { Some(i) } else { None })
        .collect();

    //print_trait_array(&grid, 0, &trait_names);

    // Simulation loop
    let start = Instant::now();
    for _t in 1..=config.timesteps {
        let width = grid.width;
    
        // Sequential over active traits (small number), parallel over rows
        for &trait_idx in &active_traits {
            let current = grid.get_trait_slice(trait_idx);
            let next_trait = next_grid.get_trait_slice_mut(trait_idx);
            
            // Process rows in parallel
            next_trait
                .par_chunks_mut(width)
                .enumerate()
                .for_each(|(row, next_row)| {
                    let row_offset = row * width;
                    
                    for col in 0..width {
                        let idx = row_offset + col;
                        
                        next_row[col] = if grid.is_empty[idx] {
                            current[idx]
                        } else {
                            rules_registry.apply_rule(trait_idx, row, col, &neighborhood_traits, &grid)
                        };
                    }
                });
        }

        // --- STEP 2: Movement ---
        movement_registry.apply_movement(
            &neighborhood_mvt,
            &mut grid,
            &mut next_grid,
        );

        // no need to swap the grids as the updates naturally comes to "grid" after the 2 steps
    }
    //print_trait_array(&grid, 0, &trait_names);

    print_separator();
    println!("\nSimulation complete!");
    
    let elapsed = start.elapsed();
    println!("Execution time: {:?}", elapsed);
    println!(
        "Performance: {:.2} timesteps/sec",
        config.timesteps as f64 / elapsed.as_secs_f64()
    );
    println!(
        "Cells/sec: {:.2}M",
        (grid.width * grid.height * config.timesteps) as f64 / elapsed.as_secs_f64() / 1_000_000.0
    );
}