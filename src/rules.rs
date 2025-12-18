use crate::cell::Cell;

/// Type alias for trait update functions
pub type TraitUpdateFn = fn(&Cell, &[Cell]) -> f32;

/// Example rule: Average of neighbors' trait values
pub fn rule_average(cell: &Cell, neighbors: &[Cell], trait_index: usize) -> f32 {
    if neighbors.is_empty() {
        return cell.get_trait(trait_index);
    }
    
    let sum: f32 = neighbors.iter().map(|n| n.get_trait(trait_index)).sum();
    (sum / neighbors.len() as f32).clamp(0.0, 1.0)
}

/// Example rule: Conway's Game of Life style (for binary-like traits)
pub fn rule_conway(cell: &Cell, neighbors: &[Cell], trait_index: usize) -> f32 {
    let alive_neighbors = neighbors
        .iter()
        .filter(|n| n.get_trait(trait_index) > 0.5)
        .count();
    
    let current_val = cell.get_trait(trait_index);
    
    if current_val > 0.5 {
        // Cell is "alive"
        if alive_neighbors == 2 || alive_neighbors == 3 {
            1.0
        } else {
            0.0
        }
    } else {
        // Cell is "dead"
        if alive_neighbors == 3 {
            1.0
        } else {
            0.0
        }
    }
}

/// Example rule: Diffusion with decay
pub fn rule_diffusion(cell: &Cell, neighbors: &[Cell], trait_index: usize) -> f32 {
    if neighbors.is_empty() {
        return (cell.get_trait(trait_index) * 0.95).clamp(0.0, 1.0);
    }
    
    let current = cell.get_trait(trait_index);
    let avg_neighbors: f32 = neighbors.iter().map(|n| n.get_trait(trait_index)).sum::<f32>() 
        / neighbors.len() as f32;
    
    // Mix 70% average with neighbors, 30% current value, then decay
    let new_val = (0.3 * current + 0.7 * avg_neighbors) * 0.98;
    new_val.clamp(0.0, 1.0)
}

/// Example rule: Maximum of neighbors
pub fn rule_maximum(cell: &Cell, neighbors: &[Cell], trait_index: usize) -> f32 {
    if neighbors.is_empty() {
        return cell.get_trait(trait_index);
    }
    
    let max_neighbor = neighbors
        .iter()
        .map(|n| n.get_trait(trait_index))
        .fold(0.0f32, |a, b| a.max(b));
    
    max_neighbor.max(cell.get_trait(trait_index) * 0.9).clamp(0.0, 1.0)
}

/// Example rule: Oscillating behavior
pub fn rule_oscillate(cell: &Cell, neighbors: &[Cell], trait_index: usize) -> f32 {
    let current = cell.get_trait(trait_index);
    let avg_neighbors: f32 = if neighbors.is_empty() {
        current
    } else {
        neighbors.iter().map(|n| n.get_trait(trait_index)).sum::<f32>() / neighbors.len() as f32
    };
    
    // Oscillate based on difference from neighbors
    let diff = (current - avg_neighbors).abs();
    ((current + diff * 0.5) % 1.0).clamp(0.0, 1.0)
}

/// Rule collection for easy trait assignment
pub struct RuleSet {
    pub rules: [fn(&Cell, &[Cell], usize) -> f32; 9],
}

impl RuleSet {
    /// Create a default rule set where all traits use the average rule
    pub fn default() -> Self {
        Self {
            rules: [
                rule_average,
                rule_average,
                rule_average,
                rule_average,
                rule_average,
                rule_average,
                rule_average,
                rule_average,
                rule_average,
            ],
        }
    }

    /// Create a custom rule set
    pub fn custom(rules: [fn(&Cell, &[Cell], usize) -> f32; 9]) -> Self {
        Self { rules }
    }

    /// Apply the rule for a specific trait
    pub fn apply_rule(&self, trait_index: usize, cell: &Cell, neighbors: &[Cell]) -> f32 {
        (self.rules[trait_index])(cell, neighbors, trait_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_average() {
        let cell = Cell::new((0, 0));
        let mut neighbor1 = Cell::new((0, 1));
        neighbor1.set_trait(0, 0.5);
        let mut neighbor2 = Cell::new((1, 0));
        neighbor2.set_trait(0, 0.8);
        
        let neighbors = vec![neighbor1, neighbor2];
        let result = rule_average(&cell, &neighbors, 0);
        assert!((result - 0.65).abs() < 0.01);
    }
}
