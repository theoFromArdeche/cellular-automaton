use trait_ac::rules::Rule;
use trait_ac::movement::Movement;

use crate::color_scheme::ColorScheme;

use serde::Deserialize;
use std::fs;

// Custom deserializer for Rule
fn deserialize_rules<'de, D>(deserializer: D) -> Result<Vec<Rule>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let names: Vec<String> = Vec::deserialize(deserializer)?;
    names
        .into_iter()
        .map(|name| {
            Rule::from_name(&name).ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "Unknown rule: '{}'. Valid rules are: {:?}",
                    name,
                    Rule::NAMES
                ))
            })
        })
        .collect()
}

// Custom deserializer for Movement
fn deserialize_movement<'de, D>(deserializer: D) -> Result<Movement, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    Movement::from_name(&name).ok_or_else(|| {
        serde::de::Error::custom(format!(
            "Unknown movement: '{}'. Valid movements are: {:?}",
            name,
            Movement::NAMES
        ))
    })
}

// Custom deserializer for ColorScheme
fn deserialize_color_scheme<'de, D>(deserializer: D) -> Result<ColorScheme, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    ColorScheme::from_name(&name).ok_or_else(|| {
        serde::de::Error::custom(format!(
            "Unknown color scheme: '{}'. Valid color schemes are: {:?}",
            name,
            ColorScheme::NAMES
        ))
    })
}


#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    // Grid settings
    pub grid_width: usize,
    pub grid_height: usize,
    pub grid_density: f32,
    pub num_traits: usize,

    // Simulation timing
    pub steps_per_second: f32,
    pub timed_simulation: bool,
    pub timestep_max: usize,

    // Grid bounds
    pub grid_width_min: usize,
    pub grid_width_max: usize,
    pub grid_height_min: usize,
    pub grid_height_max: usize,

    // Steps per second bounds
    pub steps_per_second_min: f32,
    pub steps_per_second_max: f32,

    // Cell display
    pub cell_size: f32,
    pub cell_size_min: f32,
    pub cell_size_max: f32,

    // Value display
    pub show_values: bool,
    pub show_values_minimum_cell_size: f32,

    // Stats display
    pub show_stats: bool,

    // Colors
    #[serde(deserialize_with = "deserialize_color_scheme")]
    pub color_scheme: ColorScheme,
    pub base_color_not_empty: f32,
    pub base_color_not_empty_min: f32,
    pub base_color_not_empty_max: f32,

    // Trait settings
    pub active_mask: Vec<u8>,
    pub initial_selected_trait: usize,
    pub initialisation_ranges: Vec<(f32, f32)>,

    // Rules & movement
    #[serde(deserialize_with = "deserialize_rules")]
    pub rules: Vec<Rule>,
    #[serde(deserialize_with = "deserialize_movement")]
    pub movement: Movement,

    // Neighborhood masks
    pub neighborhood_traits_mask: Vec<Vec<u8>>,
    pub neighborhood_mvt_mask: Vec<Vec<u8>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grid_width: 500,
            grid_height: 500,
            grid_density: 0.5,
            num_traits: 9,

            steps_per_second: 1000.0,
            timed_simulation: false,
            timestep_max: 100,

            grid_width_min: 3,
            grid_width_max: 5000,
            grid_height_min: 3,
            grid_height_max: 5000,

            steps_per_second_min: 1.0,
            steps_per_second_max: 10000.0,

            cell_size: 1.0,
            cell_size_min: 0.1,
            cell_size_max: 100.0,

            show_values: false,
            show_values_minimum_cell_size: 20.0,
            show_stats: true,

            color_scheme: ColorScheme::Viridis,
            base_color_not_empty: 0.0,
            base_color_not_empty_min: 0.0,
            base_color_not_empty_max: 1.0,

            active_mask: vec![
                1, 1, 0,
                0, 0, 0,
                0, 0, 0,
            ],
            initial_selected_trait: 0,

            initialisation_ranges: vec![
                (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
                (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
                (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
            ],

            rules: vec![
                Rule::SocialEnergy, Rule::SocialInfluence, Rule::ConwayOptimized,
                Rule::ConwayOptimized, Rule::ConwayOptimized, Rule::ConwayOptimized,
                Rule::ConwayOptimized, Rule::ConwayOptimized, Rule::ConwayOptimized,
            ],
            movement: Movement::Social,

            neighborhood_traits_mask: vec![
                vec![1, 1, 1],
                vec![1, 1, 1],
                vec![1, 1, 1],
            ],
            neighborhood_mvt_mask: vec![
                vec![1, 1, 1],
                vec![1, 1, 1],
                vec![1, 1, 1],
            ],
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.grid_width < self.grid_width_min || self.grid_width > self.grid_width_max {
            return Err("grid_width is out of bounds");
        }
        if self.grid_height < self.grid_height_min || self.grid_height > self.grid_height_max {
            return Err("grid_height is out of bounds");
        }
        if !(0.0..=1.0).contains(&self.grid_density) {
            return Err("grid_density must be between 0.0 and 1.0");
        }
        if self.num_traits <= 0 {
            return Err("num_traits must be > 0");
        }
        if self.steps_per_second < self.steps_per_second_min
            || self.steps_per_second > self.steps_per_second_max
        {
            return Err("steps_per_second is out of bounds");
        }
        if self.cell_size < self.cell_size_min || self.cell_size > self.cell_size_max {
            return Err("cell_size is out of bounds");
        }
        if self.cell_size_min < 0.0 || self.cell_size_min > self.cell_size_max {
            return Err("cell_size_min should be between 0.0 and cell_size_max");
        }
        if self.cell_size_max < self.cell_size_min {
            return Err("cell_size_max should be greater than cell_size_min");
        }
        if self.base_color_not_empty < self.base_color_not_empty_min || self.base_color_not_empty > self.base_color_not_empty_max {
            return Err("base_color_not_empty is out of bounds");
        }
        if self.base_color_not_empty_min < 0.0 || self.base_color_not_empty_min > 1.0 {
            return Err("base_color_not_empty_min should be between 0.0 and 1.0");
        }
        if self.base_color_not_empty_max < 0.0 || self.base_color_not_empty_max > 1.0 {
            return Err("base_color_not_empty_max should be between 0.0 and 1.0");
        }
        if self.active_mask.is_empty() {
            return Err("active_mask must not be empty");
        }
        // Check that no active bit in active_mask is at an index >= num_traits
        let max_active_index = self.active_mask
            .iter()
            .enumerate()
            .filter(|&(_, &m)| m != 0)
            .map(|(i, _)| i)
            .max();
        if let Some(max_idx) = max_active_index {
            if max_idx >= self.num_traits as usize {
                return Err("active_mask has active traits indexes beyond 'num_traits' (only the indexes of active_mask from 0 to 'num_traits' can be used)");
            }
        }
        if self.initialisation_ranges.is_empty() {
            return Err("initialisation_ranges must not be empty");
        }
        if self.rules.is_empty() {
            return Err("rules must not be empty");
        }
        if self.initial_selected_trait >= self.num_traits {
            return Err("initial_selected_trait is out of bounds");
        }
        Ok(())
    }
}