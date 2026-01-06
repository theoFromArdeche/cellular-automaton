use crate::grid::Grid;
use crate::neighborhood::Neighborhood;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use rand::prelude::*;



pub struct Movements;

impl Movements {
    /// No movement - cells stay in place
    #[inline(always)]
    pub fn static_movement(_cell_r: usize, _cell_c: usize, _neighborhood_mvt: &Neighborhood, _grid: &Grid) -> (isize, isize) {
        (0, 0)
    }

    /// Random walk - move randomly to any valid position in the neighborhood mask
    pub fn random_movement(_cell_r: usize, _cell_c: usize, neighborhood_mvt: &Neighborhood, _grid: &Grid) -> (isize, isize) {
        let mut valid_moves = [(0isize, 0isize); 9]; // Max 9 positions in 3x3
        let mut count = 0;
        
        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for mask_r in 0..neighborhood_mvt.height {
            for mask_c in 0..neighborhood_mvt.width {
                if neighborhood_mvt.is_valid(mask_r, mask_c) == 1 {                    
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
    pub fn gradient(cell_r: usize, cell_c: usize, neighborhood_mvt: &Neighborhood, grid: &Grid) -> (isize, isize) {
        let current_val = grid.get_cell_trait(cell_r, cell_c, 0);
        let mut best_val = current_val;
        let mut best_move = (0, 0);

        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for mask_r in 0..neighborhood_mvt.height {
            for mask_c in 0..neighborhood_mvt.width {
                if neighborhood_mvt.is_valid(mask_r, mask_c) == 1 &&
                   !(mask_r == center_row && mask_c == center_col) {

                    let (grid_r, grid_c) = neighborhood_mvt.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_trait(grid_r, grid_c, 0);
                
                    if neighbor_is_empty == 0 && neighbor_value > best_val {
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
    pub fn avoid_crowding(cell_r: usize, cell_c: usize, neighborhood_mvt: &Neighborhood, grid: &Grid) -> (isize, isize) {
        let mut sum = 0.0;
        let mut count = 0;

        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for mask_r in 0..neighborhood_mvt.height {
            for mask_c in 0..neighborhood_mvt.width {
                if neighborhood_mvt.is_valid(mask_r, mask_c) == 1 &&
                   !(mask_r == center_row && mask_c == center_col) {

                    let (grid_r, grid_c) = neighborhood_mvt.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_trait(grid_r, grid_c, 0);
                
                    if neighbor_is_empty == 0 {
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
                    if neighborhood_mvt.is_valid(mask_r, mask_c) == 1 {                    
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
    pub fn chemotaxis(cell_r: usize, cell_c: usize, neighborhood_mvt: &Neighborhood, grid: &Grid) -> (isize, isize) {
        let target_cell_trait = 2; // Looking for cooperation trait
        let current_val = grid.get_cell_trait(cell_r, cell_c, target_cell_trait);
        
        let mut best_val = current_val;
        let mut best_move = (0, 0);

        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;

        for mask_r in 0..neighborhood_mvt.height {
            for mask_c in 0..neighborhood_mvt.width {
                if neighborhood_mvt.is_valid(mask_r, mask_c) == 1 &&
                   !(mask_r == center_row && mask_c == center_col) {

                    let (grid_r, grid_c) = neighborhood_mvt.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_trait(grid_r, grid_c, target_cell_trait);
                
                    if neighbor_is_empty == 0 && neighbor_value > best_val {                
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
    pub fn levy_flight(_cell_r: usize, _cell_c: usize, neighborhood_mvt: &Neighborhood, _grid: &Grid) -> (isize, isize) {
        let mut rng = rand::thread_rng();
        
        // 90% local movement, 10% long jump
        if rng.gen_bool(0.9) {
            // Local movement
            Movements::random_movement(_cell_r, _cell_c, neighborhood_mvt, _grid)
        } else {
            // Long jump - use full neighborhood extent
            let mut valid_moves = [(0isize, 0isize); 9];
            let mut count = 0;
            
            let center_row = neighborhood_mvt.center_row;
            let center_col = neighborhood_mvt.center_col;
            
            for mask_r in 0..neighborhood_mvt.height {
                for mask_c in 0..neighborhood_mvt.width {
                    if neighborhood_mvt.is_valid(mask_r, mask_c) == 1 {
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
    pub fn multi_trait(cell_r: usize, cell_c: usize, neighborhood_mvt: &Neighborhood, grid: &Grid) -> (isize, isize) {
        let energy = grid.get_cell_trait(cell_r, cell_c, 0);
        let mobility = grid.get_cell_trait(cell_r, cell_c, 5);
        let aggression = grid.get_cell_trait(cell_r, cell_c, 3);
        
        // High mobility = more likely to move
        let mut rng = rand::thread_rng();
        if mobility < 0.3 {
            return (0, 0); // Low mobility = stay put
        }
        
        // High energy + high aggression = seek highest value neighbor
        if energy > 0.6 && aggression > 0.6 {
            return Movements::gradient(cell_r, cell_c, neighborhood_mvt, grid);
        }
        
        // Low energy = avoid crowding
        if energy < 0.3 {
            return Movements::avoid_crowding(cell_r, cell_c, neighborhood_mvt, grid);
        }
        
        // Default: weighted random based on mobility
        if rng.gen_bool(mobility as f64) {
            Movements::random_movement(cell_r, cell_c, neighborhood_mvt, grid)
        } else {
            (0, 0)
        }
    }

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
                    if grid.is_cell_empty(grid_r, grid_c) == 0 {
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
}





#[derive(Clone, Copy, PartialEq)]
enum ResolveState {
    Unvisited,
    Empty,
    Visited,
}

pub type MovementFn = fn(usize, usize, &Neighborhood, &Grid) -> (isize, isize);

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
    (Movements::social_movement, "social movement"),
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
                          next_grid: &mut Grid, // normal Grid
                          grid: &mut Grid, // temp next_grid from previous step (apply rule)
                          ) {

        if self.get_movement_name() == self.get_name_for_movement(Movements::static_movement) {
            // Swap buffers
            next_grid.update_grid(grid);
            return;
        }

        let height = grid.height;
        let width = grid.width;
        let len = width * height;

        // 0. Reset workspace
        self.prepare(width, height);

        // Batch sizing
        let rows_per_batch = std::cmp::max(1, 4000 / width);
        let chunk_len = rows_per_batch * width;

        // --- Phase 1: Parallel Bidding ---
        self.intentions
            .par_chunks_mut(chunk_len)
            .zip(next_grid.is_empty.par_chunks(chunk_len)) // the is_empty is never changed on the temp grid (here "grid")
            .enumerate()
            .for_each(|(batch_idx, (intent_chunk, empty_chunk))| {
                let mut rng = rand::thread_rng();
                let start_idx = batch_idx * chunk_len;

                for i in 0..intent_chunk.len() {
                    let global_idx = start_idx + i;
                    if global_idx >= len {
                        break;
                    }

                    let r = global_idx / width;
                    let c = global_idx % width;

                    // skip empty cells
                    if empty_chunk[i] == 1 {
                        continue;
                    }

                    // Movement logic (row/col-based)
                    let (dr, dc) = (self.movement_function)(r, c, neighborhood_mvt, grid);

                    let (tr, tc) = (
                        ((r as isize + dr).clamp(0, height as isize - 1)) as usize,
                        ((c as isize + dc).clamp(0, width as isize - 1)) as usize,
                    );
                    intent_chunk[i] = (tr as u16, tc as u16);

                    if (tr, tc) != (r, c) { // (tr, tc) == (r, c) is not in bid because its managed in step 3 (it always has priority)
                        let target_flat = tr * width + tc;
                        let priority: u32 = rng.next_u32();
                        let bid = ((priority as u64) << 32) | (global_idx as u64);

                        unsafe {
                            self.claims
                                .get_unchecked(target_flat)
                                .fetch_max(bid, Ordering::Relaxed);
                        }
                    }
                }
            });

        // --- Phase 2: Pruning ---
        let prune_chunk_size = width * 50;
        self.intentions
            .par_chunks_mut(prune_chunk_size)
            .enumerate()
            .for_each(|(chunk_id, chunk)| {
                let base_idx = chunk_id * prune_chunk_size;

                for i in 0..chunk.len() {
                    let global_idx = base_idx + i;
                    let (tr, tc) = chunk[i];
                    let r = global_idx / width;
                    let c = global_idx % width;

                    if (tr as usize, tc as usize) == (r, c) {
                        continue;
                    }

                    let target_flat = tr as usize * width + tc as usize;
                    let winning_bid = unsafe {
                        self.claims
                            .get_unchecked(target_flat)
                            .load(Ordering::Relaxed)
                    };
                    let winner_idx = (winning_bid & 0xFFFFFFFF) as usize;

                    if winner_idx != global_idx {
                        chunk[i] = (r as u16, c as u16);
                    }
                }
            });

        // --- Phase 3: Resolve (DFS) ---
        for r in 0..height {
            for c in 0..width {
                let idx = r * width + c;

                // We don't want to visit empty cells
                if next_grid.is_empty[idx] == 1 {
                    self.states[idx] = ResolveState::Empty;
                    continue;
                }

                let (tr_u16, tc_u16) = self.intentions[idx];
                let tr = tr_u16 as usize;
                let tc = tc_u16 as usize;
                let target_idx = tr * width + tc;

                //print!("({},{}), ", tr, tc);

                // cells that stays have the priority
                if target_idx == idx {
                    self.reserved[idx] = Some((r as u16, c as u16));
                    self.states[idx] = ResolveState::Visited;
                }
            }
            //println!("");
        }
        //println!("");

        for r in 0..height {
            for c in 0..width {
                let idx = r * width + c;
                if self.states[idx] == ResolveState::Unvisited {
                    self.resolve_move(r, c, width);
                }
            }
        }

        // --- Phase 4: Construct next grid ---
        next_grid
            .is_empty
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, out_empty)| {
                // check if reserved
                *out_empty = match self.reserved[idx] {
                    Some(_) => 0,
                    None => 1, // the cell will be empty
                };
            });

        // Update all traits in parallel (one pass per trait)
        next_grid
            .traits
            .par_iter_mut()
            .enumerate()
            .for_each(|(trait_idx, out_trait_vec)| {
                out_trait_vec
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(idx, out_trait_val)| {
                        // check if reserved or not
                        *out_trait_val = match self.reserved[idx] {
                            Some((sr, sc)) => {
                                let src_idx = sr as usize * width + sc as usize;
                                grid.traits[trait_idx][src_idx]
                            }
                            None => 0.0, // the cell will be empty
                        };
                    });
            });
    }


    // Optimized DFS for flat buffers
    pub fn resolve_move(&mut self, r: usize, c: usize, w: usize) {
        let idx = r * w + c;

        // loop 
        // (the only possible loop is a circle (end is start, start is end)
        // (because a cell has a maximum of 1 individual (that is not already in the cell) that want this cell)
        if self.states[idx] == ResolveState::Visited {
            return
        }

        self.states[idx] = ResolveState::Visited;

        let (tr_u16, tc_u16) = self.intentions[idx];
        let tr = tr_u16 as usize;
        let tc = tc_u16 as usize;
        let target_idx = tr * w + tc;

        //println!("intention ({}, {}) -> ({}, {})", r, c, tr, tc);

        if self.reserved[target_idx].is_some() {
            // we can't go because its already reserved so we stay in place
            // (because of the tie breaker of phase 2, the target is reserved only if the individual inside it does not move,
            // ties in resolve_move are only between an individual that want to move and an individual that want to stay)
            self.reserved[idx] = Some((r as u16, c as u16));
            //println!("reserved ({}, {}) -> ({}, {})", r, c, r, c);
            return;
        }

        if self.states[target_idx] == ResolveState::Empty {
            // we can go
            self.reserved[target_idx] = Some((r as u16, c as u16));
            //println!("empty ({}, {}) -> ({}, {})", r, c, tr, tc);
            return;
        }

        // the target cell has an individual inside it
        // so we need to check if the individual will move or not

        // propagate
        self.resolve_move(tr, tc, w); 
        // (tr, tc) != (r, c) because cells that want to stay as their first wish because resolve_move
        // are set to Visited earlier in step 3

        if self.reserved[target_idx].is_some() {
            // we can't go
            self.reserved[idx] = Some((r as u16, c as u16));
            //println!("staying ({}, {}) -> ({}, {})", r, c, r, c);
        } else {
            // we can go
            self.reserved[target_idx] = Some((r as u16, c as u16));
            //println!("going to target ({}, {}) -> ({}, {})", r, c, tr, tc);
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
                    grid.cells[r][c].set_cell_trait(index, 0.5);
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
        grid.cells[0][1].set_cell_trait(0, 0.9);
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
    fn test_chemotaxis_moves_toward_target_cell_trait() {
        let mut grid = build_test_grid();
        grid.cells[0][0].set_cell_trait(2, 1.0); // Cooperation trait
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