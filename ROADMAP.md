# Clio Engineering and Release Record

Clio was developed through private engineering milestones leading to one definitive public release. Those milestones were development controls, not separate public product versions.

## Completed release scope

- Architecture, processor state, ISA, assembly language, formats, compatibility rules, and security boundaries
- Parser, semantic validation, assembler, disassembler, resource admission, runtime, and independent state-vector engine
- Checked classical control, quantum operations, seeded measurement, tracing, observation, replay, and bounded inspection
- Bell, GHZ, Grover, QFT, teleportation, Bernstein–Vazirani, Deutsch–Jozsa, and hybrid-control examples
- Rust SDK, command-line tools, Clio Bench, and browser-based Clio Studio backed by the real SDK
- Known-answer, property, regression, statistical, replay, and Qiskit differential validation
- Frozen benchmark protocol, raw evidence, processed results, publication-quality figures, and integrity manifest
- Architecture documentation, tutorials, security documentation, reproducibility material, and research monograph
- Source and macOS ARM64 packages, CycloneDX SBOM, SHA-256 checksums, CI, and public repository metadata

## Publication record

The definitive GitHub release is tagged `definitive-release` and permanently archived on Zenodo at [doi:10.5281/zenodo.21403143](https://doi.org/10.5281/zenodo.21403143).

## Honest boundary

Clio is a classical simulation of a programmable virtual quantum processing architecture. It is not physical quantum hardware and does not claim quantum advantage, unlimited simulation scale, or superiority over established simulators. See [`CLAIMS_POLICY.md`](CLAIMS_POLICY.md) for the complete claims boundary.
