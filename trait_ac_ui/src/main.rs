use eframe::{egui, glow};
use egui::{Color32};
use trait_ac::grid::Grid;
use trait_ac::neighborhood::Neighborhood;
use trait_ac::rules::{RulesRegistry, Rules, RuleFn};
use trait_ac::movement::{MovementRegistry, Movements, MovementFn};
use trait_ac::utils::{semantic_traits_names, print_separator, print_active_traits};
use rayon::prelude::*;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use eframe::glow::HasContext;



fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Cellular Automata Simulator"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    
    eframe::run_native(
        "CA Simulator",
        options,
        Box::new(|cc| {
            let gl = cc.gl.as_ref().expect("Failed to get glow context").clone();
            Ok(Box::new(CAApp::new(gl)))  // Pass GL context
        }),
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

    fn to_index(&self) -> usize {
        match self {
            Self::Viridis => 0,
            Self::Plasma => 1,
            Self::Grayscale => 2,
            Self::RedBlue => 3,
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

struct GPURenderer {
    gl: Arc<glow::Context>,
    texture: glow::Texture,
    programs: [glow::Program; 4],
    vao: glow::VertexArray,
    vbo: glow::Buffer,
}

impl GPURenderer {
    fn new(gl: Arc<glow::Context>) -> Result<Self, String> {
        unsafe {
            let vertex_shader_src = include_str!("shaders/vertex.glsl");
            let fragment_shaders = [
                include_str!("shaders/viridis.glsl"),
                include_str!("shaders/plasma.glsl"),
                include_str!("shaders/grayscale.glsl"),
                include_str!("shaders/redblue.glsl"),
            ];
            
            let mut programs = [None, None, None, None];
            for i in 0..fragment_shaders.len() {
                programs[i] = Some(Self::create_program(&gl, vertex_shader_src, fragment_shaders[i])?);
            }
            
            let texture = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            
            let vao = gl.create_vertex_array()?;
            let vbo = gl.create_buffer()?;
            
            Ok(Self {
                gl,
                texture,
                programs: [programs[0].unwrap(), programs[1].unwrap(), programs[2].unwrap(), programs[3].unwrap()],
                vao,
                vbo,
            })
        }
    }
    
    unsafe fn create_program(gl: &glow::Context, vs: &str, fs: &str) -> Result<glow::Program, String> {
        let program = unsafe { gl.create_program()? };
        let v = unsafe { gl.create_shader(glow::VERTEX_SHADER)? };
        unsafe { gl.shader_source(v, vs) };
        unsafe { gl.compile_shader(v) } ;
        if unsafe { !gl.get_shader_compile_status(v) } {
            return Err(unsafe { gl.get_shader_info_log(v) });
        }
        
        let f = unsafe { gl.create_shader(glow::FRAGMENT_SHADER)? };
        unsafe { gl.shader_source(f, fs) };
        unsafe { gl.compile_shader(f) };
        if unsafe { !gl.get_shader_compile_status(f) } {
            return Err(unsafe { gl.get_shader_info_log(f) });
        }
        
        unsafe { gl.attach_shader(program, v) };
        unsafe { gl.attach_shader(program, f) };
        unsafe { gl.link_program(program) };
        
        if unsafe {!gl.get_program_link_status(program)} {
            return Err(unsafe { gl.get_program_info_log(program) });
        }
        
        unsafe { gl.delete_shader(v); }
        unsafe { gl.delete_shader(f); }
        Ok(program)
    }
    
    fn update_texture(&mut self, width: usize, height: usize, data: &[u8], resize_flag: bool) {
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            
            // Only reallocate if dimensions change
            if resize_flag {
                self.gl.tex_image_2d(
                    glow::TEXTURE_2D, 0, glow::R8 as i32,
                    width as i32, height as i32, 0,
                    glow::RED, glow::UNSIGNED_BYTE, eframe::glow::PixelUnpackData::Slice(None), // Allocate only
                );
            }

            // Fast update
            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D, 0, 
                0, 0, // x, y offset
                width as i32, height as i32,
                glow::RED, glow::UNSIGNED_BYTE,
                eframe::glow::PixelUnpackData::Slice(Some(data)),
            );
        }
    }
    
    fn paint(&self, scheme: usize, rect: egui::Rect, _screen: [f32; 2], 
            scroll_offset: egui::Vec2, content_size: egui::Vec2) {
        unsafe {
            let gl = &self.gl;
            let prog = self.programs[scheme];
            gl.use_program(Some(prog));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.uniform_1_i32(gl.get_uniform_location(prog, "u_texture").as_ref(), 0);
            
            let rect_size = rect.size();
            gl.uniform_2_f32(
                gl.get_uniform_location(prog, "u_rect_size").as_ref(),
                rect_size.x,
                rect_size.y,
            );
            
            let rect_min = rect.min;
            gl.uniform_2_f32(
                gl.get_uniform_location(prog, "u_rect_min").as_ref(),
                rect_min.x,
                rect_min.y,
            );
            
            // Calculate what portion of the texture is visible
            // Clamp to [0, 1] range to avoid repeating
            let tex_min_x = (scroll_offset.x / content_size.x).max(0.0).min(1.0);
            let tex_min_y = (scroll_offset.y / content_size.y).max(0.0).min(1.0);
            let tex_max_x = ((scroll_offset.x + rect_size.x) / content_size.x).max(0.0).min(1.0);
            let tex_max_y = ((scroll_offset.y + rect_size.y) / content_size.y).max(0.0).min(1.0);
            
            // Calculate how much of the viewport actually contains texture
            let visible_content_width = (content_size.x - scroll_offset.x).max(0.0).min(rect_size.x);
            let visible_content_height = (content_size.y - scroll_offset.y).max(0.0).min(rect_size.y);
            
            // Only render where there's actual content
            let render_max_x = rect.min.x + visible_content_width;
            let render_max_y = rect.min.y + visible_content_height;
            
            let verts: [f32; 24] = [
                rect.min.x, rect.min.y, tex_min_x, tex_min_y,
                render_max_x, rect.min.y, tex_max_x, tex_min_y,
                render_max_x, render_max_y, tex_max_x, tex_max_y,
                rect.min.x, rect.min.y, tex_min_x, tex_min_y,
                render_max_x, render_max_y, tex_max_x, tex_max_y,
                rect.min.x, render_max_y, tex_min_x, tex_max_y,
            ];
            
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&verts), glow::STREAM_DRAW);
            
            let pos = gl.get_attrib_location(prog, "a_pos").unwrap();
            let tc = gl.get_attrib_location(prog, "a_tc").unwrap();
            
            gl.enable_vertex_attrib_array(pos);
            gl.vertex_attrib_pointer_f32(pos, 2, glow::FLOAT, false, 16, 0);
            gl.enable_vertex_attrib_array(tc);
            gl.vertex_attrib_pointer_f32(tc, 2, glow::FLOAT, false, 16, 8);
            
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }
}





struct CAApp {
    // Grid state
    grid: Grid,
    grid_density: f32,
    initialisation_ranges: [(f32, f32); 9],
    next_grid: Grid,
    
    // Simulation state
    timestep: usize,
    timed_simulation: bool,
    timestep_max: usize,
    start: Instant,
    is_playing: bool,
    steps_per_second: f32,
    time_accumulator: f32,
    gpu_renderer: Arc<Mutex<Option<GPURenderer>>>,
    grayscale_buffer: Vec<u8>,
    central_panel_offset: Option<egui::Vec2>,
    last_rendered_timestep: usize,
    avg_step_time: Option<f32>,
    
    // Configuration
    active_mask: [u8; 9],
    active_traits: Vec<usize>,
    neighborhood_traits: Neighborhood,
    neighborhood_mvt: Neighborhood,
    rules_registry: RulesRegistry,
    movement_registry: MovementRegistry,
    grid_width_min: usize,
    grid_width_max: usize,
    grid_height_min: usize,
    grid_height_max: usize,
    steps_per_second_min: f32,
    steps_per_second_max: f32,
    rows_per_batch: usize,
    
    // Visualization
    selected_trait: usize,
    cell_size: f32,
    show_values: bool,
    show_values_minimum_cell_size: f32,
    color_scheme: ColorScheme,
    base_color_no_actor: f32,
    cell_size_min: f32,
    cell_size_max: f32,
    
    // Trait names
    trait_names: [String; 9],
}

impl CAApp {
    fn new(gl: Arc<glow::Context>) -> Self {
        let renderer = match GPURenderer::new(gl) {
            Ok(r) => {
                println!("✓ GPU renderer initialized");
                Some(r)
            }
            Err(e) => {
                eprintln!("✗ GPU init failed: {}", e);
                None
            }
        };

        // Configuration
        let grid_width = 500;
        let grid_height = 500;
        let steps_per_second = 1000.0;
        let grid_density = 0.5;

        let timed_simulation = false;
        let timestep_max = 100;

        let grid_width_min = 3;
        let grid_width_max = 5000;

        let grid_height_min = 3;
        let grid_height_max = 5000;

        let steps_per_second_min = 1.0;
        let steps_per_second_max = 10000.0;

        let cell_size = 1.0;
        let cell_size_min = 0.1;
        let cell_size_max = 100.0;

        let show_values = false;
        let show_values_minimum_cell_size = 20.0;

        let color_scheme = ColorScheme::Viridis;
        let base_color_no_actor = 0.0;

        let active_mask: [u8; 9] = [
            1, 1, 0,
            0, 0, 0,
            0, 0, 0,
        ];
        let initial_selected_trait = 0;

        // range at initialisation for each traits
        let initialisation_ranges = [ 
            (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
            (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
            (0.0, 1.0), (0.0, 1.0), (0.0, 1.0),
        ];

        // Define trait names
        let trait_names = semantic_traits_names();

        // Custum rules for each traits
        let rules: [RuleFn; 9] = [
                Rules::social_energy, Rules::social_influence, Rules::conway_optimized,
                Rules::conway_optimized, Rules::conway_optimized, Rules::conway_optimized,
                Rules::conway_optimized, Rules::conway_optimized, Rules::conway_optimized,
        ];
        let rules_registry = RulesRegistry::custom(rules);

        let movement_function: MovementFn = Movements::social_movement; 
        let movement_registry = MovementRegistry::custom(grid_width, grid_height, movement_function);

        // Initialize grid
        let grid = Grid::new_with_density(grid_width, grid_height, grid_density, initialisation_ranges);

        let neighborhood_traits_mask = vec![
            vec![1, 1, 1],
            vec![1, 1, 1],
            vec![1, 1, 1],
        ];

        let neighborhood_mvt_mask = vec![
            vec![1, 1, 1],
            vec![1, 1, 1],
            vec![1, 1, 1],
        ];

        let neighborhood_traits_height = neighborhood_traits_mask.len();
        let neighborhood_traits_width = neighborhood_traits_mask[0].len();
        let neighborhood_traits_center_row = (neighborhood_traits_height - 1) / 2;
        let neighborhood_traits_center_col = (neighborhood_traits_width - 1) / 2;

        let neighborhood_mvt_height = neighborhood_mvt_mask.len();
        let neighborhood_mvt_width = neighborhood_mvt_mask[0].len();
        let neighborhood_mvt_center_row = (neighborhood_mvt_height - 1) / 2;
        let neighborhood_mvt_center_col = (neighborhood_mvt_width - 1) / 2;

        let neighborhood_traits = Neighborhood::new(
            neighborhood_traits_width,
            neighborhood_traits_height,
            neighborhood_traits_center_row,
            neighborhood_traits_center_col,
            neighborhood_traits_mask,
        );

        let neighborhood_mvt = Neighborhood::new(
            neighborhood_mvt_width,
            neighborhood_mvt_height,
            neighborhood_mvt_center_row,
            neighborhood_mvt_center_col,
            neighborhood_mvt_mask,
        );

        // Pre-allocate next grid
        let next_grid = Grid {
            width: grid.width,
            height: grid.height,
            num_cells: grid.num_cells,
            data: grid.data.clone(),
            is_empty: grid.is_empty.clone(),
        };

        let rows_per_batch = std::cmp::max(1, 4000 / grid_width);

        let active_traits: Vec<usize> = active_mask
            .iter()
            .enumerate()
            .filter_map(|(i, &m)| if m != 0 { Some(i) } else { None })
            .collect();
        
        Self {
            grid,
            grid_density,
            initialisation_ranges,
            next_grid,

            timestep: 0,
            timed_simulation,
            timestep_max,
            start: Instant::now(),
            is_playing: timed_simulation,
            steps_per_second,
            time_accumulator: 0.0,
            gpu_renderer: Arc::new(Mutex::new(renderer)),
            grayscale_buffer: Vec::new(),
            central_panel_offset: None,
            last_rendered_timestep: 999999,
            avg_step_time: None,

            active_mask,
            active_traits,
            neighborhood_traits,
            neighborhood_mvt,
            rules_registry,
            movement_registry,
            grid_width_min,
            grid_width_max,
            grid_height_min,
            grid_height_max,
            steps_per_second_min,
            steps_per_second_max,
            rows_per_batch,
            
            selected_trait: initial_selected_trait,
            cell_size,
            show_values,
            show_values_minimum_cell_size,
            color_scheme,
            base_color_no_actor,
            cell_size_min,
            cell_size_max,

            trait_names,
        }
    }

    fn step_simulation(&mut self) {
        if self.timestep == 0 {
            self.start = Instant::now();
        }
        let width = self.grid.width;
 
        // Sequential over active traits (small number), parallel over rows
        for &trait_idx in &self.active_traits {
            let current = self.grid.get_trait_slice(trait_idx);
            let next_trait = self.next_grid.get_trait_slice_mut(trait_idx);
            
            // Process rows in parallel
            next_trait
                .par_chunks_mut(width)
                .enumerate()
                .for_each(|(row, next_row)| {
                    let row_offset = row * width;
                    
                    // Process in cache-friendly chunks of 64
                    for chunk_start in (0..width).step_by(64) {
                        let chunk_end = (chunk_start + 64).min(width);
                        
                        for col in chunk_start..chunk_end {
                            let idx = row_offset + col;
                            next_row[col] = if self.grid.is_empty[idx] {
                                current[idx]
                            } else {
                                self.rules_registry.apply_rule(trait_idx, row, col, &self.neighborhood_traits, &self.grid)
                            };
                        }
                    }
                });
        }

        // --- STEP 2: Movement ---
        self.movement_registry.apply_movement(
            &self.neighborhood_mvt,
            &mut self.grid,
            &mut self.next_grid,
        );
        
        self.timestep += 1;
    }
    
    fn reset_grid(&mut self) {
        self.grid = Grid::new_with_density(self.grid.width, self.grid.height, self.grid_density, self.initialisation_ranges);
        self.movement_registry.prepare(self.grid.width, self.grid.height);
        // Pre-allocate next grid
        self.next_grid = Grid {
            width: self.grid.width,
            height: self.grid.height,
            num_cells: self.grid.num_cells,
            data: self.grid.data.clone(),
            is_empty: self.grid.is_empty.clone(),
        };
        self.rows_per_batch = std::cmp::max(1, 4000 / self.grid.width);
        self.timestep = 0;
        self.time_accumulator = 0.0;
    }
    
    fn randomize_grid(&mut self) {
        self.grid.randomize();
        self.timestep = 0;
    }

    fn update_grayscale_buffer(&mut self) {
        if self.active_mask[self.selected_trait] == 0 {
            if let Some(i) = (0..9).find(|&i| self.active_mask[i] == 1) {
                self.selected_trait = i;
            } else {
                return;
            }
        }
        
        let len = self.grid.width * self.grid.height;
        let resize_flag = self.grayscale_buffer.len() != len;
        if  resize_flag {
            self.grayscale_buffer.resize(len, 0);
        }
        
        self.grayscale_buffer
            .par_chunks_mut(self.grid.width)
            .enumerate()
            .for_each(|(row, pixels)| {
                let start = row * self.grid.width;
                for (col, pixel) in pixels.iter_mut().enumerate() {
                    let idx = start + col;
                    let is_not_empty = (!self.grid.is_empty[idx]) as u8;
                    let trait_val = self.grid.get_cell_trait(row, col, self.selected_trait);
                    let offset_val = ((self.base_color_no_actor + trait_val*(1.0-self.base_color_no_actor)) * 255.0) as u8;

                    *pixel = offset_val * is_not_empty;
                }
            });
        
        // Upload to GPU
        if let Ok(mut guard) = self.gpu_renderer.lock() {
            if let Some(ref mut r) = *guard {
                r.update_texture(
                    self.grid.width,
                    self.grid.height,
                    &self.grayscale_buffer,
                    resize_flag,
                );
            }
        }

        self.last_rendered_timestep = self.timestep;
    }
}





impl eframe::App for CAApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if we've reached the maximum timestep
        if self.timed_simulation && self.timestep >= self.timestep_max {
            println!("Simulation completed at timestep {}. Closing app.", self.timestep);
            print_separator();
            self.timed_simulation = false; // to avoid printing multiple times

            println!("Configuration:");
            println!("  Grid: {}x{}", self.grid.width, self.grid.height);
            println!("  Timesteps: {}", self.timestep);
            
            // Print active traits for info
            print_active_traits(&self.active_mask, &self.trait_names, &self.rules_registry);
            
            let elapsed = self.start.elapsed();
            println!("\nExecution time: {:?}", elapsed);
            println!(
                "Performance: {:.2} timesteps/sec",
                self.timestep as f64 / elapsed.as_secs_f64()
            );
            println!(
                "Cells/sec: {:.2}M",
                (self.grid.width * self.grid.height * self.timestep) as f64 / elapsed.as_secs_f64() / 1_000_000.0
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Handle keyboard input
        ctx.input(|i| {
            // Spacebar to play/pause
            if i.key_pressed(egui::Key::Space) {
                self.is_playing = !self.is_playing;
            }
            // Delete key to reset
            if i.key_pressed(egui::Key::Delete) {
                self.reset_grid();
            }
        });

        // Handle animation
        if self.is_playing {
            self.time_accumulator += ctx.input(|i| i.stable_dt);
            let step_duration = 1.0 / self.steps_per_second;
            
            let frame_time = ctx.input(|i| i.stable_dt);
            let target_frame_time = 1.0 / 60.0;
            
            // Calculate remaining time budget based on actual frame time
            // If last frame was slow, we have less budget this frame
            let render_time_estimate = frame_time * 0.5; // Assume ~50% was rendering
            let simulation_budget = (target_frame_time - render_time_estimate).max(0.001);
            
            let estimated_step_time = self.avg_step_time.unwrap_or(0.0001);
            let max_steps = ((simulation_budget / estimated_step_time) as usize).max(1);
            
            let mut steps_taken = 0;
            let step_start = std::time::Instant::now();
            
            while self.time_accumulator >= step_duration && steps_taken < max_steps {
                self.step_simulation();
                self.time_accumulator -= step_duration;
                steps_taken += 1;
            }
            
            if steps_taken > 0 {
                let actual_step_time = step_start.elapsed().as_secs_f32() / steps_taken as f32;
                self.avg_step_time = Some(match self.avg_step_time {
                    Some(avg) => avg * 0.9 + actual_step_time * 0.1,
                    None => actual_step_time,
                });
            }
            
            if self.time_accumulator > step_duration * 2.0 {
                self.time_accumulator = 0.0;
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
            changed |= ui.add(egui::Slider::new(&mut self.grid.width, self.grid_width_min..=self.grid_width_max)
                .text("Width")).changed();
            changed |= ui.add(egui::Slider::new(&mut self.grid.height, self.grid_height_min..=self.grid_height_max)
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
                    for &name in self.movement_registry.get_all_names() {
                        if let Some(movement_fn) = self.movement_registry.get_movement_by_name(name) {
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

                        // Read from mask (1 = active, 0 = inactive)
                        let mut active = self.active_mask[trait_idx] == 1;

                        if ui
                            .checkbox(&mut active, &self.trait_names[trait_idx])
                            .changed()
                        {
                            // Write back to mask
                            self.active_mask[trait_idx] = if active { 1 } else { 0 };
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
                    // Skip inactive traits
                    if self.active_mask[trait_idx] == 0 {
                        continue;
                    }

                    ui.horizontal(|ui| {
                        ui.label(&self.trait_names[trait_idx]);

                        egui::ComboBox::from_id_salt(format!("rule_{}", trait_idx))
                            .selected_text(self.rules_registry.get_rule_name(trait_idx))
                            .show_ui(ui, |ui| {
                                for &name in self.rules_registry.get_all_names() {
                                    if let Some(rule_fn) =
                                        self.rules_registry.get_rule_by_name(name)
                                    {
                                        let is_selected = self
                                            .rules_registry
                                            .is_stored_function(trait_idx, rule_fn);

                                        if ui
                                            .selectable_label(is_selected, name)
                                            .clicked()
                                        {
                                            self.rules_registry.set_rule(trait_idx, rule_fn);
                                        }
                                    }
                                }
                            });
                    });
                }
            });

        });

        if self.active_mask[self.selected_trait] == 0 { // update currenlty displayed trait if needed
            if let Some(new_idx) = (0..9).find(|&i| self.active_mask[i] == 1) {
                self.selected_trait = new_idx;
                flag_update_texture = true;
            }
        }
        
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
                                for trait_idx in 0..9 {
                                    if self.active_mask[trait_idx] == 0 {
                                        continue;
                                    }

                                    ui.selectable_value(
                                        &mut self.selected_trait,
                                        trait_idx,
                                        &self.trait_names[trait_idx],
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
                        egui::Slider::new(&mut self.cell_size, self.cell_size_min..=self.cell_size_max)
                            .text("Cell Size")
                    );
                    
                    ui.checkbox(&mut self.show_values, "Show Values");
                    ui.separator();

                    // --- Trait Statistics ---
                    ui.label("Statistics");
                    let fill_percentage = self.grid.get_fill_percentage();
                    ui.label(format!("  density: {:.3}", fill_percentage));
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(300.0) // adjust as needed
                        .show(ui, |ui| {
                            for trait_idx in 0..9 {
                                // Skip inactive traits
                                if self.active_mask[trait_idx] == 0 {
                                    continue;
                                }

                                let values = self.grid.get_trait_slice(trait_idx);

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
            if self.last_rendered_timestep != self.timestep || flag_update_texture {
                self.update_grayscale_buffer();
            }

            // Persistent scroll ID
            let scroll_id = ui.make_persistent_id("grid_scroll");

            // Calculate content size (logical size of the entire grid)
            let content_size = egui::vec2(
                self.grid.width as f32 * self.cell_size,
                self.grid.height as f32 * self.cell_size,
            );

            // ScrollArea configuration
            let mut scroll_area = egui::ScrollArea::both()
                .id_salt(scroll_id)
                .auto_shrink([false, false]);

            if let Some(offset) = self.central_panel_offset.take() {
                scroll_area = scroll_area.scroll_offset(offset);
            }

            let scroll_output = scroll_area.show(ui, |ui| {
                // Allocate the full content size to enable scrolling
                let (response, _painter) = ui.allocate_painter(content_size, egui::Sense::drag());

                response
            });

            // Now get the scroll offset AFTER the scroll area has updated
            let current_scroll = scroll_output.state.offset;
            let viewport_rect = scroll_output.inner_rect;
            
            // Render the visible portion using a separate paint callback
            let painter = ui.painter_at(viewport_rect);
            
            let screen = ctx.screen_rect().size();
            let screen_arr = [screen.x, screen.y];
            let scheme = self.color_scheme.to_index();
            let renderer = self.gpu_renderer.clone();
            
            let cb = egui::PaintCallback {
                rect: viewport_rect,
                callback: Arc::new(egui_glow::CallbackFn::new(
                    move |_info, _painter| {
                        if let Ok(guard) = renderer.lock() {
                            if let Some(ref r) = *guard {
                                r.paint(scheme, viewport_rect, screen_arr, 
                                    current_scroll, content_size);
                            }
                        }
                    }
                )),
            };
            
            painter.add(cb);

            let image_response = scroll_output.inner;
            let viewport_rect = scroll_output.inner_rect;
            let pointer_over = ui.rect_contains_pointer(viewport_rect);

            // Draw values on top if zoomed in enough
            if self.show_values && self.cell_size >= self.show_values_minimum_cell_size {
                let painter = ui.painter();
                let values = self.grid.get_trait_slice(self.selected_trait);

                // Calculate visible cell range based on scroll offset
                let scroll_offset = scroll_output.state.offset;

                let start_col = (scroll_offset.x / self.cell_size).floor() as usize;
                let start_row = (scroll_offset.y / self.cell_size).floor() as usize;
                let visible_cols = (viewport_rect.width() / self.cell_size).ceil() as usize + 1;
                let visible_rows = (viewport_rect.height() / self.cell_size).ceil() as usize + 1;
                let end_col = (start_col + visible_cols).min(self.grid.width);
                let end_row = (start_row + visible_rows).min(self.grid.height);

                // Capture needed values to avoid borrowing issues
                let grid_width = self.grid.width;
                let cell_size = self.cell_size;

                // Create index pairs for only visible cells
                let visible_cells: Vec<_> = (start_row..end_row)
                    .flat_map(|row| {
                        (start_col..end_col).map(move |col| (row, col))
                    })
                    .collect();

                // Process visible cells in parallel
                let text_data: Vec<_> = visible_cells
                    .par_iter()
                    .filter_map(|&(row, col)| {
                        let idx = row * grid_width + col;
                        let value = values[idx];

                        // Skip empty cells
                        if self.grid.is_cell_empty(row, col) {
                            return None;
                        }

                        // Calculate cell center - using image_response.rect instead of viewport_rect
                        // because that's where the actual grid is drawn
                        let cell_x = image_response.rect.min.x + (col as f32 + 0.5) * cell_size;
                        let cell_y = image_response.rect.min.y + (row as f32 + 0.5) * cell_size;
                        let pos = egui::pos2(cell_x, cell_y);

                        let cell_color = self.color_scheme.map_value(
                            value,
                            false,
                            self.base_color_no_actor,
                        );

                        // Calculate luminance (perceived brightness)
                        let luminance = 0.2126 * cell_color[0] as f32
                                    + 0.7152 * cell_color[1] as f32
                                    + 0.0722 * cell_color[2] as f32;

                        // Choose white text for dark backgrounds, black for light backgrounds
                        let text_color = if luminance < 128.0 {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::BLACK
                        };

                        Some((pos, value, text_color))
                    })
                    .collect();

                // Draw all text (must be done on main thread due to painter)
                let font_size = (self.cell_size * 0.4).min(14.0);
                for (pos, value, text_color) in text_data {
                    painter.text(
                        pos,
                        egui::Align2::CENTER_CENTER,
                        format!("{:.2}", value),
                        egui::FontId::proportional(font_size),
                        text_color,
                    );
                }
            }

            // PAN via mouse drag
            if pointer_over && image_response.dragged() {
                let delta = image_response.drag_delta();

                if let Some(mut state) = egui::scroll_area::State::load(ctx, scroll_id) {
                    state.offset -= delta;
                    state.store(ctx, scroll_id);
                }

                ctx.request_repaint();
            }

            // PAN via wheel / trackpad (no Ctrl)
            let scroll = ctx.input(|i| i.raw_scroll_delta);

            if pointer_over && scroll != egui::Vec2::ZERO {
                let ctrl = ctx.input(|i| i.modifiers.ctrl);

                if !ctrl {
                    if let Some(mut state) = egui::scroll_area::State::load(ctx, scroll_id) {
                        state.offset -= scroll;
                        state.store(ctx, scroll_id);
                    }

                    ctx.request_repaint();
                }
            }

            // ZOOM via Ctrl + scroll (cursor anchored)
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
                        .clamp(self.cell_size_min, self.cell_size_max);

                    if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                        let rel = pos - viewport_rect.min;

                        let state = scroll_output.state.clone();

                        let old_world_pos = state.offset + rel;
                        let zoom_ratio = self.cell_size / old_cell;
                        let new_world_pos = old_world_pos * zoom_ratio;

                        let mut new_offset = new_world_pos - rel;

                        let content_size = egui::vec2(
                            self.grid.width as f32 * self.cell_size,
                            self.grid.height as f32 * self.cell_size,
                        );
                        let visible_size = viewport_rect.size();

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