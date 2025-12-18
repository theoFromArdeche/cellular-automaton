use crate::grid::Grid;

/// Print a single trait array in row-major order
pub fn print_trait_array(grid: &Grid, trait_index: usize, trait_name: &str) {
    let values = grid.get_trait_array(trait_index);
    
    println!("\n=== Trait {} ({}) ===", trait_index, trait_name);
    println!("[");
    
    for row in 0..grid.height {
        print!("  [");
        for col in 0..grid.width {
            let idx = row * grid.width + col;
            print!("{:.3}", values[idx]);
            if col < grid.width - 1 {
                print!(", ");
            }
        }
        println!("]");
    }
    println!("]");
}

/// Print all active trait arrays
pub fn print_active_traits(grid: &Grid, active_mask: &[Vec<bool>], trait_names: &[String; 9]) {
    for mask_row in 0..3 {
        for mask_col in 0..3 {
            if active_mask[mask_row][mask_col] {
                let trait_index = mask_row * 3 + mask_col;
                print_trait_array(grid, trait_index, &trait_names[trait_index]);
            }
        }
    }
}

/// Print grid statistics
pub fn print_statistics(grid: &Grid, active_mask: &[Vec<bool>]) {
    println!("\n=== Grid Statistics ===");
    println!("Grid size: {}x{}", grid.width, grid.height);
    println!("Total cells: {}", grid.width * grid.height);
    
    let active_count = active_mask.iter().flatten().filter(|&&x| x).count();
    println!("Active traits: {}/9", active_count);
    
    for mask_row in 0..3 {
        for mask_col in 0..3 {
            if active_mask[mask_row][mask_col] {
                let trait_index = mask_row * 3 + mask_col;
                let values = grid.get_trait_array(trait_index);
                
                let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
                let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let avg = values.iter().sum::<f32>() / values.len() as f32;
                
                println!("  Trait {}: min={:.3}, max={:.3}, avg={:.3}", 
                    trait_index, min, max, avg);
            }
        }
    }
}

/// Create default trait names
pub fn default_trait_names() -> [String; 9] {
    [
        "Trait_0_0".to_string(),
        "Trait_0_1".to_string(),
        "Trait_0_2".to_string(),
        "Trait_1_0".to_string(),
        "Trait_1_1".to_string(),
        "Trait_1_2".to_string(),
        "Trait_2_0".to_string(),
        "Trait_2_1".to_string(),
        "Trait_2_2".to_string(),
    ]
}

/// Create semantic trait names for examples
pub fn semantic_trait_names() -> [String; 9] {
    [
        "Energy".to_string(),
        "Confidence".to_string(),
        "Cooperation".to_string(),
        "Aggression".to_string(),
        "Stability".to_string(),
        "Mobility".to_string(),
        "Resource".to_string(),
        "Age".to_string(),
        "Adaptability".to_string(),
    ]
}

/// Print a horizontal separator
pub fn print_separator() {
    println!("\n{}", "=".repeat(60));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_names() {
        let names = default_trait_names();
        assert_eq!(names.len(), 9);
        assert_eq!(names[0], "Trait_0_0");
        assert_eq!(names[8], "Trait_2_2");
    }
}
