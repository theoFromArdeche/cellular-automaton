use trait_ac::neighborhood::Neighborhood;
use trait_ac::grid::Grid;
use trait_ac::rules::{RuleSet, rule_diffusion, rule_average, rule_conway, rule_maximum, rule_oscillate};
use trait_ac::movement::{apply_movement, movement_static};
use trait_ac::utils::{print_active_traits, print_statistics, print_separator, semantic_trait_names};

fn main() {
    println!("=== Modular Cellular Automata Simulation ===\n");

    // Configuration
    let grid_height = 10;
    let grid_width = 10;
    let timesteps = 5;

    // Define which traits are active (3x3 boolean grid)
    let active_mask = vec![
        vec![true,  true,  false],  // Traits 0, 1 active
        vec![false, true,  false],  // Trait 4 active
        vec![true,  false, false],  // Trait 6 active
    ];

    // Define neighborhood (3x3 boolean mask)
    // True = this neighbor position affects the cell
    let neighborhood_mask = vec![
        vec![true,  true,  true],   // Von Neumann + diagonals
        vec![true,  true,  true],
        vec![true,  true,  true],
    ];

    let neighborhood_height = neighborhood_mask.len();
    let neighborhood_width = neighborhood_mask[0].len();
    let neighborhood_center_row = (neighborhood_height-1)/2;
    let neighborhood_center_col = (neighborhood_width-1)/2;

    // Initialize grid
    let mut grid = Grid::new(grid_width, grid_height);

    // Default neighborhood
    let dummy_grid = Grid::new(grid_width, grid_height); // can't use the normal grid because of the lifetime
    let neighborhood_base = Neighborhood::new(
        neighborhood_width,
        neighborhood_height,
        neighborhood_center_row,
        neighborhood_center_col,
        0, 0,
        &neighborhood_mask,
        &dummy_grid,
    );

    // Define trait names
    let trait_names = semantic_trait_names();

    // Create custom rule set
    // Traits: [Energy, Confidence, Cooperation, Aggression, Stability, Mobility, Resource, Age, Adaptability]
    let ruleset = RuleSet::custom([
        rule_diffusion,    // 0: Energy diffuses
        rule_average,      // 1: Confidence averages
        rule_conway,       // 2: Cooperation (not active)
        rule_maximum,      // 3: Aggression (not active)
        rule_oscillate,    // 4: Stability oscillates
        rule_average,      // 5: Mobility (not active)
        rule_diffusion,    // 6: Resource diffuses
        rule_average,      // 7: Age (not active)
        rule_average,      // 8: Adaptability (not active)
    ]);
 
    // Choose movement function
    let movement_fn = movement_static; // Can swap to movement_random, movement_gradient, etc.

    println!("Configuration:");
    println!("  Grid: {}x{}", grid_width, grid_height);
    println!("  Timesteps: {}", timesteps);
    println!("  Active traits: {:?}", 
        active_mask.iter()
            .enumerate()
            .flat_map(|(r, row)| row.iter().enumerate().filter(|&(_, v)| *v).map(move |(c, _)| r * 3 + c))
            .collect::<Vec<_>>()
    );

    // Initial state
    print_separator();
    println!("INITIAL STATE (t=0)");
    print_active_traits(&grid, &active_mask, &trait_names);
    print_statistics(&grid, &active_mask);

    // Simulation loop
    for t in 1..=timesteps {
        print_separator();
        println!("TIMESTEP {}", t);

        // Step 1: Update all active traits
        let mut new_cells = Vec::new();
        for row in 0..grid.height {
            let mut new_row = Vec::new();
            for col in 0..grid.width {
                let cell = &grid.cells[row][col];
                let neighborhood = Neighborhood::new_from_base(row, col, &neighborhood_base, &grid);
                let mut new_cell = cell.clone();

                // Update only active traits
                for mask_row in 0..3 {
                    for mask_col in 0..3 {
                        if active_mask[mask_row][mask_col] {
                            let trait_idx = mask_row * 3 + mask_col;
                            let new_value = ruleset.apply_rule(trait_idx, cell, &neighborhood);
                            new_cell.set_trait(trait_idx, new_value);
                        }
                    }
                }

                new_row.push(new_cell);
            }
            new_cells.push(new_row);
        }
        grid.update_cells(new_cells);

        // Step 2: Apply movement
        let moved_cells = apply_movement(&grid, &neighborhood_base, movement_fn);
        grid.update_cells(moved_cells);

        // Print results
        print_active_traits(&grid, &active_mask, &trait_names);
        print_statistics(&grid, &active_mask);
    }

    print_separator();
    println!("\nSimulation complete!");
}