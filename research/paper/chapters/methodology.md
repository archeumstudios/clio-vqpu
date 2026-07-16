# Experimental Methodology

## Scope and research questions

The evaluation is organized around correctness, performance scaling, trace cost, resource admission, reproducibility, and inspectability. The current evidence package answers a bounded subset: numerical invariants, independent-reference agreement for single-qubit sequences, known-answer algorithm behavior, and smoke-scale runtime trends. It does not yet establish comparison with a mature external simulator or final performance claims.

## Correctness protocol

Property tests use a fixed 64-bit linear-congruential sequence so every generated operation is reproducible without an opaque test-framework seed. For widths one through six, 128 rotation operations are applied and then inverted in reverse order. Norm is checked after every operation and the recovered amplitude vector is compared component-wise with tolerance `1e-12`. A separately indexed dense matrix-vector implementation serves as an independence check against the engine's amplitude-pair traversal for 256 generated operations. Reduced density matrices are checked for unit trace and Hermitian off-diagonal structure.

Known-answer tests cover Bell and GHZ correlations, teleportation, Bernstein–Vazirani, Deutsch–Jozsa, Grover, and QFT/inverse-QFT recovery. Deterministic algorithms require exact shot agreement. The three-qubit one-iteration Grover workload uses a frozen seed and a conservative success threshold of 0.70, below its analytic probability of 25/32.

## Benchmark protocol

`clio-bench` constructs workloads from checked-in source-generation rules, performs one unrecorded warm-up per configuration, and records every repetition using a monotonic clock. The smoke protocol uses two repetitions; the eventual final protocol uses ten. Families vary allocated qubits, rotation depth, Bell-state shot count, and trace level. Each raw record includes schema, family, parameter, repetition, elapsed nanoseconds, shots, qubits, executed instructions, trace events, and admitted memory estimate.

Release profile execution is mandatory for retained performance evidence. Environment metadata records operating system, architecture, compiler version, build profile, repetition count, smoke/final status, and Git commit when available. Raw CSV and generated artifacts receive SHA-256 checksums. Medians are reported with observed minima and maxima; smoke data is explicitly unsuitable for strong comparative claims or confidence intervals.

## Reproduction

Run `cargo run --release -p clio-bench -- --smoke --output-dir research/benchmarks`, followed by `python3 research/benchmarks/scripts/process_results.py`. The first command regenerates raw records, environment metadata, and the raw checksum. The second derives the summary and all quantitative SVG figures. Run the full Rust quality gate before accepting evidence.

## Threats to validity

The current timings come from one machine, include full SDK parsing and execution, and have only two smoke repetitions. Cache state, process scheduling, CPU frequency, compiler version, and background activity can influence measurements. Trace serialization size and peak resident memory are not yet measured. The independent dense reference is intentionally structured differently but remains repository-owned; a mature external baseline is still required before final cross-simulator claims.
