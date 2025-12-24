use crate::cell::Cell;
use crate::grid::Grid;
use crate::neighborhood::Neighborhood;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use rand::prelude::*;



pub struct Movements;

impl Movements {
    /// No movement - cells stay in place
    #[inline(always)]
    pub fn static_movement(_cell: &Cell, _neighborhood_mvt: &Neighborhood, _grid: &Grid) -> (isize, isize) {
        (0, 0)
    }

    /// Random walk - move randomly to any valid position in the neighborhood mask
    pub fn random_movement(_cell: &Cell, neighborhood_mvt: &Neighborhood, _grid: &Grid) -> (isize, isize) {
        let mut valid_moves = [(0isize, 0isize); 9]; // Max 9 positions in 3x3
        let mut count = 0;
        
        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for mask_r in 0..neighborhood_mvt.height {
            for mask_c in 0..neighborhood_mvt.width {
                if neighborhood_mvt.is_valid(mask_r, mask_c) {                    
                    valid_moves[count] = (
                        mask_r as isize - center_row as isize,
                        mask_c as isize - center_col as isize
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
    pub fn gradient(cell: &Cell, neighborhood_mvt: &Neighborhood, grid: &Grid) -> (isize, isize) {
        let current_val = cell.get_trait(0);
        let mut best_val = current_val;
        let mut best_move = (0, 0);

        let (cell_r, cell_c) = cell.position;
        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for mask_r in 0..neighborhood_mvt.height {
            for mask_c in 0..neighborhood_mvt.width {
                if neighborhood_mvt.is_valid(mask_r, mask_c) &&
                   !(mask_r == center_row && mask_c == center_col) {

                    let (grid_r, grid_c) = neighborhood_mvt.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_value(grid_r, grid_c, 0);
                
                    if !neighbor_is_empty && neighbor_value > best_val {
                        best_val = neighbor_value;
                        best_move = (
                            mask_r as isize - center_row as isize,
                            mask_c as isize - center_col as isize,
                        );
                    }
                }
            }
        }

        best_move
    }

    /// Move away from high-density areas (gradient descent on density)
    pub fn avoid_crowding(cell: &Cell, neighborhood_mvt: &Neighborhood, grid: &Grid) -> (isize, isize) {
        let mut sum = 0.0;
        let mut count = 0;

        let (cell_r, cell_c) = cell.position;
        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for mask_r in 0..neighborhood_mvt.height {
            for mask_c in 0..neighborhood_mvt.width {
                if neighborhood_mvt.is_valid(mask_r, mask_c) &&
                   !(mask_r == center_row && mask_c == center_col) {

                    let (grid_r, grid_c) = neighborhood_mvt.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_value(grid_r, grid_c, 0);
                
                    if !neighbor_is_empty {
                        sum += neighbor_value;
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
            
            for mask_r in 0..neighborhood_mvt.height {
                for mask_c in 0..neighborhood_mvt.width {
                    if neighborhood_mvt.is_valid(mask_r, mask_c) {                    
                        valid_moves[count] = (
                            mask_r as isize - center_row as isize,
                            mask_c as isize - center_col as isize
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
    pub fn chemotaxis(cell: &Cell, neighborhood_mvt: &Neighborhood, grid: &Grid) -> (isize, isize) {
        let target_trait = 2; // Looking for cooperation trait
        let current_val = cell.get_trait(target_trait);
        
        let mut best_val = current_val;
        let mut best_move = (0, 0);

        let (cell_r, cell_c) = cell.position;
        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for mask_r in 0..neighborhood_mvt.height {
            for mask_c in 0..neighborhood_mvt.width {
                if neighborhood_mvt.is_valid(mask_r, mask_c) &&
                   !(mask_r == center_row && mask_c == center_col) {

                    let (grid_r, grid_c) = neighborhood_mvt.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_value(grid_r, grid_c, target_trait);
                
                    if !neighbor_is_empty && neighbor_value > best_val {                
                        best_val = neighbor_value;
                        best_move = (
                            mask_r as isize - center_row as isize,
                            mask_c as isize - center_col as isize,
                        );
                    }
                }
            }
        }

        best_move
    }

    /// Levy flight - occasional long-distance jumps with mostly local movement
    pub fn levy_flight(_cell: &Cell, neighborhood_mvt: &Neighborhood, _grid: &Grid) -> (isize, isize) {
        let mut rng = rand::thread_rng();
        
        // 90% local movement, 10% long jump
        if rng.gen_bool(0.9) {
            // Local movement
            Movements::random_movement(_cell, neighborhood_mvt, _grid)
        } else {
            // Long jump - use full neighborhood extent
            let mut valid_moves = [(0isize, 0isize); 9];
            let mut count = 0;
            
            let center_row = neighborhood_mvt.center_row;
            let center_col = neighborhood_mvt.center_col;
            
            for mask_r in 0..neighborhood_mvt.height {
                for mask_c in 0..neighborhood_mvt.width {
                    if neighborhood_mvt.is_valid(mask_r, mask_c) {
                        valid_moves[count] = (
                            mask_r as isize - center_row as isize,
                            mask_c as isize - center_col as isize,
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
    pub fn multi_trait(cell: &Cell, neighborhood_mvt: &Neighborhood, _grid: &Grid) -> (isize, isize) {
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
            return Movements::gradient(cell, neighborhood_mvt, _grid);
        }
        
        // Low energy = avoid crowding
        if energy < 0.3 {
            return Movements::avoid_crowding(cell, neighborhood_mvt, _grid);
        }
        
        // Default: weighted random based on mobility
        if rng.gen_bool(mobility as f64) {
            Movements::random_movement(cell, neighborhood_mvt, _grid)
        } else {
            (0, 0)
        }
    }
}





#[derive(Clone, Copy, PartialEq)]
enum ResolveState {
    Unvisited,
    Visiting,
    Empty,
    Resolved(bool),
}

pub type MovementFn = fn(&Cell, &Neighborhood, &Grid) -> (isize, isize);

pub struct MovementRegistry {
    pub movement_function: MovementFn,
    // Stores bids: High 32 bits = Priority, Low 32 bits = Source Index
    claims: Vec<AtomicU64>, 
    // Stores target (r, c) for every cell. Flattened index = r * width + c
    intentions: Vec<(u16, u16)>, 
    // DFS helper: Flattened
    reserved: Vec<Option<(u16, u16)>>,
    // DFS helper: Flattened
    states: Vec<ResolveState>,
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

    pub fn custom(width: usize, height: usize, movement_function: MovementFn) -> Self {
        let size = width * height;
        Self {
            movement_function: movement_function,
            claims: (0..size).map(|_| AtomicU64::new(0)).collect(),
            intentions: vec![(0, 0); size],
            reserved: vec![None; size],
            states: vec![ResolveState::Unvisited; size],
        }
    }

    pub fn new(width: usize, height: usize) -> Self {
        MovementRegistry::custom(width, height, Movements::static_movement)
    }

    // Fast reset without deallocating
    pub fn prepare(&mut self, width: usize, height: usize) {
        let size = width * height;
        
        // Resize if grid changed (rare, but safe)
        if self.claims.len() != size {
            self.claims = (0..size).map(|_| AtomicU64::new(0)).collect();
            self.intentions.resize(size, (0, 0));
            self.reserved.resize(size, None);
            self.states.resize(size, ResolveState::Unvisited);
        } else {
            // Reset values efficiently
            // Parallel reset is faster for large arrays
            self.claims.par_iter().for_each(|x| x.store(0, Ordering::Relaxed));
            
            // We don't strictly need to reset intentions/states/reserved here 
            // because we overwrite them during the logic, but resetting states is good practice
            // if logic depends on "Unvisited" initial state.
            self.reserved.par_iter_mut().for_each(|x| *x = None);
            self.states.par_iter_mut().for_each(|x| *x = ResolveState::Unvisited);
        }
    }

    /// Get the name of the current movement function
    #[inline(always)]
    pub fn get_movement_name(&self) -> &'static str {
        self.get_name_for_movement(self.movement_function)
    }

    pub fn set_movement_function(&mut self, movement_fn: MovementFn) {
        self.movement_function = movement_fn;
    }

    pub fn is_stored_function(&self, function: MovementFn) -> bool {
        self.movement_function as usize == function as usize
    }

    /// Get the name for a specific movement function (uses lookup table)
    #[inline]
    pub fn get_name_for_movement(&self, movement_fn: MovementFn) -> &'static str {
        for &(func, name) in MOVEMENT_LOOKUP {
            if func as usize == movement_fn as usize {
                return name;
            }
        }
        "unknown"
    }

    /// Get movement function by name (uses lookup table)
    #[inline]
    pub fn get_movement_by_name(&self, movement_name: &str) -> Option<MovementFn> {
        for &(func, name) in MOVEMENT_LOOKUP {
            if name == movement_name {
                return Some(func);
            }
        }
        None
    }

    /// Get all available movement names (from lookup table)
    #[inline(always)]
    pub fn get_all_names(&self) -> &'static [&'static str; MOVEMENT_COUNT] {
        &MOVEMENT_NAMES
    }

    pub fn apply_movement(&mut self,
                          neighborhood_mvt: &Neighborhood,
                          grid: &Grid,
                          next_grid: &mut Vec<Vec<Cell>>, // Double Buffer Target
                          ) {

        let height = grid.height;
        let width = grid.width;

        // 0. Reset Workspace (Zero allocation)
        self.prepare(width, height);

        // 1. Batch Size Calculation
        // We want enough work to offset thread startup costs. 
        // ~4000-8000 cells per batch is usually a sweet spot for simple logic.
        let rows_per_batch = std::cmp::max(1, 4000 / width); 
        let chunk_len = width * rows_per_batch;

        // --- Phase 1: Parallel Bidding (Batched) ---
        self.intentions
            .par_chunks_mut(chunk_len)
            .zip(grid.cells.par_chunks(rows_per_batch)) // Note: grid.cells is Vec<Vec>, so this chunks rows
            .enumerate()
            .for_each(|(batch_idx, (intention_slab, grid_row_slab))| {
                let mut rng = rand::thread_rng();
                let start_r = batch_idx * rows_per_batch;

                // Iterate through the rows in this batch
                for (local_r, row_cells) in grid_row_slab.iter().enumerate() {
                    let r = start_r + local_r;
                    
                    // Get the slice of the intention slab corresponding to this row
                    let intention_row = &mut intention_slab[local_r * width .. (local_r + 1) * width];

                    for (c, cell) in row_cells.iter().enumerate() {
                        // Default: Stay put
                        intention_row[c] = (r as u16, c as u16);

                        if cell.is_empty() { continue; }

                        // 1. Calculate Move logic
                        let (dr, dc) = (self.movement_function)(cell, &neighborhood_mvt, grid);

                        let target_dr = (neighborhood_mvt.center_row as isize + dr) as usize;
                        let target_dc = (neighborhood_mvt.center_col as isize + dc) as usize;
                        
                        let is_valid_move = if target_dr < neighborhood_mvt.height && target_dc < neighborhood_mvt.width {
                            unsafe { *neighborhood_mvt.mask.get_unchecked(target_dr).get_unchecked(target_dc) }
                        } else {
                            false
                        };

                        let (tr, tc) = if is_valid_move {
                            (
                                ((r as isize + dr).clamp(0, height as isize - 1)) as usize,
                                ((c as isize + dc).clamp(0, width as isize - 1)) as usize
                            )
                        } else { (r, c) };

                        // 2. Bid
                        if (tr, tc) != (r, c) {
                            let target_flat = tr * width + tc;
                            let source_flat = r * width + c;
                            
                            let priority: u32 = rng.next_u32();
                            // Priority | Source Index
                            let bid = ((priority as u64) << 32) | (source_flat as u64);

                            // We can use relaxed ordering safely here
                            unsafe {
                                self.claims.get_unchecked(target_flat).fetch_max(bid, Ordering::Relaxed);
                            }
                            
                            intention_row[c] = (tr as u16, tc as u16);
                        }
                    }
                }
            });
        

        // --- Phase 2: Pruning (Also Batched) ---
        // Verify winners. If we lost, reset intention to (r,c)
        // We can use a larger chunk size here because the logic is simpler (just checking atomic)
        let prune_chunk_size = width * 50; 
        self.intentions
            .par_chunks_mut(prune_chunk_size)
            .enumerate()
            .for_each(|(chunk_id, chunk)| {
                let chunk_start_idx = chunk_id * prune_chunk_size;
                
                for (i, target) in chunk.iter_mut().enumerate() {
                    let (tr, tc) = (target.0 as usize, target.1 as usize);
                    let current_flat = chunk_start_idx + i;
                    let r = current_flat / width;
                    let c = current_flat % width;

                    if (tr, tc) == (r, c) { continue; }

                    let target_flat = tr * width + tc;
                    
                    // Relaxed Load
                    let winning_bid = unsafe { self.claims.get_unchecked(target_flat).load(Ordering::Relaxed) };
                    let winner_idx = (winning_bid & 0xFFFFFFFF) as usize;

                    if winner_idx != current_flat {
                        *target = (r as u16, c as u16);
                    }
                }
            });


        // --- Phase 3: Resolve (Sequential DFS) ---
        // Using flat buffers significantly improves cache hits here compared to Vec<Vec>
        // Initialize empty states
        for r in 0..height {
            for c in 0..width {
                let idx = r * width + c;
                // Unsafe access is safe here due to bounds, but standard indexing is likely optimized out
                if grid.cells[r][c].is_empty() {
                    self.states[idx] = ResolveState::Empty;
                }
            }
        }

        for r in 0..height {
            for c in 0..width {
                let idx = r * width + c;
                if self.states[idx] == ResolveState::Unvisited {
                    self.resolve_move(r, c, width);
                }
            }
        }

        // --- Phase 4: Construct Grid into Buffer (Parallel) ---
        // Write directly into `next_grid`. No allocation.
        next_grid.par_iter_mut().enumerate().for_each(|(r, row_cells)| {
            for (c, target_cell) in row_cells.iter_mut().enumerate() {
                let idx = r * width + c;
                
                // Current cell in the old grid
                let old_cell = &grid.cells[r][c];

                if old_cell.is_empty() {
                    // Did someone move here?
                    match self.reserved[idx] {
                        Some((sr, sc)) => {
                            *target_cell = grid.cells[sr as usize][sc as usize].clone();
                            target_cell.position = (r, c);
                        },
                        None => {
                            // Ensure it stays empty (might contain garbage from previous turn)
                            if !target_cell.is_empty() {
                                *target_cell = Cell::empty_at((r, c));
                            }
                        }
                    }
                    continue;
                }

                match self.states[idx] {
                    ResolveState::Resolved(true) => {
                        // We moved away. Who fills our spot?
                        match self.reserved[idx] {
                            Some((sr, sc)) => {
                                *target_cell = grid.cells[sr as usize][sc as usize].clone();
                                target_cell.position = (r, c);
                            },
                            None => {
                                *target_cell = Cell::empty_at((r, c));
                            }
                        }
                    },
                    _ => {
                        // We stayed put.
                        *target_cell = old_cell.clone();
                    }
                }
            }
        });
    }

    // Optimized DFS for flat buffers
    pub fn resolve_move(
        &mut self,
        r: usize, c: usize, w: usize,
    ) -> bool {
        let idx = r * w + c;
        
        // Manual state check to avoid function call overhead
        match self.states[idx] {
            ResolveState::Resolved(res) => return res,
            ResolveState::Visiting => return true, // Cycle detected, assume valid (or handle cycle logic)
            _ => {}
        }

        self.states[idx] = ResolveState::Visiting;

        let (tr_u16, tc_u16) = self.intentions[idx];
        let (tr, tc) = (tr_u16 as usize, tc_u16 as usize);
        let target_idx = tr * w + tc;

        // Self-move or blocked logic
        if target_idx == idx {
            self.reserved[target_idx] = Some((r as u16, c as u16));
            self.states[idx] = ResolveState::Resolved(true);
            return true;
        } 
        
        if self.reserved[target_idx].is_some() {
            self.states[idx] = ResolveState::Resolved(false);
            return false;
        } 
        
        self.reserved[target_idx] = Some((r as u16, c as u16));

        if self.states[target_idx] == ResolveState::Empty {
            self.states[idx] = ResolveState::Resolved(true);
            return true;
        }

        // Recursion
        let occupant_can_move = self.resolve_move(tr, tc, w);
        
        // Check if occupant actually vacates
        let (occ_tr, occ_tc) = self.intentions[target_idx];
        let occupant_vacating = occupant_can_move && ((occ_tr as usize * w + occ_tc as usize) != target_idx);

        if occupant_vacating {
            self.states[idx] = ResolveState::Resolved(true);
            true
        } else {
            self.states[idx] = ResolveState::Resolved(false);
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
        let neighborhood_mvt = Neighborhood::new(3, 3, 1, 1, mask);
        let cell = &grid.cells[1][1];
        let mv = Movements::static_movement(cell, &neighborhood_mvt, &grid);
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
        let neighborhood_mvt = Neighborhood::new(3, 3, 1, 1, mask);
        let cell = &grid.cells[1][1];
        for _ in 0..10 {
            let mv = Movements::random_movement(cell, &neighborhood_mvt, &grid);
            let dr = mv.0 + neighborhood_mvt.center_row as isize;
            let dc = mv.1 + neighborhood_mvt.center_col as isize;
            assert!(dr >= 0 && dr < 3 && dc >= 0 && dc < 3, "Random movement must stay within neighborhood_mvt");
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
        let neighborhood_mvt = Neighborhood::new(3, 3, 1, 1, mask);
        let cell = &grid.cells[1][1];
        let mv = Movements::gradient(cell, &neighborhood_mvt, &grid);
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
        let neighborhood_mvt = Neighborhood::new(3, 3, 1, 1, mask);
        let cell = &grid.cells[1][1];
        let mv = Movements::avoid_crowding(cell, &neighborhood_mvt, &grid);
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
        let neighborhood_mvt = Neighborhood::new(3, 3, 1, 1, mask);
        let cell = &grid.cells[1][1];
        let mv = Movements::chemotaxis(cell, &neighborhood_mvt, &grid);
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
        let neighborhood_mvt = Neighborhood::new(3, 3, 1, 1, mask);
        let registry = MovementRegistry::default();
        let new_cells = registry.apply_movement(&neighborhood_mvt, &grid);
        // All cells should remain in place
        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(new_cells[r][c].position, (r, c));
            }
        }
    }
}