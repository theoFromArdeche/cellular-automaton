# All versions need to have the test mode implemented and enabled
I modified some versions slightly to add a test mode
```bash
cd perf-improve-0/trait_ac_ui
echo "


========== Perf improve 0 (base) : =========="
cargo run --release
sleep 2m
cd ../../perf-improve-1/trait_ac_ui
echo "


========== Perf improve 1: =========="
cargo run --release
sleep 2m
cd ../../perf-improve-2/trait_ac_ui
echo "


========== Perf improve 2: =========="
cargo run --release
sleep 2m
cd ../../perf-improve-3/trait_ac_ui
echo "


========== Perf improve 3: =========="
cargo run --release
sleep 2m
cd ../../perf-improve-4/trait_ac_ui
echo "


========== Perf improve 4: =========="
cargo run --release
sleep 2m
cd ../../perf-improve-5/trait_ac_ui
echo "


========== Perf improve 5: =========="
cargo run --release
sleep 2m
cd ../../perf-improve-6/trait_ac_ui
echo "


========== Perf improve 6: =========="
cargo run --release
cd ../..
```


# Results : 

```bash
========== Perf improve 0 (base) : ==========
    Finished `release` profile [optimized] target(s) in 0.14s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: Conway)

Execution time: 292.110316095s
Performance: 0.34 timesteps/sec
Cells/sec: 1.37M



========== Perf improve 1: ==========
    Finished `release` profile [optimized] target(s) in 0.14s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: Conway)

Execution time: 205.216862182s
Performance: 0.49 timesteps/sec
Cells/sec: 1.95M



========== Perf improve 2: ==========
    Finished `release` profile [optimized] target(s) in 0.14s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: conway)

Execution time: 15.889493512s
Performance: 6.29 timesteps/sec
Cells/sec: 25.17M



========== Perf improve 3: ==========
    Finished `release` profile [optimized] target(s) in 0.08s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: conway)

Execution time: 15.994609017s
Performance: 6.25 timesteps/sec
Cells/sec: 25.01M



========== Perf improve 4: ==========
    Finished `release` profile [optimized] target(s) in 0.08s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: conway)

Execution time: 8.760514853s
Performance: 11.41 timesteps/sec
Cells/sec: 45.66M



========== Perf improve 5: ==========
    Finished `release` profile [optimized + debuginfo] target(s) in 0.14s
     Running `target/release/trait_ac_ui`
Simulation completed at timestep 100. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 100
  Active traits:
    0: Energy (rule: conway optimized)

Execution time: 2.76714677s
Performance: 36.14 timesteps/sec
Cells/sec: 144.55M



========== Perf improve 6: ==========
    Finished `release` profile [optimized + debuginfo] target(s) in 0.08s
     Running `target/release/trait_ac_ui`
✓ GPU renderer initialized
Simulation completed at timestep 101. Closing app.

============================================================
Configuration:
  Grid: 2000x2000
  Timesteps: 101
  Active traits:
    0: Energy (rule: conway optimized)

Execution time: 1.320260187s
Performance: 76.50 timesteps/sec
Cells/sec: 306.00M
```