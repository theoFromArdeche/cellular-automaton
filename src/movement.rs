use crate::cell::Cell;
use crate::grid::Grid;
use crate::neighborhood::Neighborhood;
use rand::Rng;

/// No movement - cells stay in place
pub fn movement_static(_cell: &Cell, _neighborhood: &Neighborhood, _grid: &Grid) -> (isize, isize) {
    (0, 0)
}

/// Random walk - move randomly within Moore neighborhood
pub fn movement_random(_cell: &Cell, _neighborhood: &Neighborhood, _grid: &Grid) -> (isize, isize) {
    let mut rng = rand::thread_rng();
    (rng.gen_range(-1..=1), rng.gen_range(-1..=1))
}

/// Move toward the neighbor with the highest trait value
pub fn movement_gradient(cell: &Cell, neighborhood: &Neighborhood, _grid: &Grid) -> (isize, isize) {
    let current_val = cell.get_trait(0);

    let mut best_val = current_val;
    let mut best_move = (0, 0);

    for (dr, row) in neighborhood.cells.iter().enumerate() {
        for (dc, neighbor) in row.iter().enumerate() {
            if !neighborhood.mask[dr][dc] {
                continue;
            }

            // Skip center cell
            if dr == neighborhood.center_row && dc == neighborhood.center_col {
                continue;
            }

            let val = neighbor.get_trait(0);
            if val > best_val {
                best_val = val;
                best_move = (
                    dr as isize - neighborhood.center_row as isize,
                    dc as isize - neighborhood.center_col as isize,
                );
            }
        }
    }

    best_move
}

/// Move away from crowded areas (high average neighbor density)
pub fn movement_avoid_crowding(_cell: &Cell, neighborhood: &Neighborhood, _grid: &Grid) -> (isize, isize) {
    let mut sum = 0.0;
    let mut count = 0;

    for (dr, row) in neighborhood.cells.iter().enumerate() {
        for (dc, neighbor) in row.iter().enumerate() {
            if neighborhood.mask[dr][dc] &&
               !(dr == neighborhood.center_row && dc == neighborhood.center_col)
            {
                sum += neighbor.get_trait(0);
                count += 1;
            }
        }
    }

    if count == 0 {
        return (0, 0);
    }

    let avg_density = sum / count as f32;

    if avg_density > 0.7 {
        let mut rng = rand::thread_rng();
        (rng.gen_range(-1..=1), rng.gen_range(-1..=1))
    } else {
        (0, 0)
    }
}

/// Trait-based exploratory movement
pub fn movement_trait_based(cell: &Cell, neighborhood: &Neighborhood, _grid: &Grid) -> (isize, isize) {
    let trait0 = cell.get_trait(0);
    let trait1 = cell.get_trait(1);

    let mut sum = 0.0;
    let mut count = 0;

    for (dr, row) in neighborhood.cells.iter().enumerate() {
        for (dc, neighbor) in row.iter().enumerate() {
            if neighborhood.mask[dr][dc] &&
               !(dr == neighborhood.center_row && dc == neighborhood.center_col)
            {
                sum += neighbor.get_trait(0);
                count += 1;
            }
        }
    }

    let avg_neighbor_trait0 = if count == 0 {
        0.0
    } else {
        sum / count as f32
    };

    // Explore if isolated
    if trait0 > 0.7 && avg_neighbor_trait0 < 0.3 {
        let mut rng = rand::thread_rng();
        return (rng.gen_range(-1..=1), rng.gen_range(-1..=1));
    }

    // Stay if stable
    if trait1 > 0.8 {
        return (0, 0);
    }

    // Small random jitter
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.3) {
        (rng.gen_range(-1..=1), rng.gen_range(-1..=1))
    } else {
        (0, 0)
    }
}

/// Apply movement to all cells in the grid
pub fn apply_movement(grid: &Grid,
                      neighborhood_base: &Neighborhood,
                      movement_fn: fn(&Cell, &Neighborhood, &Grid) -> (isize, isize),
                     ) -> Vec<Vec<Cell>> {

    let mut new_cells = vec![vec![Cell::new((0, 0)); grid.width]; grid.height];
    let mut occupied = vec![vec![false; grid.width]; grid.height];
    let mut movements = Vec::new();

    // First pass: compute desired movements
    for row in 0..grid.height {
        for col in 0..grid.width {
            let cell = &grid.cells[row][col];
            let neighborhood = Neighborhood::new_from_base(row, col, neighborhood_base, grid);

            let (dr, dc) = movement_fn(cell, &neighborhood, grid);

            let new_row = ((row as isize + dr).rem_euclid(grid.height as isize)) as usize;
            let new_col = ((col as isize + dc).rem_euclid(grid.width as isize)) as usize;

            movements.push((cell.clone(), new_row, new_col));
        }
    }

    // Second pass: resolve collisions
    for (mut cell, r, c) in movements {
        if !occupied[r][c] {
            cell.position = (r, c);
            new_cells[r][c] = cell;
            occupied[r][c] = true;
        } else {
            let (or, oc) = cell.position;
            if !occupied[or][oc] {
                new_cells[or][oc] = cell;
                occupied[or][oc] = true;
            }
        }
    }

    new_cells
}
