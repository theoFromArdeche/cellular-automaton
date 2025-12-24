use trait_ac::neighborhood::Neighborhood;
use trait_ac::grid::Grid;
use trait_ac::rules::{RulesRegistry, Rules, RuleFn};
use trait_ac::movement::{MovementRegistry, Movements, MovementFn};
use trait_ac::utils::{print_separator, semantic_traits_names, print_active_traits};
use std::time::Instant;
use rayon::prelude::*;



fn main() {
    let start = Instant::now();
    println!("=== Modular Cellular Automata Simulation ===\n");

    // Configuration
    let grid_height = 1500;
    let grid_width = 1500;
    let grid_density = 1.0;
    let timesteps = 100;

    let active_traits: Vec<usize> = vec![0, 1, 2, 3, 4];

    // Neighborhood mask
    let neighborhood_traits_mask = vec![
        vec![true, true, true],
        vec![true, true, true],
        vec![true, true, true],
    ];

    let neighborhood_mvt_mask = vec![
        vec![true, true, true],
        vec![true, true, true],
        vec![true, true, true],
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
            Rules::conway, Rules::conway, Rules::conway,
            Rules::conway, Rules::conway, Rules::conway,
            Rules::conway, Rules::conway, Rules::conway,
    ];
    let rules_registry = RulesRegistry::custom(rules);

    let movement_function: MovementFn = Movements::static_movement; 
    let mut movement_registry = MovementRegistry::custom(grid_width, grid_height, movement_function);

    println!("Configuration:");
    println!("  Grid: {}x{}", grid_width, grid_height);
    println!("  Timesteps: {}", timesteps);
    print_active_traits(&active_traits, &trait_names, &rules_registry);

    // 0. PRE-ALLOCATION (Do this before the loop)
    // Create a secondary buffer that mirrors the grid structure perfectly.
    // We clone once to get the memory layout, then we reuse it forever.
    let mut next_grid_cells = grid.cells.clone();
    let rows_per_batch = std::cmp::max(1, 4000 / grid_width);

    // Simulation loop
    for _t in 1..=timesteps {
        
        // --- STEP 1: Update Traits (Double Buffering) ---
        
        // We iterate mutably over the 'buffer' (destination) and immutably over the 'grid' (source).
        // usage of 'par_iter_mut' paired with 'zip' allows 1:1 mapping without allocation.
        next_grid_cells
            .par_chunks_mut(rows_per_batch)
            .zip(grid.cells.par_chunks(rows_per_batch))
            .for_each(|(next_rows, current_rows)| {
                for (next_row, current_row) in next_rows.iter_mut().zip(current_rows.iter()) {
                    for (next_cell, current_cell) in next_row.iter_mut().zip(current_row.iter()) {
                        // Fast path for empty cells
                        if current_cell.is_empty() {
                            *next_cell = current_cell.clone();
                            continue;
                        }

                        // Copy base state
                        *next_cell = current_cell.clone();

                        // Apply active traits
                        for &trait_idx in &active_traits {
                            let new_value = rules_registry.apply_rule(
                                trait_idx,
                                current_cell,
                                &neighborhood_traits,
                                &grid,
                            );
                            next_cell.set_trait(trait_idx, new_value);
                        }
                    }
                }
            });


        // Swap the buffers. The 'grid' now contains the calculated state.
        // This is extremely fast (pointer swap).
        grid.update_cells_fast(&mut next_grid_cells);


        // Step 2: Movement (Write results into next_grid_cells)
        movement_registry.apply_movement(
            &neighborhood_mvt, 
            &grid, 
            &mut next_grid_cells, 
        );
        
        // Swap grids again so 'grid' has the final state for this tick
        grid.update_cells_fast(&mut next_grid_cells);
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