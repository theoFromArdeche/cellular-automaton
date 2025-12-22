```bash
cargo install flamegraph
sudo sh -c 'echo 0 > /proc/sys/kernel/perf_event_paranoid'
cargo flamegraph --bin trait_ac
```