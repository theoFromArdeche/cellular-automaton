use crate::grid::Grid;
use crate::rules::RulesRegistry;



/// Print active trait indices with their names
pub fn print_active_traits(num_traits: usize, active_mask: &[u8], trait_names: &[String], rules_registry: &RulesRegistry) {
    println!("  Active traits:");
    for trait_index in 0..num_traits {
        if active_mask[trait_index] == 0 {
            continue;
        }
        let trait_name = &trait_names[trait_index];
        let rule_name = rules_registry.get_rule_name(trait_index);
        println!("    {}: {} (rule: {})", trait_index, trait_name, rule_name);
    }
}

/// Print a single trait array in row-major order
pub fn print_trait_array(grid: &Grid, trait_index: usize, trait_names: &[String]) {
    let values = grid.get_trait_slice(trait_index);
    
    println!("\n=== Trait {} ({}) ===", trait_index, trait_names[trait_index]);
    println!("[");
    
    for row in 0..grid.height {
        print!("  [");
        for col in 0..grid.width {
            let idx = row * grid.width + col;
            if grid.is_cell_empty(row, col) {
                print!(".....");
            } else {
                print!("{:.3}", values[idx]);
            }
            
            if col < grid.width - 1 {
                print!(", ");
            }
        }
        println!("]");
    }
    println!("]");
}

/// Print all active trait arrays
pub fn print_active_traits_array(grid: &Grid, active_mask: &[u8], trait_names: &[String]) {
    for trait_index in 0..grid.num_traits {
        if active_mask[trait_index] == 0 {
            continue;
        }
        print_trait_array(grid, trait_index, trait_names);
    }
}

/// Print grid statistics
pub fn print_statistics(grid: &Grid, active_mask: &[u8; 9]) {
    println!("\n=== Grid Statistics ===");
    println!("Grid size: {}x{}", grid.width, grid.height);
    println!("Total cells: {}", grid.width * grid.height);
    println!("Active traits: {}/9", active_mask.iter().sum::<u8>());

    for trait_index in 0..grid.num_traits {
        if active_mask[trait_index] == 0 {
            continue;
        }
        let values = grid.get_trait_slice(trait_index);

        let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let avg = values.iter().sum::<f32>() / values.len() as f32;

        println!(
            "  Trait {}: min={:.3}, max={:.3}, avg={:.3}",
            trait_index, min, max, avg
        );
    }
}

/// Create default trait names
pub fn default_traits_names() -> Vec<String> {
    vec![
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
pub fn semantic_traits_names() -> Vec<String> {
    vec![
        "Energy".to_string(),
        "Social".to_string(),
        "Color/Team".to_string(),
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
    fn test_traits_names() {
        let names = default_traits_names();
        assert_eq!(names.len(), 9);
        assert_eq!(names[0], "Trait_0_0");
        assert_eq!(names[8], "Trait_2_2");
    }
}
