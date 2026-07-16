# Clio Private Engineering Roadmap

This roadmap describes internal milestones toward one definitive public release. It is not a sequence of public products or versions. A checked box requires implementation, tests, specification, documentation, and integration evidence.

## Current status: Foundation

- [x] Foundational governance and claims documents
- [x] Initial architecture, ISA, and assembly specifications
- [x] Research plan and related-work capture template
- [x] Focused Rust workspace and CI baseline
- [x] Bell-state source and first-path checklist
- [x] First-path architecture decisions reviewed and accepted

## Literature and novelty

- [ ] Primary-source bibliography and literature map
- [ ] Related-work matrix and claim boundaries
- [ ] Research questions and hypotheses frozen before final evaluation

## Architecture and formats

- [ ] Normative processor state, lifecycle, traps, and flags
- [ ] Complete instruction contracts and opcode table
- [ ] Executable, trace, replay, result, and resource specifications
- [ ] Compatibility, integrity, and untrusted-input rules

## Parser and assembler

- [x] Bell-path bounded parser, source spans, validation, diagnostics, and typed assembly
- [ ] Labels and forward branch resolution complete; complete lexer/AST, general symbols, remaining assembly, and disassembler pending
- [ ] Bounded executable serialization and malformed-input tests

## Engine and runtime

- [ ] Independent state-vector engine: single-qubit set, CX/CZ/SWAP/CCX, measurement, reset, safe highest-qubit release, seeded shots, and bounded snapshots complete; remaining observation and advanced lifecycle operations pending
- [ ] Processor runtime: checked classical arithmetic/logic, complete comparison branches, guarded backward loops, instruction/time budgets, hybrid control, and current quantum ISA complete; pause/cancellation API and remaining observation ISA pending
- [ ] Bell, measurement branching, GHZ, teleportation, Bernstein–Vazirani, and Deutsch–Jozsa complete end to end; Grover and QFT pending

## Trace, replay, resources, validation

- [ ] Bell instruction tracing complete; remaining levels, persistence, integrity, and viewer pending
- [ ] Result packages and precisely scoped replay
- [ ] Bell admission has checked memory/qubit/shot/instruction/trace limits; time, stack, and remaining artifact limits pending
- [ ] Conformance, known-answer, property, fuzz, regression, statistical, and differential suites

## Developer product

- [ ] CLI `check`, `build`, `disasm`, `estimate`, `run`, `trace`, bounded `inspect`, and canonical `validate` execute with instruction/time controls; remaining commands and complete process tests pending
- [ ] Bell Rust SDK complete; full stable API and Python bindings pending
- [ ] Clio Studio using the Rust runtime as its source of truth

## Research and definitive release

- [ ] Frozen evaluation protocol and benchmark workloads
- [ ] Raw and processed evidence, generated tables/figures, and reproducibility commands
- [ ] Full paper and architecture specification
- [ ] Security, dependency, license, claims, documentation, and product audits
- [ ] Binaries/packages, checksums, release notes, product page, public repository, Zenodo archive, and verified DOI
