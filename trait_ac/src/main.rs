use trait_ac::neighborhood::Neighborhood;
use trait_ac::grid::Grid;
use trait_ac::rules::{RulesRegistry, Rules, RuleFn};
use trait_ac::movement::{MovementRegistry, Movements, MovementFn};
use trait_ac::utils::{print_separator, semantic_traits_names, print_active_traits};
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

    // Neighborhood mask
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

    // Initialize grid
    let mut grid = Grid::new_with_density(grid_width, grid_height, grid_density);

    // Default neighborhood
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

    // Define trait names
    let trait_names = semantic_traits_names();

    // Create custom rule set
    let rules: [RuleFn; 9] = [
        Rules::conway_optimized, Rules::conway_optimized, Rules::conway_optimized,
        Rules::conway_optimized, Rules::conway_optimized, Rules::conway_optimized,
        Rules::conway_optimized, Rules::conway_optimized, Rules::conway_optimized,
    ];
    let rules_registry = RulesRegistry::custom(rules);

    let movement_function: MovementFn = Movements::static_movement;
    let mut movement_registry = MovementRegistry::custom(grid_width, grid_height, movement_function);

    println!("Configuration:");
    println!("  Grid: {}x{}", grid_width, grid_height);
    println!("  Timesteps: {}", timesteps);
    
    // Print active traits for info
    let active_traits: Vec<usize> = active_mask.iter()
        .enumerate()
        .filter_map(|(i, &active)| if active == 1 { Some(i) } else { None })
        .collect();
    print_active_traits(&active_traits, &trait_names, &rules_registry);

    // Pre-allocate next grid
    let mut next_grid = Grid {
        width: grid.width,
        height: grid.height,
        traits: grid.traits.clone(),
        is_empty: grid.is_empty.clone(),
    };

    // OPTIMIZATION: Tune chunk size for cache efficiency
    let rows_per_batch = std::cmp::max(1, 4000 / grid_width);

    // Simulation loop
    let start = Instant::now();
    for _t in 1..=timesteps {
        let width = grid.width;

        // --- OPTIMIZED TRAIT UPDATE ---
        // Process each trait vector in parallel
        next_grid.traits
            .par_iter_mut()
            .zip(grid.traits.par_iter())
            .enumerate()
            .for_each(|(trait_idx, (next_trait_vec, current_trait_vec))| {
                // Skip inactive traits entirely
                if active_mask[trait_idx] == 0 {
                    next_trait_vec.copy_from_slice(current_trait_vec);
                    return;
                }

                // Process this trait for all cells
                next_trait_vec
                    .par_chunks_mut(rows_per_batch * width)
                    .enumerate()
                    .for_each(|(chunk_idx, next_trait_chunk)| {
                        let start_idx = chunk_idx * rows_per_batch * width;
                        
                        for i in 0..next_trait_chunk.len() {
                            let cell_idx = start_idx + i;
                            
                            // FAST PATH: Skip empty cells
                            if grid.is_empty[cell_idx] != 0 {
                                next_trait_chunk[i] = current_trait_vec[cell_idx];
                                continue;
                            }
                            
                            // Calculate position
                            let row = cell_idx / width;
                            let col = cell_idx % width;
                            
                            // Apply rule only for this trait
                            next_trait_chunk[i] = rules_registry.apply_rule(
                                trait_idx, 
                                row, 
                                col, 
                                &neighborhood_traits, 
                                &grid
                            );
                        }
                    });
            });

        // Update is_empty separately (only once, not per trait)
        next_grid.is_empty
            .par_chunks_mut(rows_per_batch * width)
            .enumerate()
            .for_each(|(chunk_idx, next_empty_chunk)| {
                let start_idx = chunk_idx * rows_per_batch * width;
                for i in 0..next_empty_chunk.len() {
                    let cell_idx = start_idx + i;
                    next_empty_chunk[i] = grid.is_empty[cell_idx];
                }
            });

        // --- STEP 2: Movement ---
        movement_registry.apply_movement(
            &neighborhood_mvt,
            &mut grid,
            &mut next_grid,
        );

        // no need to swap the grids as the updates naturally comes to "grid" after the 2 steps
    }

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