use crate::cell::Cell;
use crate::neighborhood::Neighborhood;



pub struct Rules;

impl Rules {
    /// No change - cells maintain their trait value
    #[inline(always)]
    pub fn static_rule(cell: &Cell, _neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
        cell.get_trait(trait_index)
    }

    /// Average of neighbors' trait values
    pub fn average(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
        let mut sum = 0.0;
        let mut count = 0;

        for r in 0..neighborhood_traits.height {
            for c in 0..neighborhood_traits.width {
                if unsafe { *neighborhood_traits.mask.get_unchecked(r).get_unchecked(c) } {
                    let neighbor = unsafe { neighborhood_traits.cells.get_unchecked(r).get_unchecked(c) };
                    if !neighbor.is_empty() {
                        sum += neighbor.get_trait(trait_index);
                        count += 1;
                    }
                }
            }
        }

        if count == 0 {
            cell.get_trait(trait_index)
        } else {
            (sum / count as f32).clamp(0.0, 1.0)
        }
    }

    /// Conway's Game of Life style (binary traits)
    pub fn conway(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
        let mut alive_neighbors = 0;
        let center_row = neighborhood_traits.center_row;
        let center_col = neighborhood_traits.center_col;

        for r in 0..neighborhood_traits.height {
            for c in 0..neighborhood_traits.width {
                if unsafe { *neighborhood_traits.mask.get_unchecked(r).get_unchecked(c) }
                    && !(r == center_row && c == center_col)
                {
                    let neighbor = unsafe { neighborhood_traits.cells.get_unchecked(r).get_unchecked(c) };
                    if !neighbor.is_empty() && neighbor.get_trait(trait_index) > 0.5 {
                        alive_neighbors += 1;
                    }
                }
            }
        }

        let current = cell.get_trait(trait_index);

        if current > 0.5 {
            if alive_neighbors == 2 || alive_neighbors == 3 { 1.0 } else { 0.0 }
        } else {
            if alive_neighbors == 3 { 1.0 } else { 0.0 }
        }
    }

    /// Diffusion with decay
    pub fn diffusion(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
        let mut sum = 0.0;
        let mut count = 0;

        let center_row = neighborhood_traits.center_row;
        let center_col = neighborhood_traits.center_col;

        for r in 0..neighborhood_traits.height {
            for c in 0..neighborhood_traits.width {
                if unsafe { *neighborhood_traits.mask.get_unchecked(r).get_unchecked(c) }
                    && !(r == center_row && c == center_col)
                {
                    let neighbor = unsafe { neighborhood_traits.cells.get_unchecked(r).get_unchecked(c) };
                    if !neighbor.is_empty() {
                        sum += neighbor.get_trait(trait_index);
                        count += 1;
                    }
                }
            }
        }

        if count == 0 {
            return (cell.get_trait(trait_index) * 0.95).clamp(0.0, 1.0);
        }

        let current = cell.get_trait(trait_index);
        let avg = sum / count as f32;

        ((0.3 * current + 0.7 * avg) * 0.98).clamp(0.0, 1.0)
    }

    /// Maximum of neighbors
    pub fn maximum(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
        let mut max_val = cell.get_trait(trait_index);

        for r in 0..neighborhood_traits.height {
            for c in 0..neighborhood_traits.width {
                if unsafe { *neighborhood_traits.mask.get_unchecked(r).get_unchecked(c) } {
                    let neighbor = unsafe { neighborhood_traits.cells.get_unchecked(r).get_unchecked(c) };
                    if !neighbor.is_empty() {
                        max_val = max_val.max(neighbor.get_trait(trait_index));
                    }
                }
            }
        }

        (max_val * 0.98).clamp(0.0, 1.0)
    }

    /// Minimum of neighbors
    pub fn minimum(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
        let mut min_val = cell.get_trait(trait_index);

        for r in 0..neighborhood_traits.height {
            for c in 0..neighborhood_traits.width {
                if unsafe { *neighborhood_traits.mask.get_unchecked(r).get_unchecked(c) } {
                    let neighbor = unsafe { neighborhood_traits.cells.get_unchecked(r).get_unchecked(c) };
                    if !neighbor.is_empty() {
                        min_val = min_val.min(neighbor.get_trait(trait_index));
                    }
                }
            }
        }

        min_val.clamp(0.0, 1.0)
    }

    /// Weighted average by distance
    pub fn weighted_average(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
        let mut sum = 0.0;
        let mut weight_sum = 0.0;

        let cr = neighborhood_traits.center_row;
        let cc = neighborhood_traits.center_col;

        for r in 0..neighborhood_traits.height {
            for c in 0..neighborhood_traits.width {
                if unsafe { *neighborhood_traits.mask.get_unchecked(r).get_unchecked(c) }
                    && !(r == cr && c == cc)
                {
                    let neighbor = unsafe { neighborhood_traits.cells.get_unchecked(r).get_unchecked(c) };
                    if !neighbor.is_empty() {
                        let dr = (r as isize - cr as isize).abs() as f32;
                        let dc = (c as isize - cc as isize).abs() as f32;
                        let w = 1.0 / (1.0 + (dr * dr + dc * dc).sqrt());

                        sum += neighbor.get_trait(trait_index) * w;
                        weight_sum += w;
                    }
                }
            }
        }

        if weight_sum == 0.0 {
            cell.get_trait(trait_index)
        } else {
            (sum / weight_sum).clamp(0.0, 1.0)
        }
    }

    /// Majority rule (quantized)
    pub fn majority(_cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
        let mut bins = [0u32; 5];

        for r in 0..neighborhood_traits.height {
            for c in 0..neighborhood_traits.width {
                if unsafe { *neighborhood_traits.mask.get_unchecked(r).get_unchecked(c) } {
                    let neighbor = unsafe { neighborhood_traits.cells.get_unchecked(r).get_unchecked(c) };
                    if !neighbor.is_empty() {
                        let v = neighbor.get_trait(trait_index);
                        let bin = ((v * 5.0).floor() as usize).min(4);
                        bins[bin] += 1;
                    }
                }
            }
        }

        let max_bin = bins.iter()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .map(|(i, _)| i)
            .unwrap_or(0);

        ((max_bin as f32 + 0.5) / 5.0).clamp(0.0, 1.0)
    }
}




pub type RuleFn = for<'c> fn(&Cell, &Neighborhood<'c>, usize) -> f32;

#[derive(Clone, Copy)]
pub struct RulesRegistry {
    rules: [RuleFn; 9],
}

// Static lookup table for function pointer to name mapping
static RULE_LOOKUP: &[(RuleFn, &str)] = &[
    (Rules::static_rule, "static"),
    (Rules::average, "average"),
    (Rules::conway, "conway"),
    (Rules::diffusion, "diffusion"),
    (Rules::maximum, "maximum"),
    (Rules::minimum, "minimum"),
    (Rules::weighted_average, "weighted_average"),
    (Rules::majority, "majority"),
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
    pub fn apply_rule(&self, cell: &Cell, neighborhood: &Neighborhood, trait_index: usize) -> f32 {
        let rule = unsafe { *self.rules.get_unchecked(trait_index) };
        rule(cell, neighborhood, trait_index)
    }

    pub fn set_rule(&mut self, trait_idx: usize, rule_fn: RuleFn) {
        self.rules[trait_idx] = rule_fn;
    }

    /// Get the name of the rule assigned to a trait index
    #[inline]
    pub fn get_rule_name(&self, trait_index: usize) -> &'static str {
        let rule_fn = unsafe { *self.rules.get_unchecked(trait_index) };
        Self::get_name_for_rule(rule_fn)
    }

    pub fn is_stored_function(self, trait_index: usize, function: RuleFn) -> bool {
        let rule_fn = unsafe { *self.rules.get_unchecked(trait_index) };
        rule_fn as usize == function as usize
    }

    /// Get the name for a specific rule function (uses lookup table)
    #[inline]
    pub fn get_name_for_rule(rule_fn: RuleFn) -> &'static str {
        for &(func, name) in RULE_LOOKUP {
            if func as usize == rule_fn as usize {
                return name;
            }
        }
        "unknown"
    }

    /// Get rule function by name (uses lookup table)
    #[inline]
    pub fn get_rule_by_name(rule_name: &str) -> Option<RuleFn> {
        for &(func, name) in RULE_LOOKUP {
            if name == rule_name {
                return Some(func);
            }
        }
        None
    }

    /// Get all available rule names (from lookup table)
    #[inline(always)]
    pub fn get_all_names() -> &'static [&'static str; RULE_COUNT] {
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
            1, 1,        // center row, center col for movement
            &mask,       // mask
            &grid,       // reference to grid
        );
        let cell = &grid.cells[1][1];

        let result = Rules::average(cell, &neighborhood, 0);
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
            1, 1,        // center row, center col for movement
            &mask,       // mask
            &grid,       // reference to grid
        );
        let cell = &grid.cells[1][1];

        let result = Rules::conway(cell, &neighborhood, 0);
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
            1, 1,        // center row, center col for movement
            &mask,       // mask
            &grid,       // reference to grid
        );
        let cell = &grid.cells[1][1];

        let rules_registry = RulesRegistry::default();
        let result = rules_registry.apply_rule(cell, &neighborhood, 0);

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
            1, 1,        // center row, center col for movement
            &mask,       // mask
            &grid,       // reference to grid
        );
        let cell = &grid.cells[1][1];

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

        let result = rules_registry.apply_rule(cell, &neighborhood, 1);
        assert!(
            result == 0.0 || result == 1.0,
            "Custom rules registry with Conway rule should produce 0.0 or 1.0"
        );
    }
}
