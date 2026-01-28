use crate::neighborhood::Neighborhood;
use crate::grid::Grid;



pub struct RuleFunction;

impl RuleFunction {
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
        
        // Branchless wrap - avoids expensive modulo
        let r_prev = if cell_r == 0 { h - 1 } else { cell_r - 1 };
        let r_next = if cell_r == h - 1 { 0 } else { cell_r + 1 };
        let c_prev = if cell_c == 0 { w - 1 } else { cell_c - 1 };
        let c_next = if cell_c == w - 1 { 0 } else { cell_c + 1 };
        
        let row_prev = r_prev * w;
        let row_curr = cell_r * w;
        let row_next = r_next * w;
        
        let t = grid.get_trait_slice(trait_index);
        
        unsafe {
            let t = t.as_ptr();
            
            let alive = 
                (*t.add(row_prev + c_prev) > 0.5) as u8 +
                (*t.add(row_prev + cell_c) > 0.5) as u8 +
                (*t.add(row_prev + c_next) > 0.5) as u8 +
                (*t.add(row_curr + c_prev) > 0.5) as u8 +
                (*t.add(row_curr + c_next) > 0.5) as u8 +
                (*t.add(row_next + c_prev) > 0.5) as u8 +
                (*t.add(row_next + cell_c) > 0.5) as u8 +
                (*t.add(row_next + c_next) > 0.5) as u8;
            
            let is_alive = *t.add(row_curr + cell_c) > 0.5;
            
            const RESULT: [f32; 18] = [
                0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
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

    pub fn energy_update(_trait_index: usize, cell_r: usize, cell_c: usize, neighborhood: &Neighborhood, grid: &Grid) -> f32 {
        let energy = grid.get_cell_trait(cell_r, cell_c, 0);
        let phase = grid.get_cell_trait(cell_r, cell_c, 2);
        
        let center_row = neighborhood.center_row;
        let center_col = neighborhood.center_col;
        
        let mut friction: f32 = 0.0;
        let mut neighbor_energy_sum: f32 = 0.0;
        let mut neighbor_count = 0;
        
        for mask_r in 0..neighborhood.height {
            for mask_c in 0..neighborhood.width {
                if neighborhood.is_valid(mask_r, mask_c) == 1 
                && !(mask_r == center_row && mask_c == center_col) {
                    let (grid_r, grid_c) = neighborhood.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    if !grid.is_cell_empty(grid_r, grid_c) {
                        neighbor_count += 1;
                        
                        let neighbor_phase = grid.get_cell_trait(grid_r, grid_c, 2);
                        let neighbor_energy = grid.get_cell_trait(grid_r, grid_c, 0);
                        
                        // Phase difference creates friction (energy generation)
                        let phase_diff = (phase - neighbor_phase).abs();
                        let cyclic_diff = phase_diff.min(1.0 - phase_diff);
                        friction += cyclic_diff;
                        
                        neighbor_energy_sum += neighbor_energy;
                    }
                }
            }
        }
        
        let mut new_energy = energy;
        
        // Small constant decay
        new_energy -= 0.02;
        
        if neighbor_count > 0 {
            // Friction generates energy (phase differences)
            let avg_friction = friction / neighbor_count as f32;
            new_energy += avg_friction * 0.15;
            
            // Diffusion: average with neighbors
            let avg_neighbor_energy = neighbor_energy_sum / neighbor_count as f32;
            new_energy = new_energy * 0.7 + avg_neighbor_energy * 0.3;
        }
        
        new_energy.clamp(0.0, 1.0)
    }

    pub fn charge_update(_trait_index: usize, cell_r: usize, cell_c: usize, neighborhood: &Neighborhood, grid: &Grid) -> f32 {
        let charge = grid.get_cell_trait(cell_r, cell_c, 1);
        let energy = grid.get_cell_trait(cell_r, cell_c, 0);
        
        let center_row = neighborhood.center_row;
        let center_col = neighborhood.center_col;
        
        let mut high_energy_charge_sum: f32 = 0.0;
        let mut high_energy_count = 0;
        let mut low_energy_charge_sum: f32 = 0.0;
        let mut low_energy_count = 0;
        
        for mask_r in 0..neighborhood.height {
            for mask_c in 0..neighborhood.width {
                if neighborhood.is_valid(mask_r, mask_c) == 1 
                && !(mask_r == center_row && mask_c == center_col) {
                    let (grid_r, grid_c) = neighborhood.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    if !grid.is_cell_empty(grid_r, grid_c) {
                        let neighbor_charge = grid.get_cell_trait(grid_r, grid_c, 1);
                        let neighbor_energy = grid.get_cell_trait(grid_r, grid_c, 0);
                        
                        if neighbor_energy > energy {
                            high_energy_charge_sum += neighbor_charge;
                            high_energy_count += 1;
                        } else {
                            low_energy_charge_sum += neighbor_charge;
                            low_energy_count += 1;
                        }
                    }
                }
            }
        }
        
        let mut new_charge = charge;
        
        // Move toward charge of higher-energy neighbors
        // Move away from charge of lower-energy neighbors
        if high_energy_count > 0 {
            let high_avg = high_energy_charge_sum / high_energy_count as f32;
            new_charge += (high_avg - charge) * 0.1;
        }
        
        if low_energy_count > 0 {
            let low_avg = low_energy_charge_sum / low_energy_count as f32;
            new_charge -= (low_avg - charge) * 0.05; // Weaker repulsion
        }
        
        new_charge.clamp(0.0, 1.0)
    }

    pub fn phase_update(_trait_index: usize, cell_r: usize, cell_c: usize, neighborhood: &Neighborhood, grid: &Grid) -> f32 {
        let phase = grid.get_cell_trait(cell_r, cell_c, 2);
        let charge = grid.get_cell_trait(cell_r, cell_c, 1);
        
        let center_row = neighborhood.center_row;
        let center_col = neighborhood.center_col;
        
        let mut sync_pull: f32 = 0.0;
        let mut neighbor_count = 0;
        
        for mask_r in 0..neighborhood.height {
            for mask_c in 0..neighborhood.width {
                if neighborhood.is_valid(mask_r, mask_c) == 1 
                && !(mask_r == center_row && mask_c == center_col) {
                    let (grid_r, grid_c) = neighborhood.get_grid_coords(mask_r, mask_c, cell_r, cell_c, grid);
                    if !grid.is_cell_empty(grid_r, grid_c) {
                        neighbor_count += 1;
                        
                        let neighbor_phase = grid.get_cell_trait(grid_r, grid_c, 2);
                        let neighbor_charge = grid.get_cell_trait(grid_r, grid_c, 1);
                        
                        // Phase difference (cyclic)
                        let diff = neighbor_phase - phase;
                        let cyclic_diff = if diff > 0.5 {
                            diff - 1.0
                        } else if diff < -0.5 {
                            diff + 1.0
                        } else {
                            diff
                        };
                        
                        // Similar charge = sync together
                        // Opposite charge = anti-sync (push phases apart)
                        let charge_similarity = 1.0 - (charge - neighbor_charge).abs();
                        let coupling = (charge_similarity - 0.5) * 2.0; // -1 to 1
                        
                        sync_pull += cyclic_diff * coupling;
                    }
                }
            }
        }
        
        // Natural advance
        let mut new_phase = phase + 0.07;
        
        // Apply sync/anti-sync
        if neighbor_count > 0 {
            new_phase += (sync_pull / neighbor_count as f32) * 0.12;
        }
        
        new_phase.rem_euclid(1.0)
    }
}




macro_rules! define_rules {
    ($(($variant:ident, $name:expr, $func:path)),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Rule {
            $($variant),*
        }
        
        impl Rule {
            pub const ALL: &'static [Rule] = &[$(Rule::$variant),*];
            pub const NAMES: &'static [&'static str] = &[$($name),*];
            
            #[inline]
            pub fn name(&self) -> &'static str {
                match self {
                    $(Rule::$variant => $name),*
                }
            }
            
            #[inline]
            pub fn from_name(name: &str) -> Option<Rule> {
                match name {
                    $($name => Some(Rule::$variant)),*,
                    _ => None,
                }
            }
            
            #[inline]
            pub fn get_fn(&self) -> RuleFnType {
                match self {
                    $(Rule::$variant => $func),*
                }
            }
        }
    };
}

// ============================================================
// ADD NEW RULES HERE - Just add one line!
// Format: (EnumVariant, "display name", RuleFunction::function_name)
// ============================================================
define_rules!(
    (Static,          "static",           RuleFunction::static_rule),
    (Average,         "average",          RuleFunction::average),
    (Conway,          "conway",           RuleFunction::conway),
    (ConwayOptimized, "conway optimized", RuleFunction::conway_optimized),
    (Diffusion,       "diffusion",        RuleFunction::diffusion),
    (Maximum,         "maximum",          RuleFunction::maximum),
    (Minimum,         "minimum",          RuleFunction::minimum),
    (WeightedAverage, "weighted_average", RuleFunction::weighted_average),
    (Energy,          "energy",           RuleFunction::energy_update),
    (Charge,          "charge",           RuleFunction::charge_update),
    (Phase,           "phase",            RuleFunction::phase_update),
    // Add new rules here:
);

pub type RuleFnType = fn(usize, usize, usize, &Neighborhood, &Grid) -> f32;


#[derive(Clone)]
pub struct RulesRegistry {
    rules: Vec<RuleFnType>,
    rule_types: Vec<Rule>,
}

impl RulesRegistry {
    pub fn default(num_traits: usize) -> Self {
        Self {
            rules: vec![RuleFunction::average; num_traits],
            rule_types: vec![Rule::Average; num_traits],
        }
    }
    
    pub fn custom(rule_types: Vec<Rule>) -> Self {
        let rules = rule_types.iter().map(|rt| rt.get_fn()).collect();
        Self { rules, rule_types }
    }
    
    #[inline(always)]
    pub fn apply_rule(&self, trait_index: usize, cell_r: usize, cell_c: usize, neighborhood: &Neighborhood, grid: &Grid, ) -> f32 {
        let rule = unsafe { *self.rules.get_unchecked(trait_index) };
        rule(trait_index, cell_r, cell_c, neighborhood, grid)
    }
    
    pub fn set_rule(&mut self, trait_idx: usize, rule_type: Rule) {
        self.rules[trait_idx] = rule_type.get_fn();
        self.rule_types[trait_idx] = rule_type;
    }
    
    #[inline]
    pub fn get_rule_name(&self, trait_index: usize) -> &'static str {
        self.rule_types[trait_index].name()
    }

    #[inline]
    pub fn get_rule(&self, trait_index: usize) -> Rule {
        self.rule_types[trait_index]
    }

    pub fn is_stored_function(&self, trait_index: usize, rule: Rule) -> bool {
        self.rule_types[trait_index] == rule
    }

    #[inline]
    pub fn get_name_for_rule(rule: Rule) -> &'static str {
        rule.name()
    }

    #[inline]
    pub fn get_rule_by_name(rule_name: &str) -> Option<Rule> {
        Rule::from_name(rule_name)
    }

    #[inline(always)]
    pub fn get_all_names() -> &'static [&'static str] {
        Rule::NAMES
    }

    #[inline]
    pub fn get_all_rules() -> &'static [Rule] {
        Rule::ALL
    }
}





#[cfg(test)]
mod tests {
    use crate::grid::Grid;
    use crate::rules::{RuleFunction, RulesRegistry};
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

        let result = RuleFunction::average(0, 1, 1, &neighborhood, &grid);
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

        let result = RuleFunction::conway(0, 1, 1, &neighborhood, &grid);
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
            Rule::Static,
            Rule::Conway,
            Rule::Average,
            Rule::Average,
            Rule::Average,
            Rule::Average,
            Rule::Average,
            Rule::Average,
            Rule::Average,
            Rule::Average,
        ]);

        let result = rules_registry.apply_rule(1, 1, 1, &neighborhood, &grid);
        assert!(
            result == 0.0 || result == 1.0,
            "Custom rules registry with Conway rule should produce 0.0 or 1.0"
        );
    }
}
