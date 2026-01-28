# Benchmark Diffs Summary

Generated: 2026-01-28 11:02:55

## Version 0

### Commit Info

<details>
<summary>Click to expand version 0 info</summary>

```
Version: 0
Path: perf-improve-0
Generated: 2026-01-28 11:02:55

==================== CURRENT COMMIT (HEAD) ====================
Commit: d3c5067cfac9d4a82f2ff801d97d6a6ea58ac2ca
Short:  d3c5067
Author: theo <theodeygas@gmail.com>
Date:   2025-12-19 09:27:47 +0100
Msg:    update readme.md

==================== UNSTAGED CHANGES ====================
 trait_ac/src/main.rs    | 77 ++++++++++++++++++++++----------------
 trait_ac/src/utils.rs   |  5 ++-
 trait_ac_ui/src/main.rs | 99 +++++++++++++++++++++++++++++++++++++------------
 3 files changed, 124 insertions(+), 57 deletions(-)
```

</details>

### Code Changes

<details>
<summary>Click to expand version 0 diff</summary>

```diff
# Git diff for Version 0
# Unstaged changes compared to HEAD
# Generated: 2026-01-28 11:02:55
# HEAD Commit: d3c5067

diff --git a/trait_ac/src/main.rs b/trait_ac/src/main.rs
index 8b8fa5f..247da56 100644
--- a/trait_ac/src/main.rs
+++ b/trait_ac/src/main.rs
@@ -1,22 +1,23 @@
 use trait_ac::neighborhood::Neighborhood;
 use trait_ac::grid::Grid;
-use trait_ac::rules::{RuleSet, rule_static, rule_average, rule_conway, rule_diffusion, rule_maximum, rule_oscillate, rule_weighted_average, rule_von_neumann};
-use trait_ac::movement::{apply_movement, movement_static, movement_random, movement_gradient, movement_avoid_crowding, movement_trait_based};
-use trait_ac::utils::{print_active_traits, print_statistics, print_separator, semantic_trait_names};
+use trait_ac::rules::{RuleSet, rule_conway};
+use trait_ac::movement::{apply_movement, movement_static};
+use trait_ac::utils::{print_active_traits, print_separator, semantic_trait_names};
+use std::time::Instant;
 
 fn main() {
     println!("=== Modular Cellular Automata Simulation ===\n");
 
     // Configuration
-    let grid_height = 10;
-    let grid_width = 10;
-    let timesteps = 5;
+    let grid_height = 3000;
+    let grid_width = 3000;
+    let timesteps = 100;
 
     // Define which traits are active
     let active_mask = vec![
-        vec![true,  true,  false],  // Traits 0, 1 active
-        vec![false, true,  false],  // Trait 4 active
-        vec![true,  false, false],  // Trait 6 active
+        vec![true,  false, false],  // Traits 0, 1 active
+        vec![false, false, false],  // Trait 4 active
+        vec![false, false, false],  // Trait 6 active
     ];
 
     // True = this neighbor position affects the cell
@@ -44,7 +45,7 @@ fn main() {
     let nbhr_movement_center_col = (nbhr_movement_width-1)/2;
 
     // Initialize grid
-    let mut grid = Grid::new_with_density(grid_width, grid_height, 0.1);
+    let mut grid = Grid::new_with_density(grid_width, grid_height, 1.0);
 
     // Default neighborhood
     let dummy_grid = Grid::new(grid_width, grid_height); // can't use the normal grid because of the lifetime
@@ -74,15 +75,15 @@ fn main() {
     // Create custom rule set
     // Traits: [Energy, Confidence, Cooperation, Aggression, Stability, Mobility, Resource, Age, Adaptability]
     let ruleset = RuleSet::custom([
-        rule_diffusion,    // 0: Energy diffuses
-        rule_average,      // 1: Confidence averages
+        rule_conway,    // 0: Energy diffuses
+        rule_conway,      // 1: Confidence averages
         rule_conway,       // 2: Cooperation (not active)
-        rule_maximum,      // 3: Aggression (not active)
-        rule_oscillate,    // 4: Stability oscillates
-        rule_average,      // 5: Mobility (not active)
-        rule_diffusion,    // 6: Resource diffuses
-        rule_average,      // 7: Age (not active)
-        rule_average,      // 8: Adaptability (not active)
+        rule_conway,      // 3: Aggression (not active)
+        rule_conway,    // 4: Stability oscillates
+        rule_conway,      // 5: Mobility (not active)
+        rule_conway,    // 6: Resource diffuses
+        rule_conway,      // 7: Age (not active)
+        rule_conway,      // 8: Adaptability (not active)
     ]);
  
     // Choose movement function
@@ -91,23 +92,24 @@ fn main() {
     println!("Configuration:");
     println!("  Grid: {}x{}", grid_width, grid_height);
     println!("  Timesteps: {}", timesteps);
-    println!("  Active traits: {:?}", 
-        active_mask.iter()
-            .enumerate()
-            .flat_map(|(r, row)| row.iter().enumerate().filter(|&(_, v)| *v).map(move |(c, _)| r * 3 + c))
-            .collect::<Vec<_>>()
-    );
+    // println!("  Active traits: {:?}", 
+    //     active_mask.iter()
+    //         .enumerate()
+    //         .flat_map(|(r, row)| row.iter().enumerate().filter(|&(_, v)| *v).map(move |(c, _)| r * 3 + c))
+    //         .collect::<Vec<_>>()
+    // );
 
     // Initial state
-    print_separator();
-    println!("INITIAL STATE (t=0)");
+    // print_separator();
+    // println!("INITIAL STATE (t=0)");
     print_active_traits(&grid, &active_mask, &trait_names);
-    print_statistics(&grid, &active_mask);
+    // print_statistics(&grid, &active_mask);
 
     // Simulation loop
-    for t in 1..=timesteps {
-        print_separator();
-        println!("TIMESTEP {}", t);
+    let start = Instant::now();
+    for _t in 1..=timesteps {
+        // print_separator();
+        // println!("TIMESTEP {}", t);
 
         // Step 1: Update all active traits
         let mut new_cells = Vec::new();
@@ -144,10 +146,21 @@ fn main() {
         grid.update_cells(moved_cells);
 
         // Print results
-        print_active_traits(&grid, &active_mask, &trait_names);
-        print_statistics(&grid, &active_mask);
+        // print_active_traits(&grid, &active_mask, &trait_names);
+        // print_statistics(&grid, &active_mask);
     }
 
     print_separator();
     println!("\nSimulation complete!");
+    
+    let elapsed = start.elapsed();
+    println!("Execution time: {:?}", elapsed);
+    println!(
+        "Performance: {:.2} timesteps/sec",
+        timesteps as f64 / elapsed.as_secs_f64()
+    );
+    println!(
+        "Cells/sec: {:.2}M",
+        (grid_width * grid_height * timesteps) as f64 / elapsed.as_secs_f64() / 1_000_000.0
+    );
 }
\ No newline at end of file
diff --git a/trait_ac/src/utils.rs b/trait_ac/src/utils.rs
index c8211f4..46adaf8 100644
--- a/trait_ac/src/utils.rs
+++ b/trait_ac/src/utils.rs
@@ -22,12 +22,13 @@ pub fn print_trait_array(grid: &Grid, trait_index: usize, trait_name: &str) {
 }
 
 /// Print all active trait arrays
-pub fn print_active_traits(grid: &Grid, active_mask: &[Vec<bool>], trait_names: &[String; 9]) {
+pub fn print_active_traits(_grid: &Grid, active_mask: &[Vec<bool>], trait_names: &[String; 9]) {
+    println!("  Active traits:");
     for mask_row in 0..3 {
         for mask_col in 0..3 {
             if active_mask[mask_row][mask_col] {
                 let trait_index = mask_row * 3 + mask_col;
-                print_trait_array(grid, trait_index, &trait_names[trait_index]);
+                println!("    {}: {} (rule: Conway)", trait_index, &trait_names[trait_index]);
             }
         }
     }
diff --git a/trait_ac_ui/src/main.rs b/trait_ac_ui/src/main.rs
index c0cd015..6fdf25f 100644
--- a/trait_ac_ui/src/main.rs
+++ b/trait_ac_ui/src/main.rs
@@ -4,6 +4,8 @@ use trait_ac::grid::Grid;
 use trait_ac::neighborhood::Neighborhood;
 use trait_ac::rules::{rule_static, rule_average, rule_conway, rule_diffusion, rule_maximum, rule_oscillate, rule_weighted_average, rule_von_neumann};
 use trait_ac::movement::{apply_movement, movement_static, movement_random, movement_gradient, movement_avoid_crowding, movement_trait_based};
+use std::time::Instant;
+use trait_ac::utils::{print_separator, print_active_traits};
 
 fn main() -> eframe::Result<()> {
     let options = eframe::NativeOptions {
@@ -112,7 +114,10 @@ struct CAApp {
     grid_density: f32,
     
     // Simulation state
+    initialized: bool,
     timestep: usize,
+    timestep_max: usize,
+    start: Instant,
     is_playing: bool,
     steps_per_second: f32,
     time_accumulator: f32,
@@ -194,27 +199,27 @@ impl ColorScheme {
 
 impl Default for CAApp {
     fn default() -> Self {
-        let grid_width = 20;
-        let grid_height = 20;
-        let grid_density = 0.33;
+        let grid_width = 3000;
+        let grid_height = 3000;
+        let grid_density = 1.0;
         let grid = Grid::new_with_density(grid_width, grid_height, grid_density);
         
         let active_mask = vec![
-            vec![true, true, false],
-            vec![false, true, false],
             vec![true, false, false],
+            vec![false, false, false],
+            vec![false, false, false],
         ];
         
         let trait_rules = vec![
-            RuleType::Diffusion,
-            RuleType::Average,
             RuleType::Conway,
-            RuleType::Maximum,
-            RuleType::Oscillate,
-            RuleType::Average,
-            RuleType::Diffusion,
-            RuleType::Average,
-            RuleType::Average,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
         ];
         
         Self {
@@ -222,15 +227,18 @@ impl Default for CAApp {
             grid_width,
             grid_height,
             grid_density,
+            initialized: false,
             timestep: 0,
-            is_playing: false,
-            steps_per_second: 2.0,
+            timestep_max: 100,
+            start: Instant::now(),
+            is_playing: true,
+            steps_per_second: 10000.0,
             time_accumulator: 0.0,
             active_mask,
             trait_rules,
             movement_type: MovementType::Static,
             selected_trait: 0,
-            cell_size: 30.0,
+            cell_size: 0.397,
             show_values: false,
             color_scheme: ColorScheme::Viridis,
             base_color_no_actor: 0.1,
@@ -337,14 +345,59 @@ impl CAApp {
 
 impl eframe::App for CAApp {
     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
-        // Handle animation
-        if self.is_playing {
+        if self.timestep >= self.timestep_max {
+            println!("Simulation completed at timestep {}. Closing app.", self.timestep);
+            print_separator();
+            self.timestep_max = 200; // to avoid printing multiple times
+
+            println!("Configuration:");
+            println!("  Grid: {}x{}", self.grid_width, self.grid_height);
+            println!("  Timesteps: {}", self.timestep);
+
+            print_active_traits(&self.grid, &self.active_mask, &self.trait_names);
+            
+            let elapsed = self.start.elapsed();
+            println!("\nExecution time: {:?}", elapsed);
+            println!(
+                "Performance: {:.2} timesteps/sec",
+                self.timestep as f64 / elapsed.as_secs_f64()
+            );
+            println!(
+                "Cells/sec: {:.2}M",
+                (self.grid_width * self.grid_height * self.timestep) as f64 / elapsed.as_secs_f64() / 1_000_000.0
+            );
+            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
+            return;
+        }
+
+        if !self.initialized {
+            self.initialized = true;
+            ctx.request_repaint();
+        } else if self.is_playing {  // Handle animation
             self.time_accumulator += ctx.input(|i| i.stable_dt);
             let step_duration = 1.0 / self.steps_per_second;
             
-            while self.time_accumulator >= step_duration {
+            // Adaptive step limiting based on frame time
+            let frame_time = ctx.input(|i| i.stable_dt);
+            let target_frame_time = 1.0 / 60.0; // Target 60 FPS
+            
+            // Calculate how many steps we can afford this frame
+            let max_steps = if self.timestep == 0 || frame_time > target_frame_time * 1.5 {
+                1 // If we're lagging, only do 1 step per frame
+            } else {
+                10 // Otherwise allow up to 10 steps per frame
+            };
+            
+            let mut steps_taken = 0;
+            while self.time_accumulator >= step_duration && steps_taken < max_steps {
                 self.step_simulation();
                 self.time_accumulator -= step_duration;
+                steps_taken += 1;
+            }
+            
+            // Drop excess time if we can't keep up
+            if self.time_accumulator > step_duration * 2.0 {
+                self.time_accumulator = 0.0;
             }
             
             ctx.request_repaint();
@@ -373,7 +426,7 @@ impl eframe::App for CAApp {
                 self.randomize_grid();
             }
             
-            ui.add(egui::Slider::new(&mut self.steps_per_second, 0.1..=100.0)
+            ui.add(egui::Slider::new(&mut self.steps_per_second, 0.1..=10000.0)
                 .text("Steps/sec"));
             
             ui.label(format!("Timestep: {}", self.timestep));
@@ -383,9 +436,9 @@ impl eframe::App for CAApp {
             // Grid size
             ui.label("Grid Configuration");
             let mut changed = false;
-            changed |= ui.add(egui::Slider::new(&mut self.grid_width, 5..=250)
+            changed |= ui.add(egui::Slider::new(&mut self.grid_width, 5..=5000)
                 .text("Width")).changed();
-            changed |= ui.add(egui::Slider::new(&mut self.grid_height, 5..=250)
+            changed |= ui.add(egui::Slider::new(&mut self.grid_height, 5..=5000)
                 .text("Height")).changed();
             changed |= ui.add(egui::Slider::new(&mut self.grid_density, 0.01..=1.0)
                 .text("Density")).changed();
@@ -478,7 +531,7 @@ impl eframe::App for CAApp {
                             });
                     });
                     ui.add(egui::Slider::new(&mut self.base_color_no_actor, 0.0..=0.5).text("Empty cell base color"));
-                    ui.add(egui::Slider::new(&mut self.cell_size, 5.0..=60.0).text("Cell Size"));
+                    ui.add(egui::Slider::new(&mut self.cell_size, 0.01..=60.0).text("Cell Size"));
                     ui.checkbox(&mut self.show_values, "Show Values");
                     ui.separator();
 
```

</details>

---

## Version 1

### Commit Info

<details>
<summary>Click to expand version 1 info</summary>

```
Version: 1
Path: perf-improve-1
Generated: 2026-01-28 11:02:55

==================== CURRENT COMMIT (HEAD) ====================
Commit: d76218f052e602e37145179c01e7f63b12f0cb84
Short:  d76218f
Author: theo <theodeygas@gmail.com>
Date:   2025-12-22 23:09:01 +0100
Msg:    performance boost

==================== UNSTAGED CHANGES ====================
 trait_ac/src/main.rs    | 19 +++++++---
 trait_ac/src/utils.rs   |  5 ++-
 trait_ac_ui/src/main.rs | 99 +++++++++++++++++++++++++++++++++++++------------
 3 files changed, 92 insertions(+), 31 deletions(-)
```

</details>

### Code Changes

<details>
<summary>Click to expand version 1 diff</summary>

```diff
# Git diff for Version 1
# Unstaged changes compared to HEAD
# Generated: 2026-01-28 11:02:55
# HEAD Commit: d76218f

diff --git a/trait_ac/src/main.rs b/trait_ac/src/main.rs
index b682adc..6edaedd 100644
--- a/trait_ac/src/main.rs
+++ b/trait_ac/src/main.rs
@@ -2,20 +2,25 @@ use trait_ac::neighborhood::Neighborhood;
 use trait_ac::grid::Grid;
 use trait_ac::rules::{RuleSet, rule_conway};
 use trait_ac::movement::{apply_movement, movement_static};
-use trait_ac::utils::{print_separator, semantic_trait_names};
+use trait_ac::utils::{print_separator, semantic_trait_names, print_active_traits};
 use std::time::Instant;
 use rayon::prelude::*;
 
 fn main() {
-    let start = Instant::now();
     println!("=== Modular Cellular Automata Simulation ===\n");
 
     // Configuration
-    let grid_height = 250;
-    let grid_width = 250;
+    let grid_height = 3000;
+    let grid_width = 3000;
     let timesteps = 100;
 
-    let active_traits: Vec<usize> = vec![0, 1, 2, 3, 4];
+    let active_traits: Vec<usize> = vec![0];
+
+    let active_mask = vec![
+        vec![true, false, false],
+        vec![false, false, false],
+        vec![false, false, false],
+    ];
 
     // Neighborhood mask
     let neighborhood_traits_mask = vec![
@@ -82,9 +87,11 @@ fn main() {
     println!("Configuration:");
     println!("  Grid: {}x{}", grid_width, grid_height);
     println!("  Timesteps: {}", timesteps);
-    println!("  Active traits: {:?}", active_traits);
+    //println!("  Active traits: {:?}", active_traits);
+    print_active_traits(&grid, &active_mask, &trait_names);
 
     // Simulation loop with optimizations
+    let start = Instant::now();
     for _t in 1..=timesteps {
         // Step 1: Update all active traits using parallel processing
         let mut new_cells: Vec<Vec<_>> = (0..grid.height)
diff --git a/trait_ac/src/utils.rs b/trait_ac/src/utils.rs
index c8211f4..46adaf8 100644
--- a/trait_ac/src/utils.rs
+++ b/trait_ac/src/utils.rs
@@ -22,12 +22,13 @@ pub fn print_trait_array(grid: &Grid, trait_index: usize, trait_name: &str) {
 }
 
 /// Print all active trait arrays
-pub fn print_active_traits(grid: &Grid, active_mask: &[Vec<bool>], trait_names: &[String; 9]) {
+pub fn print_active_traits(_grid: &Grid, active_mask: &[Vec<bool>], trait_names: &[String; 9]) {
+    println!("  Active traits:");
     for mask_row in 0..3 {
         for mask_col in 0..3 {
             if active_mask[mask_row][mask_col] {
                 let trait_index = mask_row * 3 + mask_col;
-                print_trait_array(grid, trait_index, &trait_names[trait_index]);
+                println!("    {}: {} (rule: Conway)", trait_index, &trait_names[trait_index]);
             }
         }
     }
diff --git a/trait_ac_ui/src/main.rs b/trait_ac_ui/src/main.rs
index fd62f3e..700f1d9 100644
--- a/trait_ac_ui/src/main.rs
+++ b/trait_ac_ui/src/main.rs
@@ -5,6 +5,8 @@ use trait_ac::neighborhood::Neighborhood;
 use trait_ac::rules::{rule_static, rule_average, rule_conway, rule_diffusion, rule_maximum, rule_weighted_average};
 use trait_ac::movement::{apply_movement, movement_static, movement_random, movement_gradient, movement_avoid_crowding};
 use rayon::prelude::*;
+use std::time::Instant;
+use trait_ac::utils::{print_separator, print_active_traits};
 
 
 fn main() -> eframe::Result<()> {
@@ -103,7 +105,10 @@ struct CAApp {
     grid_density: f32,
     
     // Simulation state
+    initialized: bool,
     timestep: usize,
+    timestep_max: usize,
+    start: Instant,
     is_playing: bool,
     steps_per_second: f32,
     time_accumulator: f32,
@@ -185,27 +190,27 @@ impl ColorScheme {
 
 impl Default for CAApp {
     fn default() -> Self {
-        let grid_width = 20;
-        let grid_height = 20;
-        let grid_density = 0.33;
+        let grid_width = 3000;
+        let grid_height = 3000;
+        let grid_density = 1.0;
         let grid = Grid::new_with_density(grid_width, grid_height, grid_density);
         
         let active_mask = vec![
-            vec![true, true, false],
-            vec![false, true, false],
             vec![true, false, false],
+            vec![false, false, false],
+            vec![false, false, false],
         ];
         
         let trait_rules = vec![
-            RuleType::Diffusion,
-            RuleType::Average,
             RuleType::Conway,
-            RuleType::Maximum,
-            RuleType::Average,
-            RuleType::Average,
-            RuleType::Diffusion,
-            RuleType::Average,
-            RuleType::Average,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
+            RuleType::Conway,
         ];
         
         Self {
@@ -213,15 +218,18 @@ impl Default for CAApp {
             grid_width,
             grid_height,
             grid_density,
+            initialized: false,
             timestep: 0,
-            is_playing: false,
-            steps_per_second: 2.0,
+            timestep_max: 100,
+            start: Instant::now(),
+            is_playing: true,
+            steps_per_second: 10000.0,
             time_accumulator: 0.0,
             active_mask,
             trait_rules,
             movement_type: MovementType::Static,
             selected_trait: 0,
-            cell_size: 30.0,
+            cell_size: 0.397,
             show_values: false,
             color_scheme: ColorScheme::Viridis,
             base_color_no_actor: 0.1,
@@ -331,14 +339,59 @@ impl CAApp {
 
 impl eframe::App for CAApp {
     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
-        // Handle animation
-        if self.is_playing {
+        if self.timestep >= self.timestep_max {
+            println!("Simulation completed at timestep {}. Closing app.", self.timestep);
+            print_separator();
+            self.timestep_max = 200; // to avoid printing multiple times
+
+            println!("Configuration:");
+            println!("  Grid: {}x{}", self.grid_width, self.grid_height);
+            println!("  Timesteps: {}", self.timestep);
+
+            print_active_traits(&self.grid, &self.active_mask, &self.trait_names);
+            
+            let elapsed = self.start.elapsed();
+            println!("\nExecution time: {:?}", elapsed);
+            println!(
+                "Performance: {:.2} timesteps/sec",
+                self.timestep as f64 / elapsed.as_secs_f64()
+            );
+            println!(
+                "Cells/sec: {:.2}M",
+                (self.grid_width * self.grid_height * self.timestep) as f64 / elapsed.as_secs_f64() / 1_000_000.0
+            );
+            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
+            return;
+        }
+
+        if !self.initialized {
+            self.initialized = true;
+            ctx.request_repaint();
+        } else if self.is_playing {  // Handle animation
             self.time_accumulator += ctx.input(|i| i.stable_dt);
             let step_duration = 1.0 / self.steps_per_second;
             
-            while self.time_accumulator >= step_duration {
+            // Adaptive step limiting based on frame time
+            let frame_time = ctx.input(|i| i.stable_dt);
+            let target_frame_time = 1.0 / 60.0; // Target 60 FPS
+            
+            // Calculate how many steps we can afford this frame
+            let max_steps = if self.timestep == 0 || frame_time > target_frame_time * 1.5 {
+                1 // If we're lagging, only do 1 step per frame
+            } else {
+                10 // Otherwise allow up to 10 steps per frame
+            };
+            
+            let mut steps_taken = 0;
+            while self.time_accumulator >= step_duration && steps_taken < max_steps {
                 self.step_simulation();
                 self.time_accumulator -= step_duration;
+                steps_taken += 1;
+            }
+            
+            // Drop excess time if we can't keep up
+            if self.time_accumulator > step_duration * 2.0 {
+                self.time_accumulator = 0.0;
             }
             
             ctx.request_repaint();
@@ -367,7 +420,7 @@ impl eframe::App for CAApp {
                 self.randomize_grid();
             }
             
-            ui.add(egui::Slider::new(&mut self.steps_per_second, 0.1..=100.0)
+            ui.add(egui::Slider::new(&mut self.steps_per_second, 0.1..=10000.0)
                 .text("Steps/sec"));
             
             ui.label(format!("Timestep: {}", self.timestep));
@@ -377,9 +430,9 @@ impl eframe::App for CAApp {
             // Grid size
             ui.label("Grid Configuration");
             let mut changed = false;
-            changed |= ui.add(egui::Slider::new(&mut self.grid_width, 5..=500)
+            changed |= ui.add(egui::Slider::new(&mut self.grid_width, 5..=5000)
                 .text("Width")).changed();
-            changed |= ui.add(egui::Slider::new(&mut self.grid_height, 5..=500)
+            changed |= ui.add(egui::Slider::new(&mut self.grid_height, 5..=5000)
                 .text("Height")).changed();
             changed |= ui.add(egui::Slider::new(&mut self.grid_density, 0.01..=1.0)
                 .text("Density")).changed();
@@ -472,7 +525,7 @@ impl eframe::App for CAApp {
                             });
                     });
                     ui.add(egui::Slider::new(&mut self.base_color_no_actor, 0.0..=0.5).text("Empty cell base color"));
-                    ui.add(egui::Slider::new(&mut self.cell_size, 1.0..=60.0).text("Cell Size"));
+                    ui.add(egui::Slider::new(&mut self.cell_size, 0.01..=60.0).text("Cell Size"));
                     ui.checkbox(&mut self.show_values, "Show Values");
                     ui.separator();
 
```

</details>

---

## Version 2

### Commit Info

<details>
<summary>Click to expand version 2 info</summary>

```
Version: 2
Path: perf-improve-2
Generated: 2026-01-28 11:02:55

==================== CURRENT COMMIT (HEAD) ====================
Commit: 6002d098279238fb67a70e3e0bcef592aa4353f8
Short:  6002d09
Author: theo <theodeygas@gmail.com>
Date:   2025-12-23 19:46:15 +0100
Msg:    performance boost + refactor

==================== UNSTAGED CHANGES ====================
 trait_ac/src/main.rs    | 10 +++---
 trait_ac_ui/src/main.rs | 81 ++++++++++++++++++++++++++++++++++++++++---------
 2 files changed, 71 insertions(+), 20 deletions(-)
```

</details>

### Code Changes

<details>
<summary>Click to expand version 2 diff</summary>

```diff
# Git diff for Version 2
# Unstaged changes compared to HEAD
# Generated: 2026-01-28 11:02:55
# HEAD Commit: 6002d09

diff --git a/trait_ac/src/main.rs b/trait_ac/src/main.rs
index 3a788cf..36d3ec8 100644
--- a/trait_ac/src/main.rs
+++ b/trait_ac/src/main.rs
@@ -9,16 +9,15 @@ use rayon::prelude::*;
 
 
 fn main() {
-    let start = Instant::now();
     println!("=== Modular Cellular Automata Simulation ===\n");
 
     // Configuration
-    let grid_height = 500;
-    let grid_width = 500;
+    let grid_height = 3000;
+    let grid_width = 3000;
     let grid_density = 1.0;
-    let timesteps = 200;
+    let timesteps = 100;
 
-    let active_traits: Vec<usize> = vec![0, 1, 2, 3, 4];
+    let active_traits: Vec<usize> = vec![0];
 
     // Neighborhood mask
     let neighborhood_traits_mask = vec![
@@ -84,6 +83,7 @@ fn main() {
     print_active_traits(&active_traits, &trait_names, &rules_registry);
 
     // Simulation loop with optimizations
+    let start = Instant::now();
     for _t in 1..=timesteps {
         // Step 1: Update all active traits using parallel processing
         let mut new_cells: Vec<Vec<_>> = (0..grid.height)
diff --git a/trait_ac_ui/src/main.rs b/trait_ac_ui/src/main.rs
index 06ca16c..5cbfa15 100644
--- a/trait_ac_ui/src/main.rs
+++ b/trait_ac_ui/src/main.rs
@@ -4,9 +4,9 @@ use trait_ac::grid::Grid;
 use trait_ac::neighborhood::{Neighborhood, NeighborhoodSettings};
 use trait_ac::rules::{RulesRegistry, Rules, RuleFn};
 use trait_ac::movement::{MovementRegistry, Movements, MovementFn};
-use trait_ac::utils::{semantic_traits_names};
 use rayon::prelude::*;
-
+use std::time::Instant;
+use trait_ac::utils::{semantic_traits_names, print_separator, print_active_traits};
 
 
 fn main() -> eframe::Result<()> {
@@ -95,7 +95,10 @@ struct CAApp {
     grid_density: f32,
     
     // Simulation state
+    initialized: bool,
     timestep: usize,
+    timestep_max: usize,
+    start: Instant,
     is_playing: bool,
     steps_per_second: f32,
     time_accumulator: f32,
@@ -132,22 +135,22 @@ struct CAApp {
 impl Default for CAApp {
     fn default() -> Self {
         // Configuration
-        let grid_width = 500;
-        let grid_height = 500;
-        let steps_per_second = 25.0;
+        let grid_width = 3000;
+        let grid_height = 3000;
+        let steps_per_second = 10000.0;
         let grid_density = 1.0;
 
 
         let grid_width_min = 3;
-        let grid_width_max = 1000;
+        let grid_width_max = 5000;
 
         let grid_height_min = 3;
-        let grid_height_max = 1000;
+        let grid_height_max = 5000;
 
         let steps_per_second_min = 1.0;
-        let steps_per_second_max = 200.0;
+        let steps_per_second_max = 10000.0;
 
-        let active_traits: Vec<usize> = vec![0, 1, 2, 3, 4];
+        let active_traits: Vec<usize> = vec![0];
         let initial_selected_trait = active_traits[0];
 
         // Neighborhood mask
@@ -215,8 +218,11 @@ impl Default for CAApp {
             grid_height,
             grid_density,
 
+            initialized: false,
             timestep: 0,
-            is_playing: false,
+            timestep_max: 100,
+            start: Instant::now(),
+            is_playing: true,
             steps_per_second,
             time_accumulator: 0.0,
             grid_texture: None,
@@ -236,11 +242,11 @@ impl Default for CAApp {
             steps_per_second_max,
             
             selected_trait: initial_selected_trait,
-            cell_size: 3.0,
+            cell_size: 0.397,
             show_values: false,
             color_scheme: ColorScheme::Viridis,
             base_color_no_actor: 0.1,
-            min_cell_size: 1.0,
+            min_cell_size: 0.01,
             max_cell_size: 100.0,
 
             trait_names,
@@ -346,14 +352,59 @@ impl CAApp {
 
 impl eframe::App for CAApp {
     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
-        // Handle animation
-        if self.is_playing {
+        if self.timestep >= self.timestep_max {
+            println!("Simulation completed at timestep {}. Closing app.", self.timestep);
+            print_separator();
+            self.timestep_max = 200; // to avoid printing multiple times
+
+            println!("Configuration:");
+            println!("  Grid: {}x{}", self.grid_width, self.grid_height);
+            println!("  Timesteps: {}", self.timestep);
+
+            print_active_traits(&self.active_traits, &self.trait_names, &self.rules_registry);
+            
+            let elapsed = self.start.elapsed();
+            println!("\nExecution time: {:?}", elapsed);
+            println!(
+                "Performance: {:.2} timesteps/sec",
+                self.timestep as f64 / elapsed.as_secs_f64()
+            );
+            println!(
+                "Cells/sec: {:.2}M",
+                (self.grid_width * self.grid_height * self.timestep) as f64 / elapsed.as_secs_f64() / 1_000_000.0
+            );
+            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
+            return;
+        }
+
+        if !self.initialized {
+            self.initialized = true;
+            ctx.request_repaint();
+        } else if self.is_playing {  // Handle animation
             self.time_accumulator += ctx.input(|i| i.stable_dt);
             let step_duration = 1.0 / self.steps_per_second;
             
-            while self.time_accumulator >= step_duration {
+            // Adaptive step limiting based on frame time
+            let frame_time = ctx.input(|i| i.stable_dt);
+            let target_frame_time = 1.0 / 60.0; // Target 60 FPS
+            
+            // Calculate how many steps we can afford this frame
+            let max_steps = if self.timestep == 0 || frame_time > target_frame_time * 1.5 {
+                1 // If we're lagging, only do 1 step per frame
+            } else {
+                10 // Otherwise allow up to 10 steps per frame
+            };
+            
+            let mut steps_taken = 0;
+            while self.time_accumulator >= step_duration && steps_taken < max_steps {
                 self.step_simulation();
                 self.time_accumulator -= step_duration;
+                steps_taken += 1;
+            }
+            
+            // Drop excess time if we can't keep up
+            if self.time_accumulator > step_duration * 2.0 {
+                self.time_accumulator = 0.0;
             }
             
             ctx.request_repaint();
```

</details>

---

## Version 3

### Commit Info

<details>
<summary>Click to expand version 3 info</summary>

```
Version: 3
Path: perf-improve-3
Generated: 2026-01-28 11:02:55

==================== CURRENT COMMIT (HEAD) ====================
Commit: 4e7ee8c4ba1e3303f56a0ccdd5cabadb7a02ec53
Short:  4e7ee8c
Author: theo <theodeygas@gmail.com>
Date:   2025-12-24 16:54:18 +0100
Msg:    perf boost (less allocation + new approach)

==================== UNSTAGED CHANGES ====================
 trait_ac/src/main.rs    |  8 +++----
 trait_ac_ui/src/main.rs | 62 ++++++++++++++++++++++++++++++++++++++-----------
 2 files changed, 52 insertions(+), 18 deletions(-)
```

</details>

### Code Changes

<details>
<summary>Click to expand version 3 diff</summary>

```diff
# Git diff for Version 3
# Unstaged changes compared to HEAD
# Generated: 2026-01-28 11:02:55
# HEAD Commit: 4e7ee8c

diff --git a/trait_ac/src/main.rs b/trait_ac/src/main.rs
index 5c9ffe8..d75c696 100644
--- a/trait_ac/src/main.rs
+++ b/trait_ac/src/main.rs
@@ -9,16 +9,15 @@ use rayon::prelude::*;
 
 
 fn main() {
-    let start = Instant::now();
     println!("=== Modular Cellular Automata Simulation ===\n");
 
     // Configuration
-    let grid_height = 1500;
-    let grid_width = 1500;
+    let grid_height = 3000;
+    let grid_width = 3000;
     let grid_density = 1.0;
     let timesteps = 100;
 
-    let active_traits: Vec<usize> = vec![0, 1, 2, 3, 4];
+    let active_traits: Vec<usize> = vec![0];
 
     // Neighborhood mask
     let neighborhood_traits_mask = vec![
@@ -90,6 +89,7 @@ fn main() {
     let rows_per_batch = std::cmp::max(1, 4000 / grid_width);
 
     // Simulation loop
+    let start = Instant::now();
     for _t in 1..=timesteps {
         
         // --- STEP 1: Update Traits (Double Buffering) ---
diff --git a/trait_ac_ui/src/main.rs b/trait_ac_ui/src/main.rs
index 81764dd..78799c4 100644
--- a/trait_ac_ui/src/main.rs
+++ b/trait_ac_ui/src/main.rs
@@ -5,8 +5,9 @@ use trait_ac::cell::Cell;
 use trait_ac::neighborhood::Neighborhood;
 use trait_ac::rules::{RulesRegistry, Rules, RuleFn};
 use trait_ac::movement::{MovementRegistry, Movements, MovementFn};
-use trait_ac::utils::{semantic_traits_names};
 use rayon::prelude::*;
+use std::time::Instant;
+use trait_ac::utils::{semantic_traits_names, print_separator, print_active_traits};
 
 
 
@@ -97,7 +98,10 @@ struct CAApp {
     next_grid_cells: Vec<Vec<Cell>>,
     
     // Simulation state
+    initialized: bool,
     timestep: usize,
+    timestep_max: usize,
+    start: Instant,
     is_playing: bool,
     steps_per_second: f32,
     time_accumulator: f32,
@@ -136,23 +140,23 @@ struct CAApp {
 impl Default for CAApp {
     fn default() -> Self {
         // Configuration
-        let grid_width = 500;
-        let grid_height = 500;
-        let steps_per_second = 25.0;
+        let grid_width = 3000;
+        let grid_height = 3000;
+        let steps_per_second = 10000.0;
         let grid_density = 1.0;
 
 
         let grid_width_min = 3;
-        let grid_width_max = 1500;
+        let grid_width_max = 5000;
 
         let grid_height_min = 3;
-        let grid_height_max = 1500;
+        let grid_height_max = 5000;
 
         let steps_per_second_min = 1.0;
-        let steps_per_second_max = 500.0;
+        let steps_per_second_max = 10000.0;
 
-        let cell_size = 3.0;
-        let cell_size_min = 1.0;
+        let cell_size = 0.397;
+        let cell_size_min = 0.01;
         let cell_size_max = 100.0;
 
         let show_values = false;
@@ -161,7 +165,7 @@ impl Default for CAApp {
         let color_scheme = ColorScheme::Viridis;
         let base_color_no_actor = 0.1;
 
-        let active_traits: Vec<usize> = vec![0, 1, 2, 3, 4];
+        let active_traits: Vec<usize> = vec![0];
         let initial_selected_trait = active_traits[0];
 
         // Neighborhood mask
@@ -232,8 +236,11 @@ impl Default for CAApp {
             grid_density,
             next_grid_cells,
 
+            initialized: false,
             timestep: 0,
-            is_playing: false,
+            timestep_max: 100,
+            start: Instant::now(),
+            is_playing: true,
             steps_per_second,
             time_accumulator: 0.0,
             grid_texture: None,
@@ -391,8 +398,35 @@ impl eframe::App for CAApp {
             }
         });
 
-        // Handle animation
-        if self.is_playing {
+        if self.timestep >= self.timestep_max {
+            println!("Simulation completed at timestep {}. Closing app.", self.timestep);
+            print_separator();
+            self.timestep_max = 200; // to avoid printing multiple times
+
+            println!("Configuration:");
+            println!("  Grid: {}x{}", self.grid_width, self.grid_height);
+            println!("  Timesteps: {}", self.timestep);
+
+            print_active_traits(&self.active_traits, &self.trait_names, &self.rules_registry);
+            
+            let elapsed = self.start.elapsed();
+            println!("\nExecution time: {:?}", elapsed);
+            println!(
+                "Performance: {:.2} timesteps/sec",
+                self.timestep as f64 / elapsed.as_secs_f64()
+            );
+            println!(
+                "Cells/sec: {:.2}M",
+                (self.grid_width * self.grid_height * self.timestep) as f64 / elapsed.as_secs_f64() / 1_000_000.0
+            );
+            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
+            return;
+        }
+
+        if !self.initialized {
+            self.initialized = true;
+            ctx.request_repaint();
+        } else if self.is_playing {  // Handle animation
             self.time_accumulator += ctx.input(|i| i.stable_dt);
             let step_duration = 1.0 / self.steps_per_second;
             
@@ -401,7 +435,7 @@ impl eframe::App for CAApp {
             let target_frame_time = 1.0 / 60.0; // Target 60 FPS
             
             // Calculate how many steps we can afford this frame
-            let max_steps = if frame_time > target_frame_time * 1.5 {
+            let max_steps = if self.timestep == 0 || frame_time > target_frame_time * 1.5 {
                 1 // If we're lagging, only do 1 step per frame
             } else {
                 10 // Otherwise allow up to 10 steps per frame
```

</details>

---

## Version 4

### Commit Info

<details>
<summary>Click to expand version 4 info</summary>

```
Version: 4
Path: perf-improve-4
Generated: 2026-01-28 11:02:55

==================== CURRENT COMMIT (HEAD) ====================
Commit: c1d129c3fc3e34c80357368299aae5e6d6a85777
Short:  c1d129c
Author: theo <theodeygas@gmail.com>
Date:   2025-12-25 20:25:03 +0100
Msg:    refactor + better perf

==================== UNSTAGED CHANGES ====================
 trait_ac_ui/src/main.rs | 20 ++++++++++++--------
 1 file changed, 12 insertions(+), 8 deletions(-)
```

</details>

### Code Changes

<details>
<summary>Click to expand version 4 diff</summary>

```diff
# Git diff for Version 4
# Unstaged changes compared to HEAD
# Generated: 2026-01-28 11:02:55
# HEAD Commit: c1d129c

diff --git a/trait_ac_ui/src/main.rs b/trait_ac_ui/src/main.rs
index c4592aa..b68f2b5 100644
--- a/trait_ac_ui/src/main.rs
+++ b/trait_ac_ui/src/main.rs
@@ -97,6 +97,7 @@ struct CAApp {
     next_grid: Grid,
     
     // Simulation state
+    initialized: bool,
     timestep: usize,
     timed_simulation: bool,
     timestep_max: usize,
@@ -142,10 +143,10 @@ impl Default for CAApp {
         // Configuration
         let grid_width = 3000;
         let grid_height = 3000;
-        let steps_per_second = 25.0;
+        let steps_per_second = 10000.0;
         let grid_density = 1.0;
 
-        let timed_simulation = false;
+        let timed_simulation = true;
         let timestep_max = 100;
 
         let grid_width_min = 3;
@@ -155,10 +156,10 @@ impl Default for CAApp {
         let grid_height_max = 5000;
 
         let steps_per_second_min = 1.0;
-        let steps_per_second_max = 500.0;
+        let steps_per_second_max = 10000.0;
 
-        let cell_size = 0.5;
-        let cell_size_min = 0.1;
+        let cell_size = 0.397;
+        let cell_size_min = 0.01;
         let cell_size_max = 100.0;
 
         let show_values = false;
@@ -249,6 +250,7 @@ impl Default for CAApp {
             grid_density,
             next_grid,
 
+            initialized: false,
             timestep: 0,
             timed_simulation,
             timestep_max,
@@ -487,8 +489,10 @@ impl eframe::App for CAApp {
             }
         });
 
-        // Handle animation
-        if self.is_playing {
+        if !self.initialized {
+            self.initialized = true;
+            ctx.request_repaint();
+        } else if self.is_playing {  // Handle animation
             self.time_accumulator += ctx.input(|i| i.stable_dt);
             let step_duration = 1.0 / self.steps_per_second;
             
@@ -497,7 +501,7 @@ impl eframe::App for CAApp {
             let target_frame_time = 1.0 / 60.0; // Target 60 FPS
             
             // Calculate how many steps we can afford this frame
-            let max_steps = if frame_time > target_frame_time * 1.5 {
+            let max_steps = if self.timestep == 0 || frame_time > target_frame_time * 1.5 {
                 1 // If we're lagging, only do 1 step per frame
             } else {
                 10 // Otherwise allow up to 10 steps per frame
```

</details>

---

## Version 5

### Commit Info

<details>
<summary>Click to expand version 5 info</summary>

```
Version: 5
Path: perf-improve-5
Generated: 2026-01-28 11:02:55

==================== CURRENT COMMIT (HEAD) ====================
Commit: de2a57fa29ccf608e63f562f9a0cdb4cd0ece992
Short:  de2a57f
Author: theo <theodeygas@gmail.com>
Date:   2025-12-26 22:28:27 +0100
Msg:    gpu rendering

==================== UNSTAGED CHANGES ====================
 trait_ac_ui/src/main.rs | 18 +++++++++++-------
 1 file changed, 11 insertions(+), 7 deletions(-)
```

</details>

### Code Changes

<details>
<summary>Click to expand version 5 diff</summary>

```diff
# Git diff for Version 5
# Unstaged changes compared to HEAD
# Generated: 2026-01-28 11:02:55
# HEAD Commit: de2a57f

diff --git a/trait_ac_ui/src/main.rs b/trait_ac_ui/src/main.rs
index dbd96af..53b1a53 100644
--- a/trait_ac_ui/src/main.rs
+++ b/trait_ac_ui/src/main.rs
@@ -276,6 +276,7 @@ struct CAApp {
     next_grid: Grid,
     
     // Simulation state
+    initialized: bool,
     timestep: usize,
     timed_simulation: bool,
     timestep_max: usize,
@@ -332,10 +333,10 @@ impl CAApp {
         // Configuration
         let grid_width = 3000;
         let grid_height = 3000;
-        let steps_per_second = 100.0;
+        let steps_per_second = 10000.0;
         let grid_density = 1.0;
 
-        let timed_simulation = false;
+        let timed_simulation = true;
         let timestep_max = 100;
 
         let grid_width_min = 3;
@@ -345,9 +346,9 @@ impl CAApp {
         let grid_height_max = 5000;
 
         let steps_per_second_min = 1.0;
-        let steps_per_second_max = 500.0;
+        let steps_per_second_max = 10000.0;
 
-        let cell_size = 0.5;
+        let cell_size = 0.397;
         let cell_size_min = 0.1;
         let cell_size_max = 100.0;
 
@@ -439,6 +440,7 @@ impl CAApp {
             grid_density,
             next_grid,
 
+            initialized: false,
             timestep: 0,
             timed_simulation,
             timestep_max,
@@ -669,8 +671,10 @@ impl eframe::App for CAApp {
             }
         });
 
-        // Handle animation
-        if self.is_playing {
+        if !self.initialized {
+            self.initialized = true;
+            ctx.request_repaint();
+        } else if self.is_playing {
             self.time_accumulator += ctx.input(|i| i.stable_dt);
             let step_duration = 1.0 / self.steps_per_second;
             
@@ -679,7 +683,7 @@ impl eframe::App for CAApp {
             let target_frame_time = 1.0 / 60.0; // Target 60 FPS
             
             // Calculate how many steps we can afford this frame
-            let max_steps = if frame_time > target_frame_time * 1.5 {
+            let max_steps = if self.timestep == 0 || frame_time > target_frame_time * 1.5 {
                 1 // If we're lagging, only do 1 step per frame
             } else {
                 10 // Otherwise allow up to 10 steps per frame
```

</details>

---

## Version 6

### Commit Info

<details>
<summary>Click to expand version 6 info</summary>

```
Version: 6
Path: perf-improve-6
Generated: 2026-01-28 11:02:55

==================== CURRENT COMMIT (HEAD) ====================
Commit: d5b73d08ec0926cdac30b0a783dc5fb3337bd583
Short:  d5b73d0
Author: theo <theodeygas@gmail.com>
Date:   2026-01-07 18:41:52 +0100
Msg:    refactor, cleanup, fixed small bugs, added config files

==================== UNSTAGED CHANGES ====================
 trait_ac_ui/config.toml | 24 ++++++++++++------------
 trait_ac_ui/src/main.rs | 19 ++++++++++++++-----
 2 files changed, 26 insertions(+), 17 deletions(-)
```

</details>

### Code Changes

<details>
<summary>Click to expand version 6 diff</summary>

```diff
# Git diff for Version 6
# Unstaged changes compared to HEAD
# Generated: 2026-01-28 11:02:55
# HEAD Commit: d5b73d0

diff --git a/trait_ac_ui/config.toml b/trait_ac_ui/config.toml
index 1792849..e2f3af4 100644
--- a/trait_ac_ui/config.toml
+++ b/trait_ac_ui/config.toml
@@ -1,21 +1,21 @@
 # Grid settings
-grid_width = 500
-grid_height = 500
-grid_density = 0.5
-num_traits = 9
+grid_width = 3000
+grid_height = 3000
+grid_density = 1.0
+num_traits = 1
 
 
 # Simulation timing
-steps_per_second = 1000.0
-timed_simulation = false
+steps_per_second = 10000.0
+timed_simulation = true
 timestep_max = 100
 
 
 # Grid bounds
 grid_width_min = 3
-grid_width_max = 5000
+grid_width_max = 10000
 grid_height_min = 3
-grid_height_max = 5000
+grid_height_max = 10000
 
 
 # Steps per second bounds
@@ -24,7 +24,7 @@ steps_per_second_max = 10000.0
 
 
 # Cell display
-cell_size = 1.0
+cell_size = 0.397
 cell_size_min = 0.1
 cell_size_max = 100.0
 
@@ -40,7 +40,7 @@ base_color_no_actor = 0.0
 
 
 # Trait settings
-active_mask = [1, 1, 0, 0, 0, 0, 0, 0, 0]
+active_mask = [1, 0, 0, 0, 0, 0, 0, 0, 0]
 initial_selected_trait = 0
 
 initialisation_ranges = [
@@ -52,12 +52,12 @@ initialisation_ranges = [
 
 # Rules & movement
 rules = [
-    "social energy", "social influence", "conway optimized",
+    "conway optimized", "conway optimized", "conway optimized",
     "conway optimized", "conway optimized", "conway optimized",
     "conway optimized", "conway optimized", "conway optimized",
 ]
 
-movement = "social movement"
+movement = "static"
 
 
 # Neighborhood masks
diff --git a/trait_ac_ui/src/main.rs b/trait_ac_ui/src/main.rs
index e033b51..5574e51 100644
--- a/trait_ac_ui/src/main.rs
+++ b/trait_ac_ui/src/main.rs
@@ -41,6 +41,7 @@ struct CAApp {
     next_grid: Grid,
     
     // Simulation state
+    initialized: bool,
     timestep: usize,
     timed_simulation: bool,
     timestep_max: usize,
@@ -169,10 +170,11 @@ impl CAApp {
             initialisation_ranges: config.initialisation_ranges,
             next_grid,
 
+            initialized: false,
             timestep: 0,
             timed_simulation: config.timed_simulation,
             timestep_max: config.timestep_max,
-            start: Instant::now(),
+            start: Instant::now(), // placeholder, it will be initialized in step_simulation at timestep 0
             is_playing: config.timed_simulation,
             steps_per_second: config.steps_per_second,
             time_accumulator: 0.0,
@@ -362,13 +364,16 @@ impl eframe::App for CAApp {
             }
         });
 
-        // Handle animation
-        if self.is_playing {
+        if !self.initialized {
+            self.initialized = true;
+            ctx.request_repaint();
+        } else if self.is_playing {  // Handle animation
             self.time_accumulator += ctx.input(|i| i.stable_dt);
             let step_duration = 1.0 / self.steps_per_second;
             
-            let frame_time = ctx.input(|i| i.stable_dt);
+            
             let target_frame_time = 1.0 / 60.0;
+            let frame_time = ctx.input(|i| i.stable_dt);
             
             // Calculate remaining time budget based on actual frame time
             // If last frame was slow, we have less budget this frame
@@ -376,7 +381,11 @@ impl eframe::App for CAApp {
             let simulation_budget = (target_frame_time - render_time_estimate).max(0.001);
             
             let estimated_step_time = self.avg_step_time.unwrap_or(0.0001);
-            let max_steps = ((simulation_budget / estimated_step_time) as usize).max(1);
+            let max_steps = if self.timestep == 0 {
+                1
+            } else {
+                ((simulation_budget / estimated_step_time) as usize).max(1)
+            };
             
             let mut steps_taken = 0;
             let step_start = std::time::Instant::now();
```

</details>

---

## Version 7

### Commit Info

<details>
<summary>Click to expand version 7 info</summary>

```
Version: 7
Path: perf-improve-7
Generated: 2026-01-28 11:02:55

==================== CURRENT COMMIT (HEAD) ====================
Commit: 95c8875e8910cc97cfd6448f5fbe387f7f84858f
Short:  95c8875
Author: theo <theodeygas@gmail.com>
Date:   2026-01-27 23:11:42 +0100
Msg:    removed bitvec, updated conway_optimized, updated benchmarks

==================== UNSTAGED CHANGES ====================
 trait_ac_ui/config.toml | 38 +++++++++++++++++---------------------
 1 file changed, 17 insertions(+), 21 deletions(-)
```

</details>

### Code Changes

<details>
<summary>Click to expand version 7 diff</summary>

```diff
# Git diff for Version 7
# Unstaged changes compared to HEAD
# Generated: 2026-01-28 11:02:55
# HEAD Commit: 95c8875

diff --git a/trait_ac_ui/config.toml b/trait_ac_ui/config.toml
index 0bb619a..75ad315 100644
--- a/trait_ac_ui/config.toml
+++ b/trait_ac_ui/config.toml
@@ -1,21 +1,21 @@
 # Grid settings
-grid_width = 500
-grid_height = 500
+grid_width = 3000
+grid_height = 3000
 grid_density = 1.0
-num_traits = 3
+num_traits = 1
 
 
 # Simulation timing
-steps_per_second = 25.0
-timed_simulation = false
+steps_per_second = 10000.0
+timed_simulation = true
 timestep_max = 100
 
 
 # Grid bounds
 grid_width_min = 3
-grid_width_max = 7500
+grid_width_max = 10000
 grid_height_min = 3
-grid_height_max = 7500
+grid_height_max = 10000
 
 
 # Steps per second bounds
@@ -24,7 +24,7 @@ steps_per_second_max = 10000.0
 
 
 # Cell display
-cell_size = 0.5
+cell_size = 0.397
 cell_size_min = 0.1
 cell_size_max = 100.0
 
@@ -46,7 +46,7 @@ base_color_not_empty_max = 1.0
 
 
 # Trait settings
-active_mask = [1, 1, 1, 0, 0, 0, 0, 0, 0]
+active_mask = [1, 0, 0, 0, 0, 0, 0, 0, 0]
 initial_selected_trait = 0
 
 initialisation_ranges = [
@@ -58,27 +58,23 @@ initialisation_ranges = [
 
 # Rules & movement
 rules = [
-    "energy", "charge", "phase",
+    "conway optimized", "conway optimized", "conway optimized",
     "conway optimized", "conway optimized", "conway optimized",
     "conway optimized", "conway optimized", "conway optimized",
 ]
 
-movement = "custom2"
+movement = "static"
 
 
 # Neighborhood masks
 neighborhood_traits_mask = [
-    [1, 1, 1, 1, 1],
-    [1, 1, 1, 1, 1],
-    [1, 1, 1, 1, 1],
-    [1, 1, 1, 1, 1],
-    [1, 1, 1, 1, 1],
+    [1, 1, 1],
+    [1, 1, 1],
+    [1, 1, 1],
 ]
 
 neighborhood_mvt_mask = [
-    [1, 1, 1, 1, 1],
-    [1, 1, 1, 1, 1],
-    [1, 1, 1, 1, 1],
-    [1, 1, 1, 1, 1],
-    [1, 1, 1, 1, 1],
+    [1, 1, 1],
+    [1, 1, 1],
+    [1, 1, 1],
 ]
\ No newline at end of file
```

</details>

---

