# Initial Repository Assessment and Architecture Proposal

Date: 2026-07-16  
Scope: foundation before full runtime implementation

## Assessment

The working directory was empty: it contained no repository metadata, source, specification, tests, dependency manifests, or prior work. Rust `1.93.0` and Cargo `1.93.0` were available. The foundation therefore starts without migration constraints or user changes to preserve.

The highest immediate risk is not code volume; it is semantic drift between parser, ISA, engine, runtime, traces, Studio, and research claims. The proposed architecture establishes narrow Rust crate boundaries, freezes cross-cutting behavior in ADRs, and makes tests and specifications land with behavior. The runtime remains the only processor state machine, while quantum mathematics is isolated behind a capability-oriented backend contract and implemented independently by Clio Engine.

## Proposed initial architecture

```text
clio-parser -> spanned AST -> clio-assembler -> typed executable
                                                |
                                        clio-resource admission
                                                |
clio-sdk / clio-cli --------------------> clio-runtime
                                           /    |     \
                                  clio-backend trace  results
                                       |
                                  clio-engine
```

`clio-core` and `clio-isa` are low-level contracts. `clio-validation` consumes public interfaces rather than entering the execution core. Clio Studio will later call the SDK/application backend and must never implement gates or processor transitions in TypeScript.

The initial paths deliberately accept only verified instruction behavior needed for Bell state and measurement-driven classical branching, but these are internal vertical slices of the complete architecture—not a public reduced release. No instruction is marked supported until its execution, specification, diagnostics, trace representation, and conformance evidence exist.

## Exact first implementation sequence

1. Accept ADRs for basis order, arithmetic/flags, RNG/replay, and qubit lifecycle.
2. Implement bounded identifiers, source spans, diagnostics, processor metadata, typed operands, and the five Bell-path instructions.
3. Build a bounded lexer and parser for directives, comments, typed references, and instructions.
4. Add semantic validation and assembly for metadata, allocation/use ordering, operand types, and resolved instruction indices.
5. Implement checked resource arithmetic and perform admission before engine allocation.
6. Build the independent state vector with frozen indexing; verify initialization, H, controlled-X, normalization, probabilities, measurement, collapse, and seeded RNG injection.
7. Define the backend capability interface around those verified engine operations.
8. Implement per-shot processor initialization and fetch/decode/execute for `QALLOC`, `QH`, `QCX`, `QMEASURE`, and `HALT`.
9. Add bounded, version-identified trace events and typed result metadata tied to actual state transitions.
10. Expose the path through the Rust SDK and the needed portions of CLI `check`, `build`, `estimate`, `run`, `inspect`, and `trace`.
11. Run analytic, seed-repeatability, property, malformed-input, resource-boundary, runtime-state, trace, and differential tests.
12. Update the Bell tutorial and support table only from verified behavior.

The detailed exit criteria are in `spec/architecture/first-end-to-end-checklist.md`.

## Major risks

| Risk | Consequence | Initial control |
|---|---|---|
| Qubit/basis convention ambiguity | Correct-looking but incompatible gates and output | Freeze an ADR and cross-test every representation |
| Dynamic qubit free/compaction | Entanglement corruption or unstable logical mapping | Reject unsafe free initially; specify before implementation |
| False replay promises | Irreproducible evidence | Version RNG/seed contract and scope guarantees narrowly |
| Resource underestimation | Process OOM or denial of service | Checked conservative admission before allocation |
| Trace explosion | Memory/disk exhaustion and benchmark distortion | Bounded levels, explicit budgets, no default full vectors |
| Parser/runtime divergence | Parsed instructions that cannot execute | Typed shared ISA and instruction-linked conformance gates |
| Backend semantic mismatch | Differential comparisons become invalid | Capability negotiation and explicit convention translation |
| Floating-point drift | Incorrect collapse or unstable comparisons | Frozen tolerances, norm checks, numerical traps, properties |
| Frontend logic duplication | Studio displays fictitious state | Rust runtime is the sole source of processor truth |
| Research/marketing ahead of evidence | Indefensible public claims | Claims policy, immutable raw data, generated results, audits |
| Scope pressure | Incomplete core hidden by breadth | Complete verified subsystem order; optional adapters last |
| Untrusted artifact parsing | Exhaustion, traversal, or crashes | Bounded safe decoding, integrity checks, fuzzing, cancellation |

## Decisions that must be recorded

- State-vector basis index, logical bit order, matrices, and presentation order
- Virtual-qubit allocation, reset, release, and mapping stability
- Classical overflow, divide/modulo, shifts, comparisons, flags, and halt/fall-through
- RNG algorithm, seed expansion, shot streams, and replay compatibility
- Observation instructions versus debugger/SDK operations
- Executable/trace/result package encoding, internal identifiers, checksums, and limits
- Trace snapshot/backpressure/truncation behavior
- Backend capabilities, numerical tolerance, and unsupported-operation behavior
- Processor-state serialization boundary and treatment of opaque backend state
- Cancellation/time-budget enforcement and deterministic trap codes

## Foundation outcome

The repository now has governance, claims boundaries, a private roadmap, architecture/ISA/grammar starting contracts, research planning templates, ADR discipline, a focused Rust workspace, CI quality gates, and an explicitly non-executable Bell workload plus completion checklist. The full runtime should not begin until the first four cross-cutting ADRs are reviewed.
