use crate::cell::Cell;
use crate::grid::Grid;
use crate::neighborhood::Neighborhood;
use rand::Rng;

/// No movement - cells stay in place
pub fn movement_static(_cell: &Cell, _nbhr_movement: &Neighborhood) -> (isize, isize) {
    (0, 0)
}

/// Random walk - move randomly to any valid position in the neighborhood mask
pub fn movement_random(_cell: &Cell, nbhr_movement: &Neighborhood) -> (isize, isize) {
    // Collect all valid movement positions from the mask
    let mut valid_moves = Vec::new();
    
    for (dr, row) in nbhr_movement.mask.iter().enumerate() {
        for (dc, &is_valid) in row.iter().enumerate() {
            if is_valid {
                let move_dr = dr as isize - nbhr_movement.center_row as isize;
                let move_dc = dc as isize - nbhr_movement.center_col as isize;
                valid_moves.push((move_dr, move_dc));
            }
        }
    }
    
    if valid_moves.is_empty() {
        return (0, 0);
    }
    
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..valid_moves.len());
    valid_moves[idx]
}

/// Move toward the neighbor with the highest trait value (only to valid mask positions)
pub fn movement_gradient(cell: &Cell, nbhr_movement: &Neighborhood) -> (isize, isize) {
    let current_val = cell.get_trait(0);

    let mut best_val = current_val;
    let mut best_move = (0, 0);

    for (dr, row) in nbhr_movement.cells.iter().enumerate() {
        for (dc, neighbor) in row.iter().enumerate() {
            // Only consider positions that are in the valid movement mask
            if !nbhr_movement.mask[dr][dc] {
                continue;
            }

            // Skip center cell (though it's still a valid "move" to stay)
            if dr == nbhr_movement.center_row && dc == nbhr_movement.center_col {
                continue;
            }

            let val = neighbor.get_trait(0);
            if val > best_val {
                best_val = val;
                best_move = (
                    dr as isize - nbhr_movement.center_row as isize,
                    dc as isize - nbhr_movement.center_col as isize,
                );
            }
        }
    }

    best_move
}

/// Move away from crowded areas (only to valid mask positions)
pub fn movement_avoid_crowding(_cell: &Cell, nbhr_movement: &Neighborhood) -> (isize, isize) {
    let mut sum = 0.0;
    let mut count = 0;

    for (dr, row) in nbhr_movement.cells.iter().enumerate() {
        for (dc, neighbor) in row.iter().enumerate() {
            if nbhr_movement.mask[dr][dc] &&
               !(dr == nbhr_movement.center_row && dc == nbhr_movement.center_col) {

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
        // Collect all valid movement positions from the mask
        let mut valid_moves = Vec::new();
        
        for (dr, row) in nbhr_movement.mask.iter().enumerate() {
            for (dc, &is_valid) in row.iter().enumerate() {
                if is_valid {
                    let move_dr = dr as isize - nbhr_movement.center_row as isize;
                    let move_dc = dc as isize - nbhr_movement.center_col as isize;
                    valid_moves.push((move_dr, move_dc));
                }
            }
        }
        
        if valid_moves.is_empty() {
            return (0, 0);
        }
        
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..valid_moves.len());
        valid_moves[idx]
    } else {
        (0, 0)
    }
}

/// Trait-based exploratory movement (only to valid mask positions)
pub fn movement_trait_based(cell: &Cell, nbhr_movement: &Neighborhood) -> (isize, isize) {
    let trait0 = cell.get_trait(0);
    let trait1 = cell.get_trait(1);

    let mut sum = 0.0;
    let mut count = 0;

    for (dr, row) in nbhr_movement.cells.iter().enumerate() {
        for (dc, neighbor) in row.iter().enumerate() {
            if nbhr_movement.mask[dr][dc] &&
               !(dr == nbhr_movement.center_row && dc == nbhr_movement.center_col) &&
               !nbhr_movement.cells[dr][dc].is_empty() {

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

    // Collect all valid movement positions from the mask
    let mut valid_moves = Vec::new();
    
    for (dr, row) in nbhr_movement.mask.iter().enumerate() {
        for (dc, &is_valid) in row.iter().enumerate() {
            if is_valid {
                let move_dr = dr as isize - nbhr_movement.center_row as isize;
                let move_dc = dc as isize - nbhr_movement.center_col as isize;
                valid_moves.push((move_dr, move_dc));
            }
        }
    }
    
    if valid_moves.is_empty() {
        return (0, 0);
    }

    // Explore if isolated
    if trait0 > 0.7 && avg_neighbor_trait0 < 0.3 {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..valid_moves.len());
        return valid_moves[idx];
    }

    // Stay if stable
    if trait1 > 0.8 {
        return (0, 0);
    }

    // Small random jitter
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.3) {
        let idx = rng.gen_range(0..valid_moves.len());
        valid_moves[idx]
    } else {
        (0, 0)
    }
}


#[derive(Clone, Copy, PartialEq)]
enum ResolveState {
    Unvisited,
    Visiting,      // Currently processing (detects cycles)
    Empty,
    Resolved(bool), // Result: true = moving, false = staying
}

pub fn apply_movement(movement_fn: fn(&Cell, &Neighborhood) -> (isize, isize),
                      nbhr_movement_base: &Neighborhood,
                      grid: &Grid,
                      ) -> Vec<Vec<Cell>> {

    let height = grid.height;
    let width = grid.width;

    let mut reserved: Vec<Vec<Option<(usize, usize)>>> = vec![vec![None; width]; height];
    let mut states = vec![vec![ResolveState::Unvisited; width]; height]; // this cell can do its move or no (or we don't know yet)

    // 1. Calculate Intentions
    // intentions[r][c] = (target_row, target_col)
    let mut intentions = vec![vec![(0, 0); width]; height];
    for r in 0..height {
        for c in 0..width {
            let cell = &grid.cells[r][c];
            if cell.is_empty() {
                states[r][c] = ResolveState::Empty;
                continue;
            }

            let nbhr_movement = Neighborhood::new_from_base(r, c, nbhr_movement_base, grid);
            let (dr, dc) = movement_fn(cell, &nbhr_movement);

            // Validate that the movement is within the allowed neighborhood mask
            let target_dr = (nbhr_movement.center_row as isize + dr) as usize;
            let target_dc = (nbhr_movement.center_col as isize + dc) as usize;
            
            // Check if the move is valid according to the mask
            let is_valid_move = if target_dr < nbhr_movement.height && target_dc < nbhr_movement.width {
                nbhr_movement.mask[target_dr][target_dc]
            } else {
                false
            };

            let (tr, tc) = if is_valid_move {
                // Movement is valid, calculate wrapped coordinates
                (
                    ((r as isize + dr).rem_euclid(height as isize)) as usize,
                    ((c as isize + dc).rem_euclid(width as isize)) as usize
                )
            } else {
                // Movement is invalid, stay in place
                (r, c)
            };
            
            intentions[r][c] = (tr, tc);
        }
    }

    // 2. Resolve Movement Logic
    // reserved[r][c] = Who claimed this target? None or Some((claimant_r, claimant_c))
    for r in 0..height {
        for c in 0..width {
            if states[r][c] == ResolveState::Unvisited {
                resolve_move(r, c, &intentions, &mut reserved, &mut states);
            }
        }
    }

    // 3. Construct and fill the new Grid
    let mut new_cells = vec![vec![Cell::new((0, 0)); width]; height];
    for r in 0..height {
        for c in 0..width {
            new_cells[r][c] = Cell::empty_at((r, c));
        }
    }
    for r in 0..height {
        for c in 0..width {
            let mut cell = grid.cells[r][c].clone();
            if cell.is_empty() {
                continue;
            }
            
            match states[r][c] {
                ResolveState::Resolved(true) => {
                    // This cell is allowed to move
                    let (tr, tc) = intentions[r][c];
                    cell.position = (tr, tc);
                    new_cells[tr][tc] = cell;
                },
                _ => {
                    // This cell is blocked or staying.
                    new_cells[r][c] = cell;
                }
            }
        }
    }

    new_cells
}

fn resolve_move(r: usize,
                c: usize,
                intentions: &Vec<Vec<(usize, usize)>>,
                reserved: &mut Vec<Vec<Option<(usize, usize)>>>,
                states: &mut Vec<Vec<ResolveState>>,
                ) -> bool {
    
    // 1. Check Cache / Cycles
    match states[r][c] {
        ResolveState::Resolved(result) => return result,
        ResolveState::Visiting => {
            // Cycle Detected (e.g. A->B->A).
            return true; 
        },
        _ => {},
    }

    // Mark as currently visiting to detect loops
    states[r][c] = ResolveState::Visiting;

    let (tr, tc) = intentions[r][c];

    // 2. Handle Self-Movement (Staying put)
    if tr == r && tc == c {
        // I am moving to myself. This is always allowed
        // I "reserve" my own spot.
        reserved[tr][tc] = Some((r, c));
        states[r][c] = ResolveState::Resolved(true);
        return true;

    // 3. Contention Check (First to claim wins)
    } else if let Some((_owner_r, _owner_c)) = reserved[tr][tc] {
        // Someone already claimed this target.
        states[r][c] = ResolveState::Resolved(false);
        return false;
    } else {
        // Spot is free to claim. I claim it.
        reserved[tr][tc] = Some((r, c));
    }

    // if the target cell is empty
    if states[tr][tc] == ResolveState::Empty {
        states[r][c] = ResolveState::Resolved(true);
        return true;
    }

    // 4. Dependency Check (Phantom Collision)
    // I can move to (tr, tc) IF the person currently there moves away.
    // Since every cell is an actor, we check the actor at (tr, tc).
    
    let occupant_can_move = resolve_move(tr, tc, intentions, reserved, states);
    
    // Determine if the occupant is actually vacating the square.
    // They vacate if:
    // a) They are allowed to move (occupant_can_move == true)
    // b) AND their target is NOT the current spot (they aren't just moving to themselves).
    let (occ_tr, occ_tc) = intentions[tr][tc];
    let occupant_vacating = occupant_can_move && ((occ_tr, occ_tc) != (tr, tc));

    if occupant_vacating {
        states[r][c] = ResolveState::Resolved(true);
        true
    } else {
        // The actor in front of me is staying.
        // Therefore, I must stay.
        states[r][c] = ResolveState::Resolved(false);
        false
    }
}