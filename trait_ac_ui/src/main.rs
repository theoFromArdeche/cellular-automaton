use eframe::egui;
use egui::{Color32};
use trait_ac::grid::Grid;
use trait_ac::neighborhood::{Neighborhood, NeighborhoodSettings};
use trait_ac::rules::{RulesRegistry, Rules, RuleFn};
use trait_ac::movement::{MovementRegistry, Movements, MovementFn};
use trait_ac::utils::{semantic_traits_names};
use rayon::prelude::*;



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
    
    fn map_value(&self, value: f32, is_empty: bool, base_color_no_actor: f32) -> Color32 {
        let v;
        if !is_empty {
            v = (base_color_no_actor + value*(1.0-base_color_no_actor)).clamp(0.0, 1.0);
        } else {
            v = 0.0;
        }
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





struct CAApp {
    // Grid state
    grid: Grid,
    grid_width: usize, // usefull if the Grid object is reset
    grid_height: usize,
    grid_density: f32,
    
    // Simulation state
    timestep: usize,
    is_playing: bool,
    steps_per_second: f32,
    time_accumulator: f32,
    grid_texture: Option<egui::TextureHandle>,
    central_panel_offset: Option<egui::Vec2>,
    last_rendered_timestep: usize,
    
    // Configuration
    active_traits: Vec<usize>,
    neighborhood_traits_settings: NeighborhoodSettings,
    neighborhood_mvt_settings: NeighborhoodSettings,
    rules_registry: RulesRegistry,
    movement_registry: MovementRegistry,
    grid_width_min: usize,
    grid_width_max: usize,
    grid_height_min: usize,
    grid_height_max: usize,
    steps_per_second_min: f32,
    steps_per_second_max: f32,
    
    // Visualization
    selected_trait: usize,
    cell_size: f32,
    show_values: bool,
    color_scheme: ColorScheme,
    base_color_no_actor: f32,
    min_cell_size: f32,
    max_cell_size: f32,
    
    // Trait names
    trait_names: [String; 9],
}

impl Default for CAApp {
    fn default() -> Self {
        // Configuration
        let grid_width = 500;
        let grid_height = 500;
        let steps_per_second = 25.0;
        let grid_density = 1.0;


        let grid_width_min = 3;
        let grid_width_max = 1000;

        let grid_height_min = 3;
        let grid_height_max = 1000;

        let steps_per_second_min = 1.0;
        let steps_per_second_max = 200.0;

        let active_traits: Vec<usize> = vec![0, 1, 2, 3, 4];
        let initial_selected_trait = active_traits[0];

        // Neighborhood mask
        let neighborhood_traits_mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];

        let neighborhood_mvt_mask = vec![
            vec![true, true, true],
            vec![true, true, true],
            vec![true, true, true],
        ];

        let neighborhood_traits_height = neighborhood_traits_mask.len();
        let neighborhood_traits_width = neighborhood_traits_mask[0].len();
        let neighborhood_traits_center_row = (neighborhood_traits_height - 1) / 2;
        let neighborhood_traits_center_col = (neighborhood_traits_width - 1) / 2;

        let neighborhood_mvt_height = neighborhood_mvt_mask.len();
        let neighborhood_mvt_width = neighborhood_mvt_mask[0].len();
        let neighborhood_mvt_center_row = (neighborhood_mvt_height - 1) / 2;
        let neighborhood_mvt_center_col = (neighborhood_mvt_width - 1) / 2;

        // Initialize grid
        let grid = Grid::new_with_density(grid_width, grid_height, grid_density);


        // Default neighborhood
        let neighborhood_traits_settings = NeighborhoodSettings::new(
            neighborhood_traits_width,
            neighborhood_traits_height,
            neighborhood_traits_center_row,
            neighborhood_traits_center_col,
            neighborhood_traits_mask,
        );

        let neighborhood_mvt_settings = NeighborhoodSettings::new(
            neighborhood_mvt_width,
            neighborhood_mvt_height,
            neighborhood_mvt_center_row,
            neighborhood_mvt_center_col,
            neighborhood_mvt_mask,
        );

        // Define trait names
        let trait_names = semantic_traits_names();

        // Create custom rule set
        let rules: [RuleFn; 9] = [
                Rules::conway, Rules::conway, Rules::conway,
                Rules::conway, Rules::conway, Rules::conway,
                Rules::conway, Rules::conway, Rules::conway,
        ];
        let rules_registry = RulesRegistry::custom(rules);

        let movement_function: MovementFn = Movements::static_movement; 
        let movement_registry = MovementRegistry::custom(movement_function);

        
        Self {
            grid,
            grid_width,
            grid_height,
            grid_density,

            timestep: 0,
            is_playing: false,
            steps_per_second,
            time_accumulator: 0.0,
            grid_texture: None,
            central_panel_offset: None,
            last_rendered_timestep: 999999,

            active_traits,
            neighborhood_traits_settings,
            neighborhood_mvt_settings,
            rules_registry,
            movement_registry,
            grid_width_min,
            grid_width_max,
            grid_height_min,
            grid_height_max,
            steps_per_second_min,
            steps_per_second_max,
            
            selected_trait: initial_selected_trait,
            cell_size: 3.0,
            show_values: false,
            color_scheme: ColorScheme::Viridis,
            base_color_no_actor: 0.1,
            min_cell_size: 1.0,
            max_cell_size: 100.0,

            trait_names,
        }
    }
}

impl CAApp {
    fn step_simulation(&mut self) {

        let mut new_cells: Vec<Vec<_>> = (0..self.grid.height)
            .into_par_iter()
            .map(|row| {
                let mut new_row = Vec::with_capacity(self.grid.width);
                for col in 0..self.grid.width {
                    let cell = &self.grid.cells[row][col];
                    
                    if cell.is_empty() {
                        new_row.push(cell.clone());
                        continue;
                    }
                    
                    let mut new_cell = cell.clone();
                    let neighborhood_traits = Neighborhood::new_from_settings(row, col, &self.neighborhood_traits_settings, &self.grid);

                    // Update only active traits
                    for &trait_idx in &self.active_traits {
                        let new_value = self.rules_registry.apply_rule(cell, &neighborhood_traits, trait_idx);
                        new_cell.set_trait(trait_idx, new_value);
                    }

                    new_row.push(new_cell);
                }
                new_row
            })
            .collect();

        self.grid.update_cells_fast(&mut new_cells);

        // Step 2: Apply movement
        let mut moved_cells = self.movement_registry.apply_movement(&self.neighborhood_mvt_settings, &self.grid);
        self.grid.update_cells_fast(&mut moved_cells);
        
        self.timestep += 1;
    }
    
    fn reset_grid(&mut self) {
        self.grid = Grid::new_with_density(self.grid_width, self.grid_height, self.grid_density);
        self.timestep = 0;
        self.is_playing = false;
    }
    
    fn randomize_grid(&mut self) {
        self.grid.randomize();
        self.timestep = 0;
    }

    fn update_grid_texture(&mut self, ctx: &egui::Context) {
        let values = self.grid.get_trait_array(self.selected_trait);
        let width = self.grid_width;
        let height = self.grid_height;

        // Allocate once, fixed size
        let mut pixels = vec![egui::Color32::BLACK; width * height];

        pixels
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(row, row_pixels)| {
                let grid_row = &self.grid.cells[row];
                let base = row * width;

                for col in 0..width {
                    let idx = base + col;
                    let is_empty = grid_row[col].is_empty();
                    row_pixels[col] = self.color_scheme.map_value(
                        values[idx],
                        is_empty,
                        self.base_color_no_actor,
                    );
                }
            });

        let image = egui::ColorImage {
            size: [width, height],
            pixels,
        };

        self.grid_texture = Some(ctx.load_texture(
            "grid_tex",
            image,
            egui::TextureOptions::NEAREST,
        ));

        self.last_rendered_timestep = self.timestep;
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

        let mut flag_update_texture = false;
        
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
            
            ui.add(egui::Slider::new(&mut self.steps_per_second, self.steps_per_second_min..=self.steps_per_second_max)
                .text("Steps/sec"));
            
            ui.label(format!("Timestep: {}", self.timestep));
            
            ui.separator();
            
            // Grid size
            ui.label("Grid Configuration");
            let mut changed = false;
            changed |= ui.add(egui::Slider::new(&mut self.grid_width, self.grid_width_min..=self.grid_width_max)
                .text("Width")).changed();
            changed |= ui.add(egui::Slider::new(&mut self.grid_height, self.grid_height_min..=self.grid_height_max)
                .text("Height")).changed();
            changed |= ui.add(egui::Slider::new(&mut self.grid_density, 0.01..=1.0)
                .text("Density")).changed();
            if changed {
                flag_update_texture=true;
                self.reset_grid();
            }
            
            ui.separator();
            
            // Movement type
            ui.label("Movement Type");
            egui::ComboBox::from_id_salt("movement")
                .selected_text(self.movement_registry.get_movement_name())
                .show_ui(ui, |ui| {
                    for &name in MovementRegistry::get_all_names() {
                        if let Some(movement_fn) = MovementRegistry::get_movement_by_name(name) {
                            let is_selected = self.movement_registry.is_stored_function(movement_fn);
                            if ui.selectable_label(is_selected, name).clicked() {
                                self.movement_registry.set_movement_function(movement_fn);
                            }
                        }
                    }
                });

            ui.separator();

            // Active Traits configuration
            ui.label("Active Traits");
            egui::Grid::new("trait_grid").show(ui, |ui| {
                for mask_row in 0..3 {
                    for mask_col in 0..3 {
                        let trait_idx = mask_row * 3 + mask_col;
                        let mut active = self.active_traits.contains(&trait_idx);
                        if ui.checkbox(&mut active, &self.trait_names[trait_idx]).changed() {
                            if active {
                                // Add trait if not already present
                                if !self.active_traits.contains(&trait_idx) {
                                    self.active_traits.push(trait_idx);
                                }
                            } else {
                                // Remove trait
                                self.active_traits.retain(|&idx| idx != trait_idx);
                            }
                        }
                    }
                    ui.end_row();
                }
            });

            ui.separator();

            // Trait rules
            ui.label("Trait Rules");
            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                for &trait_idx in &self.active_traits {
                    ui.horizontal(|ui| {
                        ui.label(&self.trait_names[trait_idx]);
                        egui::ComboBox::from_id_salt(format!("rule_{}", trait_idx))
                            .selected_text(self.rules_registry.get_rule_name(trait_idx))
                            .show_ui(ui, |ui| {
                                for &name in RulesRegistry::get_all_names() {
                                    if let Some(rule_fn) = RulesRegistry::get_rule_by_name(name) {
                                        let is_selected = self.rules_registry.is_stored_function(trait_idx, rule_fn);
                                        if ui.selectable_label(is_selected, name).clicked() {
                                            self.rules_registry.set_rule(trait_idx, rule_fn);
                                        }
                                    }
                                }
                            });
                    });
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
                                for &trait_idx in &self.active_traits {
                                    ui.selectable_value(
                                        &mut self.selected_trait, 
                                        trait_idx, 
                                        &self.trait_names[trait_idx]
                                    );
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Colors:");
                        egui::ComboBox::from_id_salt("color_scheme")
                            .selected_text(self.color_scheme.name())
                            .show_ui(ui, |ui| {
                                for cs in ColorScheme::all() {
                                    if ui.selectable_value(&mut self.color_scheme, cs, cs.name()).changed() {
                                        flag_update_texture = true;
                                    }
                                }
                            });
                    });
                    let base_color_changed = ui.add(
                        egui::Slider::new(&mut self.base_color_no_actor, 0.0..=0.5).text("Empty cell base color")
                    ).changed();
                    if base_color_changed {
                        flag_update_texture = true;
                    }

                    ui.add(
                        egui::Slider::new(&mut self.cell_size, self.min_cell_size..=self.max_cell_size)
                            .text("Cell Size")
                    );
                    
                    ui.checkbox(&mut self.show_values, "Show Values");
                    ui.separator();

                    // --- Trait Statistics (scrollable only if too long) ---
                    ui.label("Trait Statistics");
                    egui::ScrollArea::vertical()
                        .max_height(300.0) // adjust as needed
                        .show(ui, |ui| {
                            for &trait_idx in &self.active_traits {
                                let values = self.grid.get_trait_array(trait_idx);
                                let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
                                let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                                let avg = values.iter().sum::<f32>() / values.len() as f32;
                                ui.label(format!("{}:", self.trait_names[trait_idx]));
                                ui.label(format!("  min: {:.3}, max: {:.3}", min, max));
                                ui.label(format!("  avg: {:.3}", avg));
                                ui.separator();
                            }
                        });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!(
                "Grid - {}",
                self.trait_names[self.selected_trait]
            ));

            // Update texture only when needed
            if self.grid_texture.is_none()
                || self.last_rendered_timestep != self.timestep
                || flag_update_texture
            {
                self.update_grid_texture(ctx);
            }

            let Some(texture) = &self.grid_texture else { return };

            // --------------------------------------------------
            // Persistent scroll ID (CRITICAL)
            // --------------------------------------------------
            let scroll_id = ui.make_persistent_id("grid_scroll");

            // --------------------------------------------------
            // ScrollArea
            // --------------------------------------------------
            let mut scroll_area = egui::ScrollArea::both()
                .id_salt(scroll_id)
                .auto_shrink([false, false]);

            if let Some(offset) = self.central_panel_offset.take() {
                scroll_area = scroll_area.scroll_offset(offset);
            }

            let scroll_output = scroll_area.show(ui, |ui| {
                let size = egui::vec2(
                    self.grid_width as f32 * self.cell_size,
                    self.grid_height as f32 * self.cell_size,
                );

                ui.add(
                    egui::Image::from_texture(texture)
                        .fit_to_exact_size(size)
                        .sense(egui::Sense::drag()),
                )
            });


            let image_response = scroll_output.inner;
            let rect = scroll_output.inner_rect;
            let pointer_over = ui.rect_contains_pointer(rect);

            // --------------------------------------------------
            // PAN via mouse drag
            // --------------------------------------------------
            if pointer_over && image_response.dragged() {
                let delta = image_response.drag_delta();

                if let Some(mut state) =
                    egui::scroll_area::State::load(ctx, scroll_id)
                {
                    state.offset -= delta;
                    state.store(ctx, scroll_id);
                }

                ctx.request_repaint();
            }

            // --------------------------------------------------
            // PAN via wheel / trackpad (no Ctrl)
            // --------------------------------------------------
            let scroll = ctx.input(|i| i.raw_scroll_delta);

            if pointer_over && scroll != egui::Vec2::ZERO {
                let ctrl = ctx.input(|i| i.modifiers.ctrl);

                if !ctrl {
                    if let Some(mut state) =
                        egui::scroll_area::State::load(ctx, scroll_id)
                    {
                        state.offset -= scroll;
                        state.store(ctx, scroll_id);
                    }

                    ctx.request_repaint();
                }
            }

            // --------------------------------------------------
            // ZOOM via Ctrl + scroll (cursor anchored)
            // --------------------------------------------------
            if pointer_over {
                let zoom_scroll = ctx.input(|i| {
                    if i.modifiers.ctrl {
                        i.raw_scroll_delta.y
                    } else {
                        0.0
                    }
                });

                if zoom_scroll != 0.0 {
                    let old_cell = self.cell_size;

                    let zoom_factor = (1.0 + zoom_scroll * 0.001)
                        .clamp(0.9, 1.1);

                    self.cell_size = (self.cell_size * zoom_factor)
                        .clamp(self.min_cell_size, self.max_cell_size);

                    if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                        let rel = pos - rect.min;

                        let state = scroll_output.state.clone();

                        let old_world_pos = state.offset + rel;
                        let zoom_ratio = self.cell_size / old_cell;
                        let new_world_pos = old_world_pos * zoom_ratio;

                        let mut new_offset = new_world_pos - rel;

                        let content_size = egui::vec2(
                            self.grid_width as f32 * self.cell_size,
                            self.grid_height as f32 * self.cell_size,
                        );
                        let visible_size = rect.size();

                        new_offset.x = new_offset.x.clamp(
                            0.0,
                            (content_size.x - visible_size.x).max(0.0),
                        );
                        new_offset.y = new_offset.y.clamp(
                            0.0,
                            (content_size.y - visible_size.y).max(0.0),
                        );

                        self.central_panel_offset = Some(new_offset);
                    }

                    ctx.request_repaint();
                }
            }
        });
    }
}