# First Executable Path: Bell State

This is a private implementation checklist, not a reduced product definition. The path is complete only when source reaches real state-vector measurement through the processor and produces retained evidence.

## 0. Freeze cross-cutting decisions

- [x] ADR: basis/index order, matrix orientation, control/target mapping, and display bit strings
- [x] ADR: checked classical arithmetic, flags, branch targets, and fall-through
- [x] ADR: RNG algorithm, seed derivation per shot, and replay compatibility
- [x] ADR: initial allocation/free/reset semantics
- [ ] Define diagnostic, trap, resource, and trace error codes used by this path

## 1. Architecture types and ISA

- [x] Bounded register indices, logical qubit IDs, source spans, diagnostics, and hashes
- [x] Typed `QALLOC`, `QH`, `QCX`, `QMEASURE`, and `HALT` instruction representations
- [x] Program metadata for name, seed, shots, budget, and trace level
- [x] Instruction metadata and source mapping
- [ ] Unit tests for all constructors, bounds, and serialization-ready shapes

## 2. Source front end

- [ ] Bounded lexer with comments, directives, typed operands, labels, and source spans
- [x] Parser for the Bell source with stable diagnostics
- [x] Semantic validation for directives, operand types, allocation order, and use-before-allocation
- [x] Assembly to typed instruction indices and an internally identified executable
- [ ] Positive snapshot plus wrong-register, missing-operand, unknown-mnemonic, overflow, and oversized-input tests

## 3. Reference engine

- [x] Checked state allocation and `|0…0>` initialization
- [x] `H` and controlled-X using the frozen basis convention
- [x] Norm/probability calculation with declared tolerance
- [x] State-derived measurement, collapse, renormalization, and deterministic RNG injection
- [x] Exact pre-measurement Bell amplitudes and correlation tests
- [x] Seed-repeatability and many-shot statistical tests without requiring an exact 50/50 split

## 4. Resource admission

- [x] Checked `16 × 2^n` raw memory calculation and conservative overhead model
- [x] Qubit, state/total memory, instruction, shot, and trace limits
- [x] Admission before engine allocation with explicit rejection evidence
- [ ] Boundary, overflow, and malicious giant-request tests

## 5. Runtime state machine

- [x] Initialize all registers, unset measurements, status, program counter, counters, and metadata
- [x] Fetch/decode/execute each Bell instruction through a backend interface
- [x] Map logical qubits to engine state without leaking raw indices into ISA callers
- [x] Write measurements; keep unset distinct from zero
- [x] Define per-shot reset and aggregate `00`/`11` counts
- [x] Correct `Ready -> Running -> Halted` transitions and typed backend trap paths
- [ ] Instruction/time cancellation budgets and no continuation after trap

## 6. Trace and results

- [x] Version-identified admitted events for allocation, gates, measurement, status, and halt
- [ ] No full vector by default; safe small-state snapshots only when requested
- [ ] Result contains hashes, build/engine/RNG identifiers, seed, shots, counts, final architectural state policy, budgets/usage, warnings, and metrics
- [ ] Integrity-checked trace/result encoding and malformed-input tests
- [ ] Supported same-contract seeded replay test

## 7. Interfaces and validation

- [x] Rust SDK parse/check/assemble/estimate/run workflow
- [ ] CLI `check`, `build`, `run`, `estimate`, `inspect`, and `trace` portions needed by Bell, including JSON and exit-code tests
- [x] Analytic validation: only `00` and `11`, exact pre-measurement probabilities within tolerance
- [ ] Differential validation against a selected primary reference with convention translation documented
- [ ] `cargo fmt`, clippy with warnings denied, unit, integration, conformance, property, and regression checks green

## Exit evidence

- [x] `examples/bell-state/main.clio` executes through parser, validator, assembler, runtime, and Clio Engine
- [x] Seeded run is repeatable under the recorded environment and RNG contract
- [ ] Trace is parseable and corresponds to actual state transitions
- [x] Resource plan precedes allocation
- [ ] Specifications and user tutorial match tested behavior
- [ ] No parser-only or hard-coded behavior is described as implemented
