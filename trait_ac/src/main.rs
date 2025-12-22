use trait_ac::neighborhood::Neighborhood;
use trait_ac::grid::Grid;
use trait_ac::rules::{RuleSet, rule_conway};
use trait_ac::movement::{apply_movement, movement_static};
use trait_ac::utils::{print_separator, semantic_trait_names};
use std::time::Instant;
use rayon::prelude::*;

fn main() {
    let start = Instant::now();
    println!("=== Modular Cellular Automata Simulation ===\n");

    // Configuration
    let grid_height = 250;
    let grid_width = 250;
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
    let mut grid = Grid::new_with_density(grid_width, grid_height, 1.0);


    // Default neighborhood
    let dummy_grid = Grid::new(grid_width, grid_height);
    let neighborhood_traits_base = Neighborhood::new(
        neighborhood_traits_width,
        neighborhood_traits_height,
        neighborhood_traits_center_row,
        neighborhood_traits_center_col,
        0, 0,
        &neighborhood_traits_mask,
        &dummy_grid,
    );

    let neighborhood_mvt_base = Neighborhood::new(
        neighborhood_mvt_width,
        neighborhood_mvt_height,
        neighborhood_mvt_center_row,
        neighborhood_mvt_center_col,
        0, 0,
        &neighborhood_mvt_mask,
        &dummy_grid,
    );

    // Define trait names
    let trait_names = semantic_trait_names();

    // Create custom rule set
    let ruleset = RuleSet::custom([
        rule_conway, rule_conway, rule_conway,
        rule_conway, rule_conway, rule_conway,
        rule_conway, rule_conway, rule_conway,
    ]);

    // Choose movement function
    let movement_fn = movement_static;

    println!("Configuration:");
    println!("  Grid: {}x{}", grid_width, grid_height);
    println!("  Timesteps: {}", timesteps);
    println!("  Active traits: {:?}", active_traits);

    // Simulation loop with optimizations
    for _t in 1..=timesteps {
        // Step 1: Update all active traits using parallel processing
        let mut new_cells: Vec<Vec<_>> = (0..grid.height)
            .into_par_iter()
            .map(|row| {
                let mut new_row = Vec::with_capacity(grid.width);
                for col in 0..grid.width {
                    let cell = &grid.cells[row][col];
                    
                    if cell.is_empty() {
                        new_row.push(cell.clone());
                        continue;
                    }
                    
                    let mut new_cell = cell.clone();
                    let neighborhood_traits = Neighborhood::new_from_base(row, col, &neighborhood_traits_base, &grid);

                    // Update only active traits
                    for &trait_idx in &active_traits {
                        let new_value = ruleset.apply_rule(cell, &neighborhood_traits, trait_idx);
                        new_cell.set_trait(trait_idx, new_value);
                    }

                    new_row.push(new_cell);
                }
                new_row
            })
            .collect();

        grid.update_cells_fast(&mut new_cells);

        // Step 2: Apply movement
        let mut moved_cells = apply_movement(movement_fn, &neighborhood_mvt_base, &grid);
        grid.update_cells_fast(&mut moved_cells);
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