use crate::grid::Grid;
use crate::neighborhood::Neighborhood;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use rand::prelude::*;


pub struct MovementFunction;

impl MovementFunction {
    /// No movement - cells stay in place
    #[inline(always)]
    pub fn static_movement(_cell_r: usize, _cell_c: usize, _neighborhood_mvt: &Neighborhood, _grid: &Grid) -> (isize, isize) {
        (0, 0)
    }

    /// Random walk - move randomly to any valid position in the neighborhood mask
    pub fn random_movement(_cell_r: usize, _cell_c: usize, neighborhood_mvt: &Neighborhood, _grid: &Grid) -> (isize, isize) {
        let mut valid_moves = Vec::with_capacity(neighborhood_mvt.height * neighborhood_mvt.width);
        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;
        
        for mask_r in 0..neighborhood_mvt.height {
            for mask_c in 0..neighborhood_mvt.width {
                if neighborhood_mvt.is_valid(mask_r, mask_c) == 1 {                    
                    valid_moves.push((
                        mask_r as isize - center_row as isize,
                        mask_c as isize - center_col as isize
                    ));
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

    /// Move toward the neighbor with the highest trait value (gradient ascent)
    /// If multiple neighbors have the same highest value, randomly choose one
    pub fn gradient(cell_r: usize, cell_c: usize, neighborhood_mvt: &Neighborhood, grid: &Grid) -> (isize, isize) {
        let current_val = grid.get_cell_trait(cell_r, cell_c, 0);
        let mut best_val = current_val;
        let mut best_moves = Vec::new();
        let center_row = neighborhood_mvt.center_row;
        let center_col = neighborhood_mvt.center_col;
        
        for mask_r in 0..neighborhood_mvt.height {
            for mask_c in 0..neighborhood_mvt.width {
                if neighborhood_mvt.is_valid(mask_r, mask_c) == 1 &&
                    !(mask_r == center_row && mask_c == center_col) {
                    let (grid_r, grid_c) = neighborhood_mvt.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_trait(grid_r, grid_c, 0);
                    
                    if !neighbor_is_empty {
                        if neighbor_value > best_val {
                            // Found a new best value - reset the list
                            best_val = neighbor_value;
                            best_moves.clear();
                            best_moves.push((
                                mask_r as isize - center_row as isize,
                                mask_c as isize - center_col as isize,
                            ));
                        } else if neighbor_value == best_val {
                            // Found another move with the same best value
                            best_moves.push((
                                mask_r as isize - center_row as isize,
                                mask_c as isize - center_col as isize,
                            ));
                        }
                    }
                }
            }
        }
        
        if best_moves.is_empty() {
            return (0, 0);
        }
        
        // Randomly choose one of the best moves
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..best_moves.len());
        best_moves[idx]
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
            let mut valid_moves = Vec::with_capacity(neighborhood_mvt.height * neighborhood_mvt.width);
            
            for mask_r in 0..neighborhood_mvt.height {
                for mask_c in 0..neighborhood_mvt.width {
                    if neighborhood_mvt.is_valid(mask_r, mask_c) == 1 {                    
                        valid_moves.push((
                            mask_r as isize - center_row as isize,
                            mask_c as isize - center_col as isize
                        ));
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

    pub fn energy_charge_phase(cell_r: usize, cell_c: usize, neighborhood: &Neighborhood, grid: &Grid) -> (isize, isize) {
        let energy = grid.get_cell_trait(cell_r, cell_c, 0);
        let phase = grid.get_cell_trait(cell_r, cell_c, 2);
        let charge = grid.get_cell_trait(cell_r, cell_c, 1);
        
        // Movement gated by phase (creates pulses)
        // AND minimum energy to move
        if phase < 0.4 || phase > 0.8 || energy < 0.2 {
            return (0, 0);
        }
        
        let center_row = neighborhood.center_row;
        let center_col = neighborhood.center_col;
        
        let mut move_dr: f32 = 0.0;
        let mut move_dc: f32 = 0.0;
        
        for mask_r in 0..neighborhood.height {
            for mask_c in 0..neighborhood.width {
                if neighborhood.is_valid(mask_r, mask_c) == 1 
                && !(mask_r == center_row && mask_c == center_col) {
                    let (grid_r, grid_c) = neighborhood.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    if !grid.is_cell_empty(grid_r, grid_c) {
                        let neighbor_energy = grid.get_cell_trait(grid_r, grid_c, 0);
                        let neighbor_charge = grid.get_cell_trait(grid_r, grid_c, 1);
                        
                        let dr = mask_r as f32 - center_row as f32;
                        let dc = mask_c as f32 - center_col as f32;
                        
                        // Attract to similar charge with high energy
                        // Repel from opposite charge
                        let charge_diff = (charge - neighbor_charge).abs();
                        let attraction = (0.5 - charge_diff) * neighbor_energy;
                        
                        move_dr += dr * attraction;
                        move_dc += dc * attraction;
                    }
                }
            }
        }
        
        let mut rng = rand::thread_rng();
        
        if move_dr.abs() > 0.05 || move_dc.abs() > 0.05 {
            let dr = if move_dr.abs() < 0.05 { 0 } else { move_dr.signum() as isize };
            let dc = if move_dc.abs() < 0.05 { 0 } else { move_dc.signum() as isize };
            
            if dr != 0 && dc != 0 && rng.gen_bool(0.5) {
                if rng.gen_bool(0.5) { (dr, 0) } else { (0, dc) }
            } else {
                (dr, dc)
            }
        } else {
            // Weak random drift
            if rng.gen_bool(0.3) {
                let moves = [(0,1), (0,-1), (1,0), (-1,0)];
                moves[rng.gen_range(0..moves.len())]
            } else {
                (0, 0)
            }
        }
    }
}



macro_rules! define_movements {
    ($(($variant:ident, $name:expr, $func:path)),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Movement {
            $($variant),*
        }
        
        impl Movement {
            pub const ALL: &'static [Movement] = &[$(Movement::$variant),*];
            pub const NAMES: &'static [&'static str] = &[$($name),*];
            
            #[inline]
            pub fn name(&self) -> &'static str {
                match self {
                    $(Movement::$variant => $name),*
                }
            }
            
            #[inline]
            pub fn from_name(name: &str) -> Option<Movement> {
                match name {
                    $($name => Some(Movement::$variant)),*,
                    _ => None,
                }
            }
            
            #[inline]
            pub fn get_fn(&self) -> MovementFnType {
                match self {
                    $(Movement::$variant => $func),*
                }
            }
        }
    };
}

// ============================================================
// ADD NEW MOVEMENTS HERE - Just add one line!
// Format: (EnumVariant, "display name", MovementFunction::function_name)
// ============================================================
define_movements!(
    (Static,            "static",              MovementFunction::static_movement),
    (Random,            "random",              MovementFunction::random_movement),
    (Gradient,          "gradient",            MovementFunction::gradient),
    (AvoidCrowding,     "avoid crowding",      MovementFunction::avoid_crowding),
    (EnergyChargePhase, "energy charge phase", MovementFunction::energy_charge_phase),
    // Add new movements here:
);

#[derive(Clone, Copy, PartialEq)]
enum ResolveState {
    Unvisited,
    Empty,
    Visited,
}

pub type MovementFnType = fn(usize, usize, &Neighborhood, &Grid) -> (isize, isize);

pub struct MovementRegistry {
    pub movement_function: MovementFnType,
    movement: Movement,
    // Stores bids: High 32 bits = Priority, Low 32 bits = Source Index
    claims: Vec<AtomicU64>,
    // Stores target (r, c) for every cell. Flattened index = r * width + c
    intentions: Vec<(u16, u16)>,
    // DFS helper: Flattened
    reserved: Vec<Option<(u16, u16)>>,
    // DFS helper: Flattened
    states: Vec<ResolveState>,
}

impl MovementRegistry {
    pub fn new(width: usize, height: usize) -> Self {
        Self::custom(width, height, Movement::Static)
    }
    
    pub fn custom(width: usize, height: usize, movement: Movement) -> Self {
        let size = width * height;
        Self {
            movement_function: movement.get_fn(),
            movement,
            claims: (0..size).map(|_| AtomicU64::new(0)).collect(),
            intentions: vec![(0, 0); size],
            reserved: vec![None; size],
            states: vec![ResolveState::Unvisited; size],
        }
    }
    
    // Fast reset without deallocating
    pub fn prepare(&mut self, width: usize, height: usize) {
        let size = width * height;
        
        if self.claims.len() != size {
            self.claims = (0..size).map(|_| AtomicU64::new(0)).collect();
            self.intentions.resize(size, (0, 0));
            self.reserved.resize(size, None);
            self.states.resize(size, ResolveState::Unvisited);
        } else {
            self.claims.par_iter().for_each(|x| x.store(0, Ordering::Relaxed));
            self.reserved.par_iter_mut().for_each(|x| *x = None);
            self.states.par_iter_mut().for_each(|x| *x = ResolveState::Unvisited);
        }
    }
    
    pub fn set_movement(&mut self, movement: Movement) {
        self.movement_function = movement.get_fn();
        self.movement = movement;
    }
    
    #[inline]
    pub fn get_movement_name(&self) -> &'static str {
        self.movement.name()
    }
    
    #[inline]
    pub fn get_movement(&self) -> Movement {
        self.movement
    }
    
    #[inline]
    pub fn is_stored_movement(&self, movement: Movement) -> bool {
        self.movement == movement
    }
    
    /// Get movement type by name
    #[inline]
    pub fn get_movement_by_name(name: &str) -> Option<Movement> {
        Movement::from_name(name)
    }

    #[inline]
    pub fn get_name_for_movement(movement: Movement) -> &'static str {
        movement.name()
    }
    
    /// Get all available movement names
    #[inline]
    pub fn get_all_names() -> &'static [&'static str] {
        Movement::NAMES
    }
    
    /// Get all available movement types
    #[inline]
    pub fn get_all_movements() -> &'static [Movement] {
        Movement::ALL
    }

    pub fn apply_movement(&mut self,
                          neighborhood_mvt: &Neighborhood,
                          next_grid: &mut Grid, // normal Grid
                          grid: &mut Grid, // temp next_grid from previous step (apply rule)
                          ) {

        if self.movement == Movement::Static {
            // Swap buffers
            std::mem::swap(&mut grid.data, &mut next_grid.data);
            // the is_empty is never changed on the temp grid (here "grid"), the correct values are always in the normal grid (here "next_grid")
            return;
        }

        let height = grid.height;
        let width = grid.width;
        let len = width * height;

        // Reset workspace
        self.prepare(width, height);

        let rows_per_batch = std::cmp::max(1, 4000 / width);
        let chunk_len = rows_per_batch * width;

        // --- Phase 1: Parallel Bidding ---
        self.intentions
            .par_chunks_mut(chunk_len)
            .enumerate()
            .for_each(|(batch_idx, intent_chunk)| {
                let mut rng = rand::thread_rng();
                let start_idx = batch_idx * chunk_len;
                
                for i in 0..intent_chunk.len() {
                    let global_idx = start_idx + i;
                    if global_idx >= len {
                        break;
                    }
                    
                    let r = global_idx / width;
                    let c = global_idx % width;
                    
                    // Skip empty cells - BitVec: true = empty
                    if next_grid.is_empty[global_idx] { // the is_empty is never changed on the temp grid (here "grid"), the correct values are always in the normal grid (here "next_grid")
                        continue;
                    }
                    
                    // Movement logic
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
                if next_grid.is_empty[idx] {
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
        next_grid.is_empty
            .par_iter_mut()
            .zip(self.reserved.par_iter())
            .for_each(|(empty, reserved)| {
                *empty = reserved.is_none();
            });

        let width = grid.width;

        for trait_idx in 0..grid.num_traits {
            let current = grid.get_trait_slice(trait_idx);
            let out_trait = next_grid.get_trait_slice_mut(trait_idx);
            
            out_trait
                .par_iter_mut()
                .enumerate()
                .for_each(|(idx, out_trait_val)| {
                    *out_trait_val = match self.reserved[idx] {
                        Some((sr, sc)) => {
                            let src_idx = sr as usize * width + sc as usize;
                            current[src_idx]
                        }
                        None => 0.0,
                    };
                });
        }
    }


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
                for index in 0..grid.num_traits {
                    grid.set_cell_trait(r, c, index, 0.5);
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
        let mv = MovementFunction::static_movement(cell, &neighborhood_mvt, &grid);
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
            let mv = MovementFunction::random_movement(cell, &neighborhood_mvt, &grid);
            let dr = mv.0 + neighborhood_mvt.center_row as isize;
            let dc = mv.1 + neighborhood_mvt.center_col as isize;
            assert!(dr >= 0 && dr < 3 && dc >= 0 && dc < 3, "Random movement must stay within neighborhood_mvt");
        }
    }

    #[test]
    fn test_gradient_moves_toward_highest_trait() {
        let mut grid = build_test_grid();
        // Set a neighbor with higher trait
        grid.set_cell_trait(0, 1, 0, 0.9);
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let neighborhood_mvt = Neighborhood::new(3, 3, 1, 1, mask);
        let cell = &grid.cells[1][1];
        let mv = MovementFunction::gradient(cell, &neighborhood_mvt, &grid);
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
        let mv = MovementFunction::avoid_crowding(cell, &neighborhood_mvt, &grid);
        assert_eq!(mv, (0, 0), "Avoid crowding should stay put if density is low");
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