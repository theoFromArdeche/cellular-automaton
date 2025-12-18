use crate::cell::Cell;
use crate::neighborhood::Neighborhood;


/// No movement - cells stay in place
pub fn rule_static(cell: &Cell, _neighborhood: &Neighborhood, trait_index: usize) -> f32 {
    cell.get_trait(trait_index)
}

/// Example rule: Average of neighbors' trait values
pub fn rule_average(cell: &Cell, neighborhood: &Neighborhood, trait_index: usize) -> f32 {
    let mut sum = 0.0;
    let mut count = 0;
    
    for (delta_row, row) in neighborhood.cells.iter().enumerate() {
        for (delta_col, neighbor) in row.iter().enumerate() {
            // Use mask to check if this cell is in the neighborhood
            if neighborhood.mask[delta_row][delta_col] {
                sum += neighbor.get_trait(trait_index);
                count += 1;
            }
        }
    }
    
    if count == 0 {
        return cell.get_trait(trait_index);
    }
    
    (sum / count as f32).clamp(0.0, 1.0)
}

/// Example rule: Conway's Game of Life style (for binary-like traits)
pub fn rule_conway(cell: &Cell, neighborhood: &Neighborhood, trait_index: usize) -> f32 {
    let mut alive_neighbors = 0;
    
    for (delta_row, row) in neighborhood.cells.iter().enumerate() {
        for (delta_col, neighbor) in row.iter().enumerate() {
            // Skip the center cell and use mask to check if this cell is in the neighborhood
            if neighborhood.mask[delta_row][delta_col] && 
               !(delta_row == neighborhood.center_row && delta_col == neighborhood.center_col) {
                if neighbor.get_trait(trait_index) > 0.5 {
                    alive_neighbors += 1;
                }
            }
        }
    }
    
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
pub fn rule_diffusion(cell: &Cell, neighborhood: &Neighborhood, trait_index: usize) -> f32 {
    let mut sum = 0.0;
    let mut count = 0;
    
    for (delta_row, row) in neighborhood.cells.iter().enumerate() {
        for (delta_col, neighbor) in row.iter().enumerate() {
            // Skip the center cell and use mask to check if this cell is in the neighborhood
            if neighborhood.mask[delta_row][delta_col] && 
               !(delta_row == neighborhood.center_row && delta_col == neighborhood.center_col) {
                sum += neighbor.get_trait(trait_index);
                count += 1;
            }
        }
    }
    
    if count == 0 {
        return (cell.get_trait(trait_index) * 0.95).clamp(0.0, 1.0);
    }
    
    let current = cell.get_trait(trait_index);
    let avg_neighbors = sum / count as f32;
    
    // Mix 70% average with neighbors, 30% current value, then decay
    let new_val = (0.3 * current + 0.7 * avg_neighbors) * 0.98;
    new_val.clamp(0.0, 1.0)
}

/// Example rule: Maximum of neighbors
pub fn rule_maximum(cell: &Cell, neighborhood: &Neighborhood, trait_index: usize) -> f32 {
    let mut max_neighbor = 0.0f32;
    let mut found_any = false;
    
    for (delta_row, row) in neighborhood.cells.iter().enumerate() {
        for (delta_col, neighbor) in row.iter().enumerate() {
            // Use mask to check if this cell is in the neighborhood
            if neighborhood.mask[delta_row][delta_col] {
                max_neighbor = max_neighbor.max(neighbor.get_trait(trait_index));
                found_any = true;
            }
        }
    }
    
    if !found_any {
        return cell.get_trait(trait_index);
    }
    
    max_neighbor.max(cell.get_trait(trait_index) * 0.9).clamp(0.0, 1.0)
}

/// Example rule: Oscillating behavior
pub fn rule_oscillate(cell: &Cell, neighborhood: &Neighborhood, trait_index: usize) -> f32 {
    let current = cell.get_trait(trait_index);
    
    let mut sum = 0.0;
    let mut count = 0;
    
    for (delta_row, row) in neighborhood.cells.iter().enumerate() {
        for (delta_col, neighbor) in row.iter().enumerate() {
            // Skip the center cell and use mask to check if this cell is in the neighborhood
            if neighborhood.mask[delta_row][delta_col] && 
               !(delta_row == neighborhood.center_row && delta_col == neighborhood.center_col) {
                sum += neighbor.get_trait(trait_index);
                count += 1;
            }
        }
    }
    
    let avg_neighbors = if count == 0 {
        current
    } else {
        sum / count as f32
    };
    
    // Oscillate based on difference from neighbors
    let diff = (current - avg_neighbors).abs();
    ((current + diff * 0.5) % 1.0).clamp(0.0, 1.0)
}

/// Example rule: Weighted average based on distance from center
pub fn rule_weighted_average(cell: &Cell, neighborhood: &Neighborhood, trait_index: usize) -> f32 {
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;
    
    for (delta_row, row) in neighborhood.cells.iter().enumerate() {
        for (delta_col, neighbor) in row.iter().enumerate() {
            // Skip the center cell and use mask to check if this cell is in the neighborhood
            if neighborhood.mask[delta_row][delta_col] && 
               !(delta_row == neighborhood.center_row && delta_col == neighborhood.center_col) {
                // Calculate distance from center
                let dr = (delta_row as isize - neighborhood.center_row as isize).abs() as f32;
                let dc = (delta_col as isize - neighborhood.center_col as isize).abs() as f32;
                let distance = (dr * dr + dc * dc).sqrt();
                
                // Weight inversely proportional to distance (closer = more weight)
                let weight = 1.0 / (1.0 + distance);
                
                weighted_sum += neighbor.get_trait(trait_index) * weight;
                total_weight += weight;
            }
        }
    }
    
    if total_weight == 0.0 {
        return cell.get_trait(trait_index);
    }
    
    (weighted_sum / total_weight).clamp(0.0, 1.0)
}

/// Example rule: Only consider direct neighbors (not diagonals)
pub fn rule_von_neumann(cell: &Cell, neighborhood: &Neighborhood, trait_index: usize) -> f32 {
    let mut sum = 0.0;
    let mut count = 0;
    
    let center_row = neighborhood.center_row;
    let center_col = neighborhood.center_col;
    
    // Only check cardinal directions (up, down, left, right)
    let directions = [
        (center_row.checked_sub(1), Some(center_col)),  // up
        (Some(center_row + 1), Some(center_col)),       // down
        (Some(center_row), center_col.checked_sub(1)),  // left
        (Some(center_row), Some(center_col + 1)),       // right
    ];
    
    for &(row_opt, col_opt) in &directions {
        if let (Some(dr), Some(dc)) = (row_opt, col_opt) {
            if dr < neighborhood.height && dc < neighborhood.width {
                // Use mask to check if this cell is in the neighborhood
                if neighborhood.mask[dr][dc] {
                    let neighbor = &neighborhood.cells[dr][dc];
                    sum += neighbor.get_trait(trait_index);
                    count += 1;
                }
            }
        }
    }
    
    if count == 0 {
        return cell.get_trait(trait_index);
    }
    
    (sum / count as f32).clamp(0.0, 1.0)
}

/// Rule collection for easy trait assignment
pub struct RuleSet {
    pub rules: [fn(&Cell, &Neighborhood, usize) -> f32; 9],
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
    pub fn custom(rules: [fn(&Cell, &Neighborhood, usize) -> f32; 9]) -> Self {
        Self { rules }
    }

    /// Apply the rule for a specific trait
    pub fn apply_rule(&self, trait_index: usize, cell: &Cell, neighborhood: &Neighborhood) -> f32 {
        (self.rules[trait_index])(cell, neighborhood, trait_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    #[test]
    fn test_rule_average() {
        let grid = Grid::new(3, 3);
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        
        let neighborhood = Neighborhood::new(3, 3, 1, 1, 1, 1, &mask, &grid);
        let cell = &grid.cells[1][1];
        
        let result = rule_average(cell, &neighborhood, 0);
        assert!(result >= 0.0 && result <= 1.0);
    }

    #[test]
    fn test_rule_conway() {
        let grid = Grid::new(3, 3);
        let mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        
        let neighborhood = Neighborhood::new(3, 3, 1, 1, 1, 1, &mask, &grid);
        let cell = &grid.cells[1][1];
        
        let result = rule_conway(cell, &neighborhood, 0);
        assert!(result == 0.0 || result == 1.0);
    }
}