I have a grid of cells, every cell can have an individual or be empty, every individual have traits that are values that define him, every individual have the same number of traits, there are a rule for each trait that define its update, and there is a rule for movement that define the update of the position of the individual


rule example : 

/// Energy increases when social needs are met
    /// High-social individuals gain energy near others
    /// Low-social individuals gain energy when alone
    pub fn social_energy(_trait_index: usize, cell_r: usize, cell_c: usize, neighborhood: &Neighborhood, grid: &Grid) -> f32 {
        let energy = grid.get_cell_trait(cell_r, cell_c, 0);
        let social = grid.get_cell_trait(cell_r, cell_c, 1);
        
        let center_row = neighborhood.center_row;
        let center_col = neighborhood.center_col;
        
        let mut neighbor_count = 0;
        for mask_r in 0..neighborhood.height {
            for mask_c in 0..neighborhood.width {
                if neighborhood.is_valid(mask_r, mask_c) == 1 
                && !(mask_r == center_row && mask_c == center_col) {
                    let (grid_r, grid_c) = neighborhood.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    if !grid.is_cell_empty(grid_r, grid_c) {
                        neighbor_count += 1;
                    }
                }
            }
        }
        let neighbor_total = neighborhood.height * neighborhood.width - 1;
        let density = neighbor_count as f32 / neighbor_total as f32;
        
        // Social individuals want density, loners want solitude
        let satisfaction = 1.0 - (social - density).abs();
        
        // Energy drifts toward satisfaction level
        (energy * 0.8 + satisfaction * 0.2).clamp(0.0, 1.0)
    }



movement example : 

pub fn social_movement(cell_r: usize, cell_c: usize, neighborhood: &Neighborhood, grid: &Grid) -> (isize, isize) {
        let social = grid.get_cell_trait(cell_r, cell_c, 1);
        let energy = grid.get_cell_trait(cell_r, cell_c, 0);
        
        // Low energy: don't move
        if energy < 0.2 {
            return (0, 0);
        }
        
        let center_row = neighborhood.center_row;
        let center_col = neighborhood.center_col;
        
        let mut crowd_dr: f32 = 0.0;
        let mut crowd_dc: f32 = 0.0;
        let mut has_neighbors = false;
        
        for mask_r in 0..neighborhood.height {
            for mask_c in 0..neighborhood.width {
                if neighborhood.is_valid(mask_r, mask_c) == 1 
                && !(mask_r == center_row && mask_c == center_col) {
                    let (grid_r, grid_c) = neighborhood.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    if !grid.is_cell_empty(grid_r, grid_c) {
                        let dr = mask_r as f32 - center_row as f32;
                        let dc = mask_c as f32 - center_col as f32;
                        crowd_dr += dr;
                        crowd_dc += dc;
                        has_neighbors = true;
                    }
                }
            }
        }
        
        let mut rng = rand::thread_rng();
        
        if has_neighbors {
            // Social >= 0.5: move toward crowd, < 0.5: move away
            let direction = if social >= 0.5 { 1.0 } else { -1.0 };
            let target_dr = crowd_dr * direction;
            let target_dc = crowd_dc * direction;
            
            // FIX: Handle ties and zero cases with randomness
            let dr = if target_dr.abs() < 0.001 {
                // Tie or zero: random choice
                [-1, 0, 1][rng.gen_range(0..3)]
            } else {
                target_dr.signum() as isize
            };
            
            let dc = if target_dc.abs() < 0.001 {
                [-1, 0, 1][rng.gen_range(0..3)]
            } else {
                target_dc.signum() as isize
            };
            
            // FIX: Prevent diagonal bias by sometimes restricting to cardinal
            if dr != 0 && dc != 0 {
                // Diagonal move: 50% chance to make it cardinal instead
                if rng.gen_bool(0.5) {
                    if rng.gen_bool(0.5) {
                        return (dr, 0);
                    } else {
                        return (0, dc);
                    }
                }
            }
            
            (dr, dc)
        } else {
            // No neighbors: random walk (cardinal only to avoid diagonal bias)
            let moves = [(0,1), (0,-1), (1,0), (-1,0), (0,0)];
            moves[rng.gen_range(0..moves.len())]
        }
    }





I want global complex movements and phenomenon based on simple rules, I want simple rules for each trait.
the strength of this model is combining different traits
what could lead to interesting phenomenon is not having an equilibirum too quickly, having some values always change and impact the decisions
could be an idea.



I want you to make a draft of the different traits and how they interract, don't code anything, lets just try to find the core ideas
I want very simple rules with between 2 and 4 traits and I want a complex system like the game of life by conway, use the interactions with the different traits to create something interresting, the key is to not have a fix point after a couple of iterations