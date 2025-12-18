use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use trait_ac::grid::Grid;
use trait_ac::neighborhood::Neighborhood;
use trait_ac::rules::{rule_static, rule_average, rule_conway, rule_diffusion, rule_maximum, rule_oscillate, rule_weighted_average, rule_von_neumann};
use trait_ac::movement::{apply_movement, movement_static, movement_random, movement_gradient, movement_avoid_crowding, movement_trait_based};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Cellular Automata Simulator"),
        ..Default::default()
    };
    
    eframe::run_native(
        "CA Simulator",
        options,
        Box::new(|_cc| Ok(Box::new(CAApp::default()))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RuleType {
    Static,
    Average,
    Conway,
    Diffusion,
    Maximum,
    Oscillate,
    WeightedAverage,
    VonNeumann,
}

impl RuleType {
    fn all() -> Vec<Self> {
        vec![
            Self::Static,
            Self::Average,
            Self::Conway,
            Self::Diffusion,
            Self::Maximum,
            Self::Oscillate,
            Self::WeightedAverage,
            Self::VonNeumann,
        ]
    }
    
    fn name(&self) -> &str {
        match self {
            Self::Static => "Static",
            Self::Average => "Average",
            Self::Conway => "Conway",
            Self::Diffusion => "Diffusion",
            Self::Maximum => "Maximum",
            Self::Oscillate => "Oscillate",
            Self::WeightedAverage => "Weighted Avg",
            Self::VonNeumann => "Von Neumann",
        }
    }
    
    fn get_rule_fn(&self) -> fn(&trait_ac::cell::Cell, &Neighborhood, usize) -> f32 {
        match self {
            Self::Static => rule_static,
            Self::Average => rule_average,
            Self::Conway => rule_conway,
            Self::Diffusion => rule_diffusion,
            Self::Maximum => rule_maximum,
            Self::Oscillate => rule_oscillate,
            Self::WeightedAverage => rule_weighted_average,
            Self::VonNeumann => rule_von_neumann,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MovementType {
    Static,
    Random,
    Gradient,
    AvoidCrowding,
    TraitBased,
}

impl MovementType {
    fn all() -> Vec<Self> {
        vec![
            Self::Static,
            Self::Random,
            Self::Gradient,
            Self::AvoidCrowding,
            Self::TraitBased,
        ]
    }
    
    fn name(&self) -> &str {
        match self {
            Self::Static => "Static",
            Self::Random => "Random",
            Self::Gradient => "Gradient",
            Self::AvoidCrowding => "Avoid Crowding",
            Self::TraitBased => "Trait Based",
        }
    }
}

struct CAApp {
    // Grid state
    grid: Grid,
    grid_width: usize, // usefull if the Grid object is reset
    grid_height: usize,
    
    // Simulation state
    timestep: usize,
    is_playing: bool,
    steps_per_second: f32,
    time_accumulator: f32,
    
    // Configuration
    active_mask: Vec<Vec<bool>>,
    trait_rules: Vec<RuleType>,
    movement_type: MovementType,
    
    // Visualization
    selected_trait: usize,
    cell_size: f32,
    show_values: bool,
    color_scheme: ColorScheme,
    
    // Trait names
    trait_names: [String; 9],
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColorScheme {
    Viridis,
    Plasma,
    Grayscale,
    RedBlue,
}

impl ColorScheme {
    fn all() -> Vec<Self> {
        vec![Self::Viridis, Self::Plasma, Self::Grayscale, Self::RedBlue]
    }
    
    fn name(&self) -> &str {
        match self {
            Self::Viridis => "Viridis",
            Self::Plasma => "Plasma",
            Self::Grayscale => "Grayscale",
            Self::RedBlue => "Red-Blue",
        }
    }
    
    fn map_value(&self, value: f32) -> Color32 {
        let v = value.clamp(0.0, 1.0);
        match self {
            Self::Viridis => {
                let r = (68.0 + v * (253.0 - 68.0)) as u8;
                let g = (1.0 + v * (231.0 - 1.0)) as u8;
                let b = (84.0 + v * (37.0 - 84.0)) as u8;
                Color32::from_rgb(r, g, b)
            }
            Self::Plasma => {
                let r = (13.0 + v * (240.0 - 13.0)) as u8;
                let g = (8.0 + v * (50.0 - 8.0)) as u8;
                let b = (135.0 + v * (33.0 - 135.0)) as u8;
                Color32::from_rgb(r, g, b)
            }
            Self::Grayscale => {
                let c = (v * 255.0) as u8;
                Color32::from_rgb(c, c, c)
            }
            Self::RedBlue => {
                if v < 0.5 {
                    let t = v * 2.0;
                    Color32::from_rgb(0, 0, (t * 255.0) as u8)
                } else {
                    let t = (v - 0.5) * 2.0;
                    Color32::from_rgb((t * 255.0) as u8, 0, ((1.0 - t) * 255.0) as u8)
                }
            }
        }
    }
}

impl Default for CAApp {
    fn default() -> Self {
        let grid_width = 20;
        let grid_height = 20;
        let grid = Grid::new(grid_width, grid_height);
        
        let active_mask = vec![
            vec![true, true, false],
            vec![false, true, false],
            vec![true, false, false],
        ];
        
        let trait_rules = vec![
            RuleType::Diffusion,
            RuleType::Average,
            RuleType::Conway,
            RuleType::Maximum,
            RuleType::Oscillate,
            RuleType::Average,
            RuleType::Diffusion,
            RuleType::Average,
            RuleType::Average,
        ];
        
        Self {
            grid,
            grid_width,
            grid_height,
            timestep: 0,
            is_playing: false,
            steps_per_second: 2.0,
            time_accumulator: 0.0,
            active_mask,
            trait_rules,
            movement_type: MovementType::Static,
            selected_trait: 0,
            cell_size: 30.0,
            show_values: false,
            color_scheme: ColorScheme::Viridis,
            trait_names: [
                "Energy".to_string(),
                "Confidence".to_string(),
                "Cooperation".to_string(),
                "Aggression".to_string(),
                "Stability".to_string(),
                "Mobility".to_string(),
                "Resource".to_string(),
                "Age".to_string(),
                "Adaptability".to_string(),
            ],
        }
    }
}

impl CAApp {
    fn step_simulation(&mut self) {
        // Create neighborhood masks
        let neighborhood_mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        
        let dummy_grid = Grid::new(self.grid_width, self.grid_height);
        let neighborhood_base = Neighborhood::new(
            3, 3, 1, 1, 0, 0,
            &neighborhood_mask,
            &dummy_grid,
        );
        
        // Update traits
        let mut new_cells = Vec::new();
        
        for row in 0..self.grid.height {
            let mut new_row = Vec::new();
            for col in 0..self.grid.width {
                let cell = &self.grid.cells[row][col];
                let neighborhood = Neighborhood::new_from_base(row, col, &neighborhood_base, &self.grid);
                let mut new_cell = cell.clone();
                
                for mask_row in 0..3 {
                    for mask_col in 0..3 {
                        if self.active_mask[mask_row][mask_col] {
                            let trait_idx = mask_row * 3 + mask_col;
                            let rule_fn = self.trait_rules[trait_idx].get_rule_fn();
                            let new_value = rule_fn(cell, &neighborhood, trait_idx);
                            new_cell.set_trait(trait_idx, new_value);
                        }
                    }
                }
                
                new_row.push(new_cell);
            }
            new_cells.push(new_row);
        }
        self.grid.update_cells(new_cells);
        
        // Apply movement
        let movement_fn = match self.movement_type {
            MovementType::Static => movement_static,
            MovementType::Random => movement_random,
            MovementType::Gradient => movement_gradient,
            MovementType::AvoidCrowding => movement_avoid_crowding,
            MovementType::TraitBased => movement_trait_based,
        };
        
        let nbhr_movement_mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];
        let nbhr_movement_base = Neighborhood::new(
            3, 3, 1, 1, 0, 0,
            &nbhr_movement_mask,
            &dummy_grid,
        );
        
        let moved_cells = apply_movement(movement_fn, &nbhr_movement_base, &self.grid);
        self.grid.update_cells(moved_cells);
        
        self.timestep += 1;
    }
    
    fn reset_grid(&mut self) {
        self.grid = Grid::new(self.grid_width, self.grid_height);
        self.timestep = 0;
        self.is_playing = false;
    }
    
    fn randomize_grid(&mut self) {
        self.grid.randomize();
        self.timestep = 0;
    }
}

impl eframe::App for CAApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle animation
        if self.is_playing {
            self.time_accumulator += ctx.input(|i| i.stable_dt);
            let step_duration = 1.0 / self.steps_per_second;
            
            while self.time_accumulator >= step_duration {
                self.step_simulation();
                self.time_accumulator -= step_duration;
            }
            
            ctx.request_repaint();
        }
        
        // Left panel: Controls
        egui::SidePanel::left("controls").min_width(300.0).show(ctx, |ui| {
            ui.heading("Controls");
            ui.separator();
            
            // Playback controls
            ui.label("Simulation");
            ui.horizontal(|ui| {
                if ui.button(if self.is_playing { "⏸ Pause" } else { "▶ Play" }).clicked() {
                    self.is_playing = !self.is_playing;
                }
                if ui.button("⏭ Step").clicked() {
                    self.step_simulation();
                }
                if ui.button("🔄 Reset").clicked() {
                    self.reset_grid();
                }
            });
            
            if ui.button("🎲 Randomize").clicked() {
                self.randomize_grid();
            }
            
            ui.add(egui::Slider::new(&mut self.steps_per_second, 0.1..=100.0)
                .text("Steps/sec"));
            
            ui.label(format!("Timestep: {}", self.timestep));
            
            ui.separator();
            
            // Grid size
            ui.label("Grid Configuration");
            let mut changed = false;
            changed |= ui.add(egui::Slider::new(&mut self.grid_width, 5..=250)
                .text("Width")).changed();
            changed |= ui.add(egui::Slider::new(&mut self.grid_height, 5..=250)
                .text("Height")).changed();
            if changed {
                self.reset_grid();
            }
            
            ui.separator();
            
            // Movement type
            ui.label("Movement Type");
            egui::ComboBox::from_id_salt("movement")
                .selected_text(self.movement_type.name())
                .show_ui(ui, |ui| {
                    for mt in MovementType::all() {
                        ui.selectable_value(&mut self.movement_type, mt, mt.name());
                    }
                });
            
            ui.separator();
            
            // Trait configuration
            ui.label("Active Traits");
            egui::Grid::new("trait_grid").show(ui, |ui| {
                for mask_row in 0..3 {
                    for mask_col in 0..3 {
                        let trait_idx = mask_row * 3 + mask_col;
                        let mut active = self.active_mask[mask_row][mask_col];
                        if ui.checkbox(&mut active, &self.trait_names[trait_idx]).changed() {
                            self.active_mask[mask_row][mask_col] = active;
                        }
                    }
                    ui.end_row();
                }
            });
            
            ui.separator();
            
            // Trait rules
            ui.label("Trait Rules");
            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                for trait_idx in 0..9 {
                    if self.active_mask[trait_idx / 3][trait_idx % 3] {
                        ui.horizontal(|ui| {
                            ui.label(&self.trait_names[trait_idx]);
                            egui::ComboBox::from_id_salt(format!("rule_{}", trait_idx))
                                .selected_text(self.trait_rules[trait_idx].name())
                                .show_ui(ui, |ui| {
                                    for rt in RuleType::all() {
                                        ui.selectable_value(&mut self.trait_rules[trait_idx], rt, rt.name());
                                    }
                                });
                        });
                    }
                }
            });
        });
        
        egui::SidePanel::right("stats")
            .min_width(250.0)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Statistics");
                    ui.separator();

                    // --- Visualization controls ---
                    ui.label("Visualization");
                    ui.horizontal(|ui| {
                        ui.label("Trait:");
                        egui::ComboBox::from_id_salt("trait_select")
                            .selected_text(&self.trait_names[self.selected_trait])
                            .show_ui(ui, |ui| {
                                for (idx, name) in self.trait_names.iter().enumerate() {
                                    if self.active_mask[idx / 3][idx % 3] {
                                        ui.selectable_value(&mut self.selected_trait, idx, name);
                                    }
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Colors:");
                        egui::ComboBox::from_id_salt("color_scheme")
                            .selected_text(self.color_scheme.name())
                            .show_ui(ui, |ui| {
                                for cs in ColorScheme::all() {
                                    ui.selectable_value(&mut self.color_scheme, cs, cs.name());
                                }
                            });
                    });

                    ui.add(egui::Slider::new(&mut self.cell_size, 5.0..=60.0).text("Cell Size"));
                    ui.checkbox(&mut self.show_values, "Show Values");
                    ui.separator();

                    // --- Trait Statistics (scrollable only if too long) ---
                    ui.label("Trait Statistics");
                    egui::ScrollArea::vertical()
                        .max_height(300.0) // adjust as needed
                        .show(ui, |ui| {
                            for mask_row in 0..3 {
                                for mask_col in 0..3 {
                                    if self.active_mask[mask_row][mask_col] {
                                        let trait_idx = mask_row * 3 + mask_col;
                                        let values = self.grid.get_trait_array(trait_idx);
                                        let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
                                        let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                                        let avg = values.iter().sum::<f32>() / values.len() as f32;

                                        ui.label(format!("{}:", self.trait_names[trait_idx]));
                                        ui.label(format!("  min: {:.3}, max: {:.3}", min, max));
                                        ui.label(format!("  avg: {:.3}", avg));
                                        ui.separator();
                                    }
                                }
                            }
                        });
                });
            });

        
        // Central panel: Grid visualization
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Grid - {}", self.trait_names[self.selected_trait]));
            
            egui::ScrollArea::both().show(ui, |ui| {
                let values = self.grid.get_trait_array(self.selected_trait);
                let (response, painter) = ui.allocate_painter(
                    Vec2::new(
                        self.grid_width as f32 * self.cell_size,
                        self.grid_height as f32 * self.cell_size,
                    ),
                    egui::Sense::hover(),
                );
                
                let rect = response.rect;
                
                // Draw cells
                for row in 0..self.grid_height {
                    for col in 0..self.grid_width {
                        let idx = row * self.grid_width + col;
                        let value = values[idx];
                        
                        let x = rect.min.x + col as f32 * self.cell_size;
                        let y = rect.min.y + row as f32 * self.cell_size;
                        
                        let cell_rect = Rect::from_min_size(
                            Pos2::new(x, y),
                            Vec2::new(self.cell_size, self.cell_size),
                        );
                        
                        let color = self.color_scheme.map_value(value);
                        painter.rect_filled(cell_rect, 0.0, color);
                        painter.rect_stroke(cell_rect, 0.0, Stroke::new(0.5, Color32::GRAY));
                        
                        // Draw value text if enabled
                        if self.show_values && self.cell_size > 25.0 {
                            let text = format!("{:.2}", value);
                            let text_color = if value > 0.5 {
                                Color32::BLACK
                            } else {
                                Color32::WHITE
                            };
                            painter.text(
                                cell_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                text,
                                egui::FontId::monospace(10.0),
                                text_color,
                            );
                        }
                    }
                }
            });
        });
    }
}