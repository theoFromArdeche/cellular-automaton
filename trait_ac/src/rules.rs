use crate::cell::Cell;
use crate::neighborhood::Neighborhood;

/// No change - cells maintain their trait value
#[inline(always)]
pub fn rule_static(cell: &Cell, _neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
    cell.get_trait(trait_index)
}

/// Average of neighbors' trait values
pub fn rule_average(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
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
pub fn rule_conway(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
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
pub fn rule_diffusion(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
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
pub fn rule_maximum(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
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
pub fn rule_minimum(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
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
pub fn rule_weighted_average(cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
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
pub fn rule_majority(_cell: &Cell, neighborhood_traits: &Neighborhood, trait_index: usize) -> f32 {
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

/// Rule collection
pub struct RuleSet {
    pub rules: [fn(&Cell, &Neighborhood, usize) -> f32; 9],
}

impl RuleSet {
    pub fn default() -> Self {
        Self {
            rules: [
                rule_average, rule_average, rule_average,
                rule_average, rule_average, rule_average,
                rule_average, rule_average, rule_average,
            ],
        }
    }

    pub fn custom(rules: [fn(&Cell, &Neighborhood, usize) -> f32; 9]) -> Self {
        Self { rules }
    }

    #[inline(always)]
    pub fn apply_rule(
        &self,
        cell: &Cell,
        neighborhood_traits: &Neighborhood,
        trait_index: usize,
    ) -> f32 {
        let rule = unsafe { *self.rules.get_unchecked(trait_index) };
        rule(cell, neighborhood_traits, trait_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    #[test]
    fn test_rule_average() {
        let grid = Grid::new(3, 3);
        let mask = vec![vec![true; 3]; 3];
        let neighborhood_traits = Neighborhood::new(3, 3, 1, 1, 1, 1, &mask, &grid);
        let cell = &grid.cells[1][1];

        let result = rule_average(cell, &neighborhood_traits, 0);
        assert!((0.0..=1.0).contains(&result));
    }

    #[test]
    fn test_rule_conway() {
        let grid = Grid::new(3, 3);
        let mask = vec![vec![true; 3]; 3];
        let neighborhood_traits = Neighborhood::new(3, 3, 1, 1, 1, 1, &mask, &grid);
        let cell = &grid.cells[1][1];

        let result = rule_conway(cell, &neighborhood_traits, 0);
        assert!(result == 0.0 || result == 1.0);
    }
}
