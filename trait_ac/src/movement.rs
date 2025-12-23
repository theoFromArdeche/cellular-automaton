use crate::cell::Cell;
use crate::grid::Grid;
use crate::neighborhood::{Neighborhood, NeighborhoodSettings};
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use rand::prelude::*;



pub struct Movements;

impl Movements {
    /// No movement - cells stay in place
    #[inline(always)]
    pub fn static_movement(_cell: &Cell, _neighborhood_mvt: &Neighborhood) -> (isize, isize) {
        (0, 0)
    }

    /// Random walk - move randomly to any valid position in the neighborhood mask
    pub fn random_movement(_cell: &Cell, neighborhood_mvt: &Neighborhood) -> (isize, isize) {
        // Pre-computed valid moves would be better, but this is still reasonable
        let mut valid_moves = [(0isize, 0isize); 9]; // Max 9 positions in 3x3
        let mut count = 0;
        
        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;
        
        for dr in 0..neighborhood_mvt.height {
            for dc in 0..neighborhood_mvt.width {
                if unsafe { *neighborhood_mvt.mask.get_unchecked(dr).get_unchecked(dc) } {
                    valid_moves[count] = (
                        dr as isize - center_row as isize,
                        dc as isize - center_col as isize
                    );
                    count += 1;
                }
            }
        }
        
        if count == 0 {
            return (0, 0);
        }
        
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..count);
        unsafe { *valid_moves.get_unchecked(idx) }
    }

    /// Move toward the neighbor with the highest trait value (gradient ascent)
    pub fn gradient(cell: &Cell, neighborhood_mvt: &Neighborhood) -> (isize, isize) {
        let current_val = cell.get_trait(0);
        let mut best_val = current_val;
        let mut best_move = (0, 0);

        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for dr in 0..neighborhood_mvt.height {
            for dc in 0..neighborhood_mvt.width {
                if !unsafe { *neighborhood_mvt.mask.get_unchecked(dr).get_unchecked(dc) } {
                    continue;
                }

                if dr == center_row && dc == center_col {
                    continue;
                }

                let neighbor = unsafe { neighborhood_mvt.cells.get_unchecked(dr).get_unchecked(dc) };
                let val = neighbor.get_trait(0);
                
                if val > best_val {
                    best_val = val;
                    best_move = (
                        dr as isize - center_row as isize,
                        dc as isize - center_col as isize,
                    );
                }
            }
        }

        best_move
    }

    /// Move away from high-density areas (gradient descent on density)
    pub fn avoid_crowding(_cell: &Cell, neighborhood_mvt: &Neighborhood) -> (isize, isize) {
        let mut sum = 0.0;
        let mut count = 0;

        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for dr in 0..neighborhood_mvt.height {
            for dc in 0..neighborhood_mvt.width {
                if unsafe { *neighborhood_mvt.mask.get_unchecked(dr).get_unchecked(dc) } &&
                !(dr == center_row && dc == center_col) {
                    
                    let neighbor = unsafe { neighborhood_mvt.cells.get_unchecked(dr).get_unchecked(dc) };
                    if !neighbor.is_empty() {
                        sum += neighbor.get_trait(0);
                        count += 1;
                    }
                }
            }
        }

        if count == 0 {
            return (0, 0);
        }

        let avg_density = sum / count as f32;

        if avg_density > 0.7 {
            let mut valid_moves = [(0isize, 0isize); 9];
            let mut move_count = 0;
            
            for dr in 0..neighborhood_mvt.height {
                for dc in 0..neighborhood_mvt.width {
                    if unsafe { *neighborhood_mvt.mask.get_unchecked(dr).get_unchecked(dc) } {
                        valid_moves[move_count] = (
                            dr as isize - center_row as isize,
                            dc as isize - center_col as isize,
                        );
                        move_count += 1;
                    }
                }
            }
            
            if move_count == 0 {
                return (0, 0);
            }
            
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0..move_count);
            unsafe { *valid_moves.get_unchecked(idx) }
        } else {
            (0, 0)
        }
    }

    /// Chemotaxis - move toward areas with specific trait combinations
    pub fn chemotaxis(cell: &Cell, neighborhood_mvt: &Neighborhood) -> (isize, isize) {
        let target_trait = 2; // Looking for cooperation trait
        let current_val = cell.get_trait(target_trait);
        
        let mut best_val = current_val;
        let mut best_move = (0, 0);

        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for dr in 0..neighborhood_mvt.height {
            for dc in 0..neighborhood_mvt.width {
                if !unsafe { *neighborhood_mvt.mask.get_unchecked(dr).get_unchecked(dc) } {
                    continue;
                }

                if dr == center_row && dc == center_col {
                    continue;
                }

                let neighbor = unsafe { neighborhood_mvt.cells.get_unchecked(dr).get_unchecked(dc) };
                if neighbor.is_empty() {
                    continue;
                }
                
                let val = neighbor.get_trait(target_trait);
                if val > best_val {
                    best_val = val;
                    best_move = (
                        dr as isize - center_row as isize,
                        dc as isize - center_col as isize,
                    );
                }
            }
        }

        best_move
    }

    /// Levy flight - occasional long-distance jumps with mostly local movement
    pub fn levy_flight(_cell: &Cell, neighborhood_mvt: &Neighborhood) -> (isize, isize) {
        let mut rng = rand::thread_rng();
        
        // 90% local movement, 10% long jump
        if rng.gen_bool(0.9) {
            // Local movement
            Movements::random_movement(_cell, neighborhood_mvt)
        } else {
            // Long jump - use full neighborhood extent
            let mut valid_moves = [(0isize, 0isize); 9];
            let mut count = 0;
            
            let center_row = neighborhood_mvt.center_row;
            let center_col = neighborhood_mvt.center_col;
            
            for dr in 0..neighborhood_mvt.height {
                for dc in 0..neighborhood_mvt.width {
                    if unsafe { *neighborhood_mvt.mask.get_unchecked(dr).get_unchecked(dc) } {
                        valid_moves[count] = (
                            dr as isize - center_row as isize,
                            dc as isize - center_col as isize,
                        );
                        count += 1;
                    }
                }
            }
            
            if count == 0 {
                return (0, 0);
            }
            
            let idx = rng.gen_range(0..count);
            unsafe { *valid_moves.get_unchecked(idx) }
        }
    }

    /// Multi-trait based movement - considers multiple traits for decision
    pub fn multi_trait(cell: &Cell, neighborhood_mvt: &Neighborhood) -> (isize, isize) {
        let energy = cell.get_trait(0);
        let mobility = cell.get_trait(5);
        let aggression = cell.get_trait(3);
        
        // High mobility = more likely to move
        let mut rng = rand::thread_rng();
        if mobility < 0.3 {
            return (0, 0); // Low mobility = stay put
        }
        
        // High energy + high aggression = seek highest value neighbor
        if energy > 0.6 && aggression > 0.6 {
            return Movements::gradient(cell, neighborhood_mvt);
        }
        
        // Low energy = avoid crowding
        if energy < 0.3 {
            return Movements::avoid_crowding(cell, neighborhood_mvt);
        }
        
        // Default: weighted random based on mobility
        if rng.gen_bool(mobility as f64) {
            Movements::random_movement(cell, neighborhood_mvt)
        } else {
            (0, 0)
        }
    }

    /// Directional persistence - tend to keep moving in the same direction
    pub fn persistent(cell: &Cell, neighborhood_mvt: &Neighborhood) -> (isize, isize) {
        // Use traits to encode "momentum" or direction memory
        let momentum_x = cell.get_trait(7) * 2.0 - 1.0; // Convert [0,1] to [-1,1]
        let momentum_y = cell.get_trait(8) * 2.0 - 1.0;
        
        let mut rng = rand::thread_rng();
        
        // 70% chance to continue in similar direction
        if rng.gen_bool(0.7) && (momentum_x.abs() > 0.1 || momentum_y.abs() > 0.1) {
            let dr = momentum_y.round() as isize;
            let dc = momentum_x.round() as isize;
            
            // Validate against mask
            let target_dr = (neighborhood_mvt.center_row as isize + dr) as usize;
            let target_dc = (neighborhood_mvt.center_col as isize + dc) as usize;
            
            if target_dr < neighborhood_mvt.height && target_dc < neighborhood_mvt.width {
                if unsafe { *neighborhood_mvt.mask.get_unchecked(target_dr).get_unchecked(target_dc) } {
                    return (dr, dc);
                }
            }
        }
        
        // Otherwise random movement
        Movements::random_movement(cell, neighborhood_mvt)
    }
}





#[derive(Clone, Copy, PartialEq)]
enum ResolveState {
    Unvisited,
    Visiting,
    Empty,
    Resolved(bool),
}

pub type MovementFn = for<'c> fn(&Cell, &Neighborhood<'c>) -> (isize, isize);

#[derive(Clone, Copy)]
pub struct MovementRegistry {
    pub movement_function: MovementFn,
}

// Static lookup table for function pointer to name mapping
static MOVEMENT_LOOKUP: &[(MovementFn, &str)] = &[
    (Movements::static_movement, "static"),
    (Movements::random_movement, "random"),
    (Movements::gradient, "gradient"),
    (Movements::avoid_crowding, "avoid crowding"),
    (Movements::chemotaxis, "chemotaxis"),
    (Movements::levy_flight, "levy flight"),
    (Movements::multi_trait, "multi trait"),
    (Movements::persistent, "persistent"),
    // Add more movements here as needed
];

const MOVEMENT_COUNT: usize = MOVEMENT_LOOKUP.len();

const fn extract_movement_names<'a, const N: usize>(lookup: &'a [(MovementFn, &'a str)]) -> [&'a str; N] {
    let mut names = [""; N];
    let mut i = 0;
    while i < N {
        names[i] = lookup[i].1;
        i += 1;
    }
    names
}

// Static array of just the names, extracted from lookup table
static MOVEMENT_NAMES: [&str; MOVEMENT_COUNT] = extract_movement_names::<MOVEMENT_COUNT>(MOVEMENT_LOOKUP);

impl MovementRegistry {
    pub fn default() -> Self {
        Self {
            movement_function: Movements::static_movement,
        }
    }

    pub fn custom(movement_function: MovementFn) -> Self {
        Self { movement_function }
    }

    /// Get the name of the current movement function
    #[inline(always)]
    pub fn get_movement_name(&self) -> &'static str {
        Self::get_name_for_movement(self.movement_function)
    }

    pub fn set_movement_function(&mut self, movement_fn: MovementFn) {
        self.movement_function = movement_fn;
    }

    pub fn is_stored_function(self, function: MovementFn) -> bool {
        self.movement_function as usize == function as usize
    }

    /// Get the name for a specific movement function (uses lookup table)
    #[inline]
    pub fn get_name_for_movement(movement_fn: MovementFn) -> &'static str {
        for &(func, name) in MOVEMENT_LOOKUP {
            if func as usize == movement_fn as usize {
                return name;
            }
        }
        "unknown"
    }

    /// Get movement function by name (uses lookup table)
    #[inline]
    pub fn get_movement_by_name(movement_name: &str) -> Option<MovementFn> {
        for &(func, name) in MOVEMENT_LOOKUP {
            if name == movement_name {
                return Some(func);
            }
        }
        None
    }

    /// Get all available movement names (from lookup table)
    #[inline(always)]
    pub fn get_all_names() -> &'static [&'static str; MOVEMENT_COUNT] {
        &MOVEMENT_NAMES
    }

    pub fn apply_movement(&self, neighborhood_mvt_settings: &NeighborhoodSettings, grid: &Grid) -> Vec<Vec<Cell>> {
        let height = grid.height;
        let width = grid.width;

        // A flat buffer to store "Bids" for target cells. 
        // Format: High 32 bits = Random Priority, Low 32 bits = Source Cell Index.
        // This allows us to use atomic_max to pick a random winner in parallel.
        let mut claims = Vec::with_capacity(width * height);
        claims.resize_with(width * height, || AtomicU64::new(0));

        // --- Phase 1: Parallel Intention Calculation & Bidding ---
        let mut intentions: Vec<Vec<(usize, usize)>> = grid.cells
            .par_iter()
            .enumerate()
            .map(|(r, row)| {
                let mut row_intentions = Vec::with_capacity(width);
                let mut rng = rand::thread_rng(); // Thread-local RNG for performance

                for (c, cell) in row.iter().enumerate() {
                    if cell.is_empty() {
                        row_intentions.push((r, c));
                        continue;
                    }

                    // 1. Calculate Move
                    let neighborhood_mvt = Neighborhood::new_from_settings(r, c, neighborhood_mvt_settings, grid);
                    let (dr, dc) = (self.movement_function)(cell, &neighborhood_mvt);

                    let target_dr = (neighborhood_mvt.center_row as isize + dr) as usize;
                    let target_dc = (neighborhood_mvt.center_col as isize + dc) as usize;
                    
                    let is_valid_move = if target_dr < neighborhood_mvt.height && target_dc < neighborhood_mvt.width {
                        unsafe { *neighborhood_mvt.mask.get_unchecked(target_dr).get_unchecked(target_dc) }
                    } else {
                        false
                    };

                    let (tr, tc) = if is_valid_move {
                        (
                            ((r as isize + dr).rem_euclid(height as isize)) as usize,
                            ((c as isize + dc).rem_euclid(width as isize)) as usize
                        )
                    } else {
                        (r, c)
                    };

                    // 2. Bid for the target (Random Collision Resolution)
                    if (tr, tc) != (r, c) {
                        let target_flat_idx = tr * width + tc;
                        let source_flat_idx = r * width + c;
                        
                        // Generate random priority (using high bits ensures randomness)
                        let priority: u32 = rng.next_u32(); 
                        // Pack: Priority (top 32) | Source Index (bottom 32)
                        let bid = ((priority as u64) << 32) | (source_flat_idx as u64);

                        // Atomic Max: Attempt to write our bid. If a higher bid exists, we lose.
                        // Relaxed ordering is sufficient as we synchronize via the next phase.
                        unsafe {
                            claims.get_unchecked(target_flat_idx).fetch_max(bid, Ordering::Relaxed);
                        }
                    }

                    row_intentions.push((tr, tc));
                }
                row_intentions
            })
            .collect();

        // --- Phase 2: Parallel Pruning ---
        // Check if we won the bid. If not, reset intention to (r,c).
        intentions.par_iter_mut().enumerate().for_each(|(r, row)| {
            for (c, target) in row.iter_mut().enumerate() {
                let (tr, tc) = *target;
                
                // If we intended to move...
                if (tr, tc) != (r, c) {
                    let target_flat_idx = tr * width + tc;
                    let source_flat_idx = r * width + c;
                    
                    let winning_bid = unsafe { claims.get_unchecked(target_flat_idx).load(Ordering::Relaxed) };
                    let winner_idx = (winning_bid & 0xFFFFFFFF) as usize;

                    // If the winner is not us, we must cancel our move.
                    if winner_idx != source_flat_idx {
                        *target = (r, c);
                    }
                }
            }
        });

        // --- Phase 3: Resolve Logic (Sequential DFS) ---
        // The graph is now simplified (max 1 incoming per cell), but we still need DFS
        // to handle "vacancy chains" (A->B is only valid if B moves away).
        let mut reserved: Vec<Vec<Option<(usize, usize)>>> = vec![vec![None; width]; height];
        let mut states = vec![vec![ResolveState::Unvisited; width]; height];

        for r in 0..height {
            for c in 0..width {
                let cell = unsafe { grid.cells.get_unchecked(r).get_unchecked(c) };
                if cell.is_empty() {
                    unsafe { *states.get_unchecked_mut(r).get_unchecked_mut(c) = ResolveState::Empty; }
                }
            }
        }

        for r in 0..height {
            for c in 0..width {
                if unsafe { *states.get_unchecked(r).get_unchecked(c) } == ResolveState::Unvisited {
                    self.resolve_move(r, c, &intentions, &mut reserved, &mut states);
                }
            }
        }

        // --- Phase 4: Construct New Grid (Parallel) ---
        // We can construct the rows in parallel.
        let new_cells: Vec<Vec<Cell>> = (0..height).into_par_iter().map(|r| {
            let mut row_cells = Vec::with_capacity(width);
            for c in 0..width {
                let cell = unsafe { grid.cells.get_unchecked(r).get_unchecked(c) };

                if cell.is_empty() {
                    // If cell is empty, check if someone moved into us (reserved holds the source)
                    match unsafe { *reserved.get_unchecked(r).get_unchecked(c) } {
                        Some((sr, sc)) => {
                            let source_cell = unsafe { grid.cells.get_unchecked(sr).get_unchecked(sc) };
                            let mut moved_cell = source_cell.clone();
                            moved_cell.position = (r, c);
                            row_cells.push(moved_cell);
                        },
                        None => row_cells.push(Cell::empty_at((r, c))),
                    }
                    continue;
                }

                let state = unsafe { *states.get_unchecked(r).get_unchecked(c) };
                match state {
                    ResolveState::Resolved(true) => {
                        // We moved away successfully.
                        // Did someone fill our spot?
                        match unsafe { *reserved.get_unchecked(r).get_unchecked(c) } {
                            Some((sr, sc)) => {
                                let source_cell = unsafe { grid.cells.get_unchecked(sr).get_unchecked(sc) };
                                let mut moved_cell = source_cell.clone();
                                moved_cell.position = (r, c);
                                row_cells.push(moved_cell);
                            },
                            None => row_cells.push(Cell::empty_at((r, c))),
                        }
                    },
                    _ => {
                        // We stayed put (either intentionally or blocked).
                        row_cells.push(cell.clone());
                    }
                }
            }
            row_cells
        }).collect();

        new_cells
    }

    // Keeping your DFS logic helper exactly as is (it's efficient for the simplified graph)
    #[inline]
    fn resolve_move(&self, 
                    r: usize,
                    c: usize,
                    intentions: &Vec<Vec<(usize, usize)>>,
                    reserved: &mut Vec<Vec<Option<(usize, usize)>>>,
                    states: &mut Vec<Vec<ResolveState>>,
                    ) -> bool {

        let state = unsafe { *states.get_unchecked(r).get_unchecked(c) };
        
        match state {
            ResolveState::Resolved(result) => return result,
            ResolveState::Visiting => return true,
            _ => {},
        }

        unsafe { *states.get_unchecked_mut(r).get_unchecked_mut(c) = ResolveState::Visiting; }

        let (tr, tc) = unsafe { *intentions.get_unchecked(r).get_unchecked(c) };

        if tr == r && tc == c {
            unsafe { 
                *reserved.get_unchecked_mut(tr).get_unchecked_mut(tc) = Some((r, c));
                *states.get_unchecked_mut(r).get_unchecked_mut(c) = ResolveState::Resolved(true);
            }
            return true;
        } else if unsafe { reserved.get_unchecked(tr).get_unchecked(tc) }.is_some() {
            unsafe { *states.get_unchecked_mut(r).get_unchecked_mut(c) = ResolveState::Resolved(false); }
            return false;
        } else {
            unsafe { *reserved.get_unchecked_mut(tr).get_unchecked_mut(tc) = Some((r, c)); }
        }

        if unsafe { *states.get_unchecked(tr).get_unchecked(tc) } == ResolveState::Empty {
            unsafe { *states.get_unchecked_mut(r).get_unchecked_mut(c) = ResolveState::Resolved(true); }
            return true;
        }

        let occupant_can_move = self.resolve_move(tr, tc, intentions, reserved, states);
        
        let (occ_tr, occ_tc) = unsafe { *intentions.get_unchecked(tr).get_unchecked(tc) };
        let occupant_vacating = occupant_can_move && ((occ_tr, occ_tc) != (tr, tc));

        if occupant_vacating {
            unsafe { *states.get_unchecked_mut(r).get_unchecked_mut(c) = ResolveState::Resolved(true); }
            true
        } else {
            unsafe { *states.get_unchecked_mut(r).get_unchecked_mut(c) = ResolveState::Resolved(false); }
            false
        }
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;
    use crate::neighborhood::Neighborhood;

    fn build_test_grid() -> Grid {
        let mut grid = Grid::new(3, 3);
        for r in 0..3 {
            for c in 0..3 {
                for index in 0..9 {
                    grid.cells[r][c].set_trait(index, 0.5);
                }
            }
        }
        grid
    }

    #[test]
    fn test_static_movement() {
        let grid = build_test_grid();
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let neighborhood = Neighborhood::new(3, 3, 1, 1, 1, 1, &mask, &grid);
        let cell = &grid.cells[1][1];
        let mv = Movements::static_movement(cell, &neighborhood);
        assert_eq!(mv, (0, 0), "Static movement should not move");
    }

    #[test]
    fn test_random_movement_with_full_mask() {
        let grid = build_test_grid();
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let neighborhood = Neighborhood::new(3, 3, 1, 1, 1, 1, &mask, &grid);
        let cell = &grid.cells[1][1];
        for _ in 0..10 {
            let mv = Movements::random_movement(cell, &neighborhood);
            let dr = mv.0 + neighborhood.center_row as isize;
            let dc = mv.1 + neighborhood.center_col as isize;
            assert!(dr >= 0 && dr < 3 && dc >= 0 && dc < 3, "Random movement must stay within neighborhood");
        }
    }

    #[test]
    fn test_gradient_moves_toward_highest_trait() {
        let mut grid = build_test_grid();
        // Set a neighbor with higher trait
        grid.cells[0][1].set_trait(0, 0.9);
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let neighborhood = Neighborhood::new(3, 3, 1, 1, 1, 1, &mask, &grid);
        let cell = &grid.cells[1][1];
        let mv = Movements::gradient(cell, &neighborhood);
        assert_eq!(mv, (-1, 0), "Gradient should move toward highest trait neighbor");
    }

    #[test]
    fn test_avoid_crowding_stays_put_if_low_density() {
        let grid = build_test_grid();
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let neighborhood = Neighborhood::new(3, 3, 1, 1, 1, 1, &mask, &grid);
        let cell = &grid.cells[1][1];
        let mv = Movements::avoid_crowding(cell, &neighborhood);
        assert_eq!(mv, (0, 0), "Avoid crowding should stay put if density is low");
    }

    #[test]
    fn test_chemotaxis_moves_toward_target_trait() {
        let mut grid = build_test_grid();
        grid.cells[0][0].set_trait(2, 1.0); // Cooperation trait
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let neighborhood = Neighborhood::new(3, 3, 1, 1, 1, 1, &mask, &grid);
        let cell = &grid.cells[1][1];
        let mv = Movements::chemotaxis(cell, &neighborhood);
        assert_eq!(mv, (-1, -1), "Chemotaxis should move toward neighbor with higher target trait");
    }

    #[test]
    fn test_movement_registry_static() {
        let grid = build_test_grid();
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let neighborhood = Neighborhood::new(3, 3, 1, 1, 1, 1, &mask, &grid);
        let registry = MovementRegistry::default();
        let new_cells = registry.apply_movement(&neighborhood, &grid);
        // All cells should remain in place
        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(new_cells[r][c].position, (r, c));
            }
        }
    }
}
