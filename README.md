# Clio VQPU

**A Programmable Virtual Quantum Processing Architecture**  
An Archeum Studios project created and led by Advaith Praveen.

Clio VQPU is a software-defined virtual quantum processor with its own architecture, instruction set, processor-state model, runtime, reference execution engine, hybrid classical–quantum control system, tracing infrastructure, validation framework, and visual development environment.

Clio is being built incrementally in private toward one definitive public release. The repository currently contains verified Bell, GHZ, measurement-driven branching, teleportation, Bernstein–Vazirani, and Deutsch–Jozsa paths plus checked classical arithmetic, logic, comparisons, and budgeted loops. These execute through parsing, semantic validation, assembly, resource admission, Clio Runtime, Clio Engine, tracing, results, the Rust SDK, and CLI. This is not the complete product or definitive release. See [ROADMAP.md](ROADMAP.md) for completion gates.

## Honest execution boundary

Clio is not physical quantum hardware. Its built-in engine will simulate quantum-state evolution using classical computation. A general state-vector simulation stores \(2^n\) complex amplitudes for \(n\) qubits and requires approximately \(16 \times 2^n\) bytes for raw double-precision amplitudes, excluding runtime overhead. Clio does not claim quantum advantage, unlimited qubit capacity, or superiority over established simulators.

## Intended processor pipeline

```text
Clio Assembly -> parser -> AST -> semantic validation -> assembler
              -> validated instruction stream -> Clio Runtime
              -> Clio Engine/backend -> result + trace + replay package
```

The architecture is owned by Clio. External frameworks may later be optional interoperability and differential-validation adapters; they are not required for Clio's identity or local reference execution.

## Repository map

- `crates/`: focused Rust libraries and the `clio` CLI
- `spec/`: normative architecture, ISA, assembly, and decision records
- `examples/`: executable Clio Assembly workloads as support lands
- `docs/`: user and contributor documentation
- `research/`: literature, novelty, protocol, and reproducibility artifacts
- `tests/`: cross-crate conformance and integration tests (created with their implementations)
- `apps/clio-studio/`: the future Tauri/React visual product

## Current developer checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

The first executable end-to-end target is [`examples/bell-state/main.clio`](examples/bell-state/main.clio). Its exact implementation checklist is in [spec/architecture/first-end-to-end-checklist.md](spec/architecture/first-end-to-end-checklist.md), and the internal tutorial is in [docs/tutorials/bell-state.md](docs/tutorials/bell-state.md).

## Project governance

- [Project contract](PROJECT_CONTRACT.md)
- [Claims policy](CLAIMS_POLICY.md)
- [Private engineering roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)

## License and citation

The intended source license is Apache License 2.0, subject to the final dependency and asset audit. Citation metadata is provided in [`CITATION.cff`](CITATION.cff). No public release has occurred yet; release identifiers and DOI fields must only be filled with verified values.
## Clio Studio

Launch the real visual processor laboratory locally:

```bash
cargo run --release -p clio-studio
```

Then open `http://127.0.0.1:4317`. Studio provides source editing, canonical examples, build/run/validation, instruction timeline scrubbing, bounded state-vector inspection, registers, measurements, resource plans, results, and verified replay import/export.

The complete research monograph is generated at `output/pdf/clio-vqpu-research-paper.pdf`.
