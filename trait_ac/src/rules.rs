use crate::neighborhood::Neighborhood;
use crate::grid::Grid;



pub struct Rules;

impl Rules {
    /// No change - cells maintain their trait value
    #[inline(always)]
    pub fn static_rule(trait_index: usize, cell_r: usize, cell_c: usize, _neighborhood_traits: &Neighborhood, grid: &Grid) -> f32 {
        grid.get_cell_trait(cell_r, cell_c, trait_index)
    }

    /// Average of neighbors' trait values
    pub fn average(trait_index: usize, cell_r: usize, cell_c: usize, neighborhood_traits: &Neighborhood, grid: &Grid) -> f32 {
        let mut sum = 0.0;
        let mut count = 0;

        for mask_r in 0..neighborhood_traits.height {
            for mask_c in 0..neighborhood_traits.width {
                if neighborhood_traits.is_valid(mask_r, mask_c) == 1 {

                    let (grid_r, grid_c) = neighborhood_traits.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);

                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_trait(grid_r, grid_c, trait_index);
                    
                    if !neighbor_is_empty {
                        sum += neighbor_value;
                        count += 1;
                    }
                }
            }
        }

        if count == 0 {
            grid.get_cell_trait(cell_r, cell_c, trait_index)
        } else {
            (sum / count as f32).clamp(0.0, 1.0)
        }
    }

    /// Conway's Game of Life style
    pub fn conway(trait_index: usize, cell_r: usize, cell_c: usize, neighborhood_traits: &Neighborhood, grid: &Grid) -> f32 {
        let mut alive_neighbors = 0;

        let center_row = neighborhood_traits.center_row;
        let center_col = neighborhood_traits.center_col;

        for mask_r in 0..neighborhood_traits.height {
            for mask_c in 0..neighborhood_traits.width {
                if neighborhood_traits.is_valid(mask_r, mask_c) == 1 &&
                   !(mask_r == center_row && mask_c == center_col) {

                    let (grid_r, grid_c) = neighborhood_traits.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_trait(grid_r, grid_c, trait_index);

                    if !neighbor_is_empty && neighbor_value > 0.5 {
                        alive_neighbors += 1;
                    }
                }
            }
        }

        let current = grid.get_cell_trait(cell_r, cell_c, trait_index);

        if current > 0.5 {
            if alive_neighbors == 2 || alive_neighbors == 3 { 1.0 } else { 0.0 }
        } else {
            if alive_neighbors == 3 { 1.0 } else { 0.0 }
        }
    }

    #[inline(always)]
    pub fn conway_optimized(
        trait_index: usize,
        cell_r: usize,
        cell_c: usize,
        _neighborhood: &Neighborhood,
        grid: &Grid,
    ) -> f32 {
        let w = grid.width;
        let h = grid.height;
        
        let r_prev = cell_r.wrapping_sub(1) % h;
        let r_next = (cell_r + 1) % h;
        let c_prev = cell_c.wrapping_sub(1) % w;
        let c_next = (cell_c + 1) % w;
        
        let row_prev = r_prev * w;
        let row_curr = cell_r * w;
        let row_next = r_next * w;
        
        let t = grid.get_trait_slice(trait_index);
        
        unsafe {
            // Count alive: cast (value > 0.5) directly to u8
            let alive = 
                (*t.get_unchecked(row_prev + c_prev) > 0.5) as u8 +
                (*t.get_unchecked(row_prev + cell_c) > 0.5) as u8 +
                (*t.get_unchecked(row_prev + c_next) > 0.5) as u8 +
                (*t.get_unchecked(row_curr + c_prev) > 0.5) as u8 +
                (*t.get_unchecked(row_curr + c_next) > 0.5) as u8 +
                (*t.get_unchecked(row_next + c_prev) > 0.5) as u8 +
                (*t.get_unchecked(row_next + cell_c) > 0.5) as u8 +
                (*t.get_unchecked(row_next + c_next) > 0.5) as u8;
            
            let is_alive = *t.get_unchecked(row_curr + cell_c) > 0.5;
            
            // Combined branchless lookup: alive + 9 * is_alive indexes into single array
            const RESULT: [f32; 18] = [
                // Dead cell (indices 0-8)
                0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                // Alive cell (indices 9-17)
                0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ];
            
            *RESULT.get_unchecked(alive as usize + 9 * is_alive as usize)
        }
    }

    /// Diffusion with decay
    pub fn diffusion(trait_index: usize, cell_r: usize, cell_c: usize, neighborhood_traits: &Neighborhood, grid: &Grid) -> f32 {
        let mut sum = 0.0;
        let mut count = 0;

        let center_row = neighborhood_traits.center_row;
        let center_col = neighborhood_traits.center_col;

        for mask_r in 0..neighborhood_traits.height {
            for mask_c in 0..neighborhood_traits.width {
                if neighborhood_traits.is_valid(mask_r, mask_c) == 1 &&
                   !(mask_r == center_row && mask_c == center_col) {

                    let (grid_r, grid_c) = neighborhood_traits.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_trait(grid_r, grid_c, trait_index);

                    if !neighbor_is_empty {
                        sum += neighbor_value;
                        count += 1;
                    }
                }
            }
        }

        if count == 0 {
            return (grid.get_cell_trait(cell_r, cell_c, trait_index) * 0.95).clamp(0.0, 1.0);
        }

        let current = grid.get_cell_trait(cell_r, cell_c, trait_index);
        let avg = sum / count as f32;

        ((0.3 * current + 0.7 * avg) * 0.98).clamp(0.0, 1.0)
    }

    /// Maximum of neighbors
    pub fn maximum(trait_index: usize, cell_r: usize, cell_c: usize, neighborhood_traits: &Neighborhood, grid: &Grid) -> f32 {
        let mut max_val = grid.get_cell_trait(cell_r, cell_c, trait_index);

        for mask_r in 0..neighborhood_traits.height {
            for mask_c in 0..neighborhood_traits.width {
                if neighborhood_traits.is_valid(mask_r, mask_c) == 1 {

                    let (grid_r, grid_c) = neighborhood_traits.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_trait(grid_r, grid_c, trait_index);

                    if !neighbor_is_empty {
                        max_val = max_val.max(neighbor_value);
                    }
                }
            }
        }

        max_val
    }

    /// Minimum of neighbors
    pub fn minimum(trait_index: usize, cell_r: usize, cell_c: usize, neighborhood_traits: &Neighborhood, grid: &Grid) -> f32 {
        let mut min_val = grid.get_cell_trait(cell_r, cell_c, trait_index);

        for mask_r in 0..neighborhood_traits.height {
            for mask_c in 0..neighborhood_traits.width {
                if neighborhood_traits.is_valid(mask_r, mask_c) == 1 {

                    let (grid_r, grid_c) = neighborhood_traits.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_trait(grid_r, grid_c, trait_index);

                    if !neighbor_is_empty {
                        min_val = min_val.min(neighbor_value);
                    }
                }
            }
        }

        min_val
    }

    /// Weighted average by distance
    pub fn weighted_average(trait_index: usize, cell_r: usize, cell_c: usize, neighborhood_traits: &Neighborhood, grid: &Grid) -> f32 {
        let mut sum = 0.0;
        let mut weight_sum = 0.0;

        let center_row = neighborhood_traits.center_row;
        let center_col = neighborhood_traits.center_col;

        for mask_r in 0..neighborhood_traits.height {
            for mask_c in 0..neighborhood_traits.width {
                if neighborhood_traits.is_valid(mask_r, mask_c) == 1 &&
                   !(mask_r == center_row && mask_c == center_col) {

                    let (grid_r, grid_c) = neighborhood_traits.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    let neighbor_is_empty = grid.is_cell_empty(grid_r, grid_c);
                    let neighbor_value = grid.get_cell_trait(grid_r, grid_c, trait_index);

                    if !neighbor_is_empty {
                        let dr = (mask_r as isize - center_row as isize).abs() as f32;
                        let dc = (mask_c as isize - center_col as isize).abs() as f32;
                        let w = 1.0 / (1.0 + (dr * dr + dc * dc).sqrt());

                        sum += neighbor_value * w;
                        weight_sum += w;
                    }
                }
            }
        }

        if weight_sum == 0.0 {
            grid.get_cell_trait(cell_r, cell_c, trait_index)
        } else {
            (sum / weight_sum).clamp(0.0, 1.0)
        }
    }

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

    /// Individuals become more like their neighbors over time
    pub fn social_influence(_trait_index: usize, cell_r: usize, cell_c: usize, neighborhood: &Neighborhood, grid: &Grid) -> f32 {
        let my_social = grid.get_cell_trait(cell_r, cell_c, 1);
        
        let center_row = neighborhood.center_row;
        let center_col = neighborhood.center_col;
        
        let mut neighbor_social_sum = 0.0;
        let mut neighbor_count = 0;
        
        for mask_r in 0..neighborhood.height {
            for mask_c in 0..neighborhood.width {
                if neighborhood.is_valid(mask_r, mask_c) == 1 
                && !(mask_r == center_row && mask_c == center_col) {
                    let (grid_r, grid_c) = neighborhood.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    if !grid.is_cell_empty(grid_r, grid_c) {
                        neighbor_social_sum += grid.get_cell_trait(grid_r, grid_c, 1);
                        neighbor_count += 1;
                    }
                }
            }
        }
        
        if neighbor_count == 0 {
            // Alone: drift toward being a loner
            (my_social * 0.90).max(0.0)
        } else {
            // With others: average slightly toward neighbors
            let avg = neighbor_social_sum / neighbor_count as f32;
            my_social * 0.80 + avg * 0.2
        }
    }
}




pub type RuleFn = fn(usize, usize, usize, &Neighborhood, &Grid) -> f32;

#[derive(Clone, Copy)]
pub struct RulesRegistry {
    rules: [RuleFn; 9],
}

// Static lookup table for function pointer to name mapping
static RULE_LOOKUP: &[(RuleFn, &str)] = &[
    (Rules::static_rule, "static"),
    (Rules::average, "average"),
    (Rules::conway, "conway"),
    (Rules::conway_optimized, "conway optimized"),
    (Rules::diffusion, "diffusion"),
    (Rules::maximum, "maximum"),
    (Rules::minimum, "minimum"),
    (Rules::weighted_average, "weighted_average"),
    (Rules::social_energy, "social energy"),
    (Rules::social_influence, "social influence"),
];

const RULE_COUNT: usize = RULE_LOOKUP.len();

const fn extract_names<'a, const N: usize>(lookup: &'a [(RuleFn, &'a str)]) -> [&'a str; N] {
    let mut names = [""; N];
    let mut i = 0;
    while i < N {
        names[i] = lookup[i].1;
        i += 1;
    }
    names
}

// Static array of just the names, extracted from lookup table
static RULE_NAMES: [&str; RULE_COUNT] = extract_names::<RULE_COUNT>(RULE_LOOKUP);

impl RulesRegistry {
    pub fn default() -> Self {
        Self {
            rules: [Rules::average; 9],
        }
    }

    pub fn custom(rules: [RuleFn; 9]) -> Self {
        Self { rules }
    }

    /// Apply a rule by its trait index
    #[inline(always)]
    pub fn apply_rule(&self, trait_index: usize, cell_r: usize, cell_c: usize, neighborhood: &Neighborhood, grid: &Grid) -> f32 {
        let rule = unsafe { *self.rules.get_unchecked(trait_index) };
        rule(trait_index, cell_r, cell_c, neighborhood, grid)
    }

    pub fn set_rule(&mut self, trait_idx: usize, rule_fn: RuleFn) {
        self.rules[trait_idx] = rule_fn;
    }

    /// Get the name of the rule assigned to a trait index
    #[inline]
    pub fn get_rule_name(&self, trait_index: usize) -> &'static str {
        let rule_fn = unsafe { *self.rules.get_unchecked(trait_index) };
        self.get_name_for_rule(rule_fn)
    }

    pub fn is_stored_function(&self, trait_index: usize, function: RuleFn) -> bool {
        let rule_fn = unsafe { *self.rules.get_unchecked(trait_index) };
        rule_fn as usize == function as usize
    }

    /// Get the name for a specific rule function (uses lookup table)
    #[inline]
    pub fn get_name_for_rule(&self, rule_fn: RuleFn) -> &'static str {
        for &(func, name) in RULE_LOOKUP {
            if func as usize == rule_fn as usize {
                return name;
            }
        }
        "unknown"
    }

    /// Get rule function by name (uses lookup table)
    #[inline]
    pub fn get_rule_by_name(&self, rule_name: &str) -> Option<RuleFn> {
        for &(func, name) in RULE_LOOKUP {
            if name == rule_name {
                return Some(func);
            }
        }
        None
    }

    /// Get all available rule names (from lookup table)
    #[inline(always)]
    pub fn get_all_names(&self) -> &'static [&'static str; RULE_COUNT] {
        &RULE_NAMES
    }
}





#[cfg(test)]
mod tests {
    use crate::grid::Grid;
    use crate::rules::{Rules, RulesRegistry};
    use crate::neighborhood::Neighborhood;

    #[test]
    fn test_rule_average_direct() {
        let grid = Grid::new(3, 3);
        let mask = vec![vec![true; 3]; 3];
        let neighborhood = Neighborhood::new(
            3, 3,        // width, height
            1, 1,        // center row, center col for traits
            mask,       // mask
        );

        let result = Rules::average(0, 1, 1, &neighborhood, &grid);
        assert!(
            (0.0..=1.0).contains(&result),
            "Average rule should produce value between 0.0 and 1.0"
        );
    }

    #[test]
    fn test_rule_conway_direct() {
        let grid = Grid::new(3, 3);
        let mask = vec![vec![true; 3]; 3];
        let neighborhood = Neighborhood::new(
            3, 3,        // width, height
            1, 1,        // center row, center col for traits
            mask,       // mask
        );

        let result = Rules::conway(0, 1, 1, &neighborhood, &grid);
        assert!(
            result == 0.0 || result == 1.0,
            "Conway rule should produce 0.0 or 1.0"
        );
    }

    #[test]
    fn test_rules_registry_apply_default() {
        let grid = Grid::new(3, 3);
        let mask = vec![vec![true; 3]; 3];
        let neighborhood = Neighborhood::new(
            3, 3,        // width, height
            1, 1,        // center row, center col for traits
            mask,       // mask
        );

        let rules_registry = RulesRegistry::default();
        let result = rules_registry.apply_rule(0, 1, 1, &neighborhood, &grid);

        assert!(
            (0.0..=1.0).contains(&result),
            "Default rules registry should return value between 0.0 and 1.0"
        );
    }

    #[test]
    fn test_rules_registry_apply_custom() {
        let grid = Grid::new(3, 3);
        let mask = vec![vec![true; 3]; 3];
        let neighborhood = Neighborhood::new(
            3, 3,        // width, height
            1, 1,        // center row, center col for traits
            mask,       // mask
        );

        let rules_registry = RulesRegistry::custom([
            Rules::static_rule,
            Rules::conway,
            Rules::average,
            Rules::average,
            Rules::average,
            Rules::average,
            Rules::average,
            Rules::average,
            Rules::average,
        ]);

        let result = rules_registry.apply_rule(1, 1, 1, &neighborhood, &grid);
        assert!(
            result == 0.0 || result == 1.0,
            "Custom rules registry with Conway rule should produce 0.0 or 1.0"
        );
    }
}
