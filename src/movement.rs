use crate::cell::Cell;
use crate::grid::Grid;
use rand::Rng;

/// Type alias for movement functions
pub type MovementFn = fn(&Cell, &[Cell], &Grid) -> (isize, isize);

/// No movement - cells stay in place
pub fn movement_static(_cell: &Cell, _neighbors: &[Cell], _grid: &Grid) -> (isize, isize) {
    (0, 0)
}

/// Random walk - move to a random neighbor position
pub fn movement_random(_cell: &Cell, _neighbors: &[Cell], _grid: &Grid) -> (isize, isize) {
    let mut rng = rand::thread_rng();
    let dx = rng.gen_range(-1..=1);
    let dy = rng.gen_range(-1..=1);
    (dx, dy)
}

/// Move towards higher trait values in the neighborhood
pub fn movement_gradient(cell: &Cell, neighbors: &[Cell], _grid: &Grid) -> (isize, isize) {
    if neighbors.is_empty() {
        return (0, 0);
    }

    // Use first trait for gradient following
    let current_val = cell.get_trait(0);
    
    // Find neighbor with highest first trait value
    let mut max_val = current_val;
    let mut best_direction = (0, 0);
    
    // Check all 8 directions
    let directions = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),           (0, 1),
        (1, -1),  (1, 0),  (1, 1),
    ];
    
    for (idx, neighbor) in neighbors.iter().enumerate() {
        let val = neighbor.get_trait(0);
        if val > max_val && idx < directions.len() {
            max_val = val;
            best_direction = directions[idx];
        }
    }
    
    best_direction
}

/// Move away from crowded areas
pub fn movement_avoid_crowding(cell: &Cell, neighbors: &[Cell], _grid: &Grid) -> (isize, isize) {
    if neighbors.is_empty() {
        return (0, 0);
    }

    // Calculate average neighbor density (using first trait as "presence")
    let avg_density: f32 = neighbors.iter().map(|n| n.get_trait(0)).sum::<f32>() 
        / neighbors.len() as f32;
    
    // If too crowded, try to move away
    if avg_density > 0.7 {
        let mut rng = rand::thread_rng();
        // Move in a random direction away
        (rng.gen_range(-1..=1), rng.gen_range(-1..=1))
    } else {
        (0, 0)
    }
}

/// Move based on trait differential with neighbors
pub fn movement_trait_based(cell: &Cell, neighbors: &[Cell], _grid: &Grid) -> (isize, isize) {
    if neighbors.is_empty() {
        return (0, 0);
    }

    // Use multiple traits to decide movement
    let trait0 = cell.get_trait(0);
    let trait1 = cell.get_trait(1);
    
    let avg_neighbor_trait0: f32 = neighbors.iter().map(|n| n.get_trait(0)).sum::<f32>() 
        / neighbors.len() as f32;
    
    // If trait0 is high and neighbors are low, move randomly (explore)
    if trait0 > 0.7 && avg_neighbor_trait0 < 0.3 {
        let mut rng = rand::thread_rng();
        return (rng.gen_range(-1..=1), rng.gen_range(-1..=1));
    }
    
    // If trait1 is high, stay put (stability)
    if trait1 > 0.8 {
        return (0, 0);
    }
    
    // Default: small random movement
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.3) {
        (rng.gen_range(-1..=1), rng.gen_range(-1..=1))
    } else {
        (0, 0)
    }
}

/// Apply movement to all cells in the grid
pub fn apply_movement(grid: &Grid, movement_fn: MovementFn) -> Vec<Vec<Cell>> {
    let mut new_cells = vec![vec![Cell::new((0, 0)); grid.width]; grid.height];
    let mut occupied = vec![vec![false; grid.width]; grid.height];
    
    // First pass: calculate desired movements
    let mut movements = Vec::new();
    for row in 0..grid.height {
        for col in 0..grid.width {
            let cell = &grid.cells[row][col];
            let neighbors = grid.get_neighbors(row, col);
            let (dx, dy) = movement_fn(cell, &neighbors, grid);
            
            let new_row = ((row as isize + dx).rem_euclid(grid.height as isize)) as usize;
            let new_col = ((col as isize + dy).rem_euclid(grid.width as isize)) as usize;
            
            movements.push((cell.clone(), new_row, new_col));
        }
    }
    
    // Second pass: resolve conflicts (first-come-first-served)
    for (mut cell, new_row, new_col) in movements {
        if !occupied[new_row][new_col] {
            cell.position = (new_row, new_col);
            new_cells[new_row][new_col] = cell;
            occupied[new_row][new_col] = true;
        } else {
            // Position occupied, stay in original position
            let (orig_row, orig_col) = cell.position;
            if !occupied[orig_row][orig_col] {
                new_cells[orig_row][orig_col] = cell;
                occupied[orig_row][orig_col] = true;
            }
        }
    }
    
    new_cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_movement() {
        let cell = Cell::new((5, 5));
        let neighbors = vec![];
        let mask = vec![vec![true; 3]; 3];
        let grid = Grid::new(10, 10, mask);
        
        let (dx, dy) = movement_static(&cell, &neighbors, &grid);
        assert_eq!((dx, dy), (0, 0));
    }
}
