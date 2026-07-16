# Reproducible benchmarks

Smoke evidence is generated with:

```bash
cargo run --release -p clio-bench -- --smoke --output-dir research/benchmarks
python3 research/benchmarks/scripts/process_results.py
```

Omit `--smoke` for the ten-repetition protocol. Raw records are append-free snapshots: regenerate them as a complete evidence package, never hand-edit values. The environment manifest identifies smoke versus full collection. SVG figures and the processed CSV are deterministically derived from the raw CSV. Checksums are regenerated alongside their artifacts.

The checked-in dataset is preliminary smoke evidence from an Apple `aarch64` macOS environment using Rust 1.93. It validates the pipeline but is not a final comparative benchmark.
