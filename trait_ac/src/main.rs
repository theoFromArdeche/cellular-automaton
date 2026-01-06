use trait_ac::neighborhood::Neighborhood;
use trait_ac::grid::Grid;
use trait_ac::rules::{RulesRegistry, Rules, RuleFn};
use trait_ac::movement::{MovementRegistry, Movements, MovementFn};
use trait_ac::utils::{print_separator, semantic_traits_names, print_active_traits}; // print_trait_array
use std::time::Instant;
use rayon::prelude::*;

fn main() {
    println!("=== Modular Cellular Automata Simulation ===\n");

    // Configuration
    let grid_height = 3000;
    let grid_width = 3000;
    let grid_density = 1.0;
    let timesteps = 100;
    
    let active_mask: [u8; 9] = [
        1, 0, 0,
        0, 0, 0,
        0, 0, 0,
    ];

    // range at initialisation for each traits
    let initialisation_ranges = [ 
        (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
        (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
        (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
    ];

    let trait_names = semantic_traits_names();

    // Custum rules for each traits
    let rules: [RuleFn; 9] = [
        Rules::conway_optimized, Rules::conway_optimized, Rules::conway_optimized,
        Rules::conway_optimized, Rules::conway_optimized, Rules::conway_optimized,
        Rules::conway_optimized, Rules::conway_optimized, Rules::conway_optimized,
    ];
    let rules_registry = RulesRegistry::custom(rules);

    let movement_function: MovementFn = Movements::static_movement;
    let mut movement_registry = MovementRegistry::custom(grid_width, grid_height, movement_function);

    // Initialize grid
    let mut grid = Grid::new_with_density(grid_width, grid_height, grid_density, initialisation_ranges);

    let neighborhood_traits_mask = vec![
        vec![1, 1, 1],
        vec![1, 1, 1],
        vec![1, 1, 1],
    ];

    let neighborhood_mvt_mask = vec![
        vec![1, 1, 1],
        vec![1, 1, 1],
        vec![1, 1, 1],
    ];

    let neighborhood_traits_height = neighborhood_traits_mask.len();
    let neighborhood_traits_width = neighborhood_traits_mask[0].len();
    let neighborhood_traits_center_row = (neighborhood_traits_height - 1) / 2;
    let neighborhood_traits_center_col = (neighborhood_traits_width - 1) / 2;

    let neighborhood_mvt_height = neighborhood_mvt_mask.len();
    let neighborhood_mvt_width = neighborhood_mvt_mask[0].len();
    let neighborhood_mvt_center_row = (neighborhood_mvt_height - 1) / 2;
    let neighborhood_mvt_center_col = (neighborhood_mvt_width - 1) / 2;

    let neighborhood_traits = Neighborhood::new(
        neighborhood_traits_width,
        neighborhood_traits_height,
        neighborhood_traits_center_row,
        neighborhood_traits_center_col,
        neighborhood_traits_mask,
    );

    let neighborhood_mvt = Neighborhood::new(
        neighborhood_mvt_width,
        neighborhood_mvt_height,
        neighborhood_mvt_center_row,
        neighborhood_mvt_center_col,
        neighborhood_mvt_mask,
    );

    println!("Configuration:");
    println!("  Grid: {}x{}", grid_width, grid_height);
    println!("  Timesteps: {}", timesteps);
    print_active_traits(&active_mask, &trait_names, &rules_registry);

    // Pre-allocate next grid
    let mut next_grid = Grid {
        width: grid.width,
        height: grid.height,
        traits: grid.traits.clone(),
        is_empty: grid.is_empty.clone(),
    };

    // Collect active trait indices once
    let active_traits: Vec<usize> = active_mask
        .iter()
        .enumerate()
        .filter_map(|(i, &m)| if m != 0 { Some(i) } else { None })
        .collect();

    //print_trait_array(&grid, 0, &trait_names);

    // Simulation loop
    let start = Instant::now();
    for _t in 1..=timesteps {
        let width = grid.width;
        
        // Process each trait in parallel
        next_grid.traits
            .par_iter_mut()
            .enumerate()
            .filter(|(idx, _)| active_traits.contains(idx))
            .for_each(|(trait_idx, next_trait)| {
                let current = &grid.traits[trait_idx];
                
                // Process rows in parallel, use tiling within each row for cache locality
                next_trait
                    .par_chunks_mut(width)
                    .enumerate()
                    .for_each(|(row, next_row)| {
                        let row_offset = row * width;
                        
                        // Process in cache-friendly chunks of 64
                        for chunk_start in (0..width).step_by(64) {
                            let chunk_end = (chunk_start + 64).min(width);
                            
                            for col in chunk_start..chunk_end {
                                let idx = row_offset + col;
                                next_row[col] = if grid.is_empty[idx] {
                                    current[idx]
                                } else {
                                    rules_registry.apply_rule(trait_idx, row, col, &neighborhood_traits, &grid)
                                };
                            }
                        }
                    });
            });

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
        timesteps as f64 / elapsed.as_secs_f64()
    );
    println!(
        "Cells/sec: {:.2}M",
        (grid_width * grid_height * timesteps) as f64 / elapsed.as_secs_f64() / 1_000_000.0
    );
}