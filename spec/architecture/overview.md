# Clio Architecture Overview

Status: initial design contract; normative details will be refined through reviewed ADRs and conformance tests.

## System boundary

Clio accepts bounded Clio Assembly source, produces a validated executable instruction stream, and executes that stream as a virtual processor. The runtime owns state transitions and delegates quantum-state operations through a backend trait. Clio Engine is the required built-in reference implementation. Optional adapters cannot change ISA-visible behavior without an explicit capability or compatibility error.

```text
source -> parser -> AST -> validator -> assembler -> executable
                                                     |
                                          resource admission
                                                     |
                                              Clio Runtime
                                          /       |        \
                                      Engine    Trace     Results
```

## Initial module boundaries

- `clio-core`: shared bounded identifiers, source spans, diagnostics, and architecture metadata; no execution logic.
- `clio-isa`: typed operands, instructions, opcodes, and instruction metadata.
- `clio-parser`: source text to spanned AST; no semantic acceptance.
- `clio-assembler`: symbol resolution, semantic validation, and executable construction.
- `clio-engine`: independent state-vector mathematics and measurement.
- `clio-backend`: capability-oriented backend interface used by the runtime.
- `clio-resource`: conservative plans, budgets, usage, and admission decisions.
- `clio-trace`: bounded structured events and package encoding.
- `clio-runtime`: the only ISA execution state machine.
- `clio-validation`: conformance and comparison metrics.
- `clio-sdk`: stable orchestration API over parser, assembler, runtime, and artifacts.
- `clio-cli`: command adapter with human and machine output.

Dependencies must point toward contracts: CLI/SDK depend on subsystems; runtime depends on ISA, backend, resource, and trace contracts; the engine does not depend on the runtime, parser, CLI, or UI.

## Processor state

`ProcessorState` is serializable where safe and contains:

- program counter as an instruction index;
- `Ready | Running | Paused | Completed | Halted | Trapped | ResourceRejected | ValidationFailed` status;
- sixteen signed 64-bit classical registers `r0..r15`, initialized to zero;
- sixteen measurement registers `m0..m15`, initialized to `Unset`, then `Zero` or `One`;
- typed virtual-qubit records and logical-to-engine mapping;
- opaque quantum-state/backend handle rather than a serialized arbitrary backend object;
- comparison and measurement flags with defined writers;
- seed, RNG identity, shot index/count, instruction counter, budgets and usage;
- trace configuration, warnings, diagnostics, and runtime/build metadata;
- a bounded call stack only if `CALL` and `RET` are accepted by a later ADR.

An unset measurement register is an error when read. Moving a set measurement value to a classical register yields integer `0` or `1`.

## Classical arithmetic

Clio uses checked signed 64-bit arithmetic. Overflow, division or modulo by zero, invalid shifts, and `i64::MIN / -1` trap; they never wrap silently. Bitwise operations use the signed register bit representation and right shift is arithmetic. `CMP lhs, rhs` sets explicit ordering information consumed by conditional branches. Branch targets are resolved and range-checked during assembly. Backward branches are allowed only under configured global instruction and wall-clock budgets; their resource plan conservatively assumes the full instruction budget.

## Virtual-qubit lifecycle

```text
Undeclared -> Allocated -> Measured (still allocated) -> Reset/operated -> Allocated
                         \-> Freed
Allocated ----------------> Freed
```

`QALLOC qN` creates a typed logical handle and maps it to an engine position. Duplicate allocation traps or is rejected during static validation when provable. Quantum operations require allocation. `QFREE` requires the qubit to be separable and in `|0>` unless the final engine design defines and tests a trace-preserving compaction rule; the first implementation should reject unsafe release. Freed handles cannot be reused implicitly within the same execution. `QRESET` measures/collapses as necessary and produces `|0>` according to the engine convention.

## Execution lifecycle

1. Parse bounded input and retain source spans.
2. Resolve symbols and validate types, directives, control-flow targets, capabilities, and declared limits.
3. Assemble a versioned in-memory executable and compute its program hash.
4. Estimate worst-case required resources known statically and reject budgets before state-vector allocation.
5. For every shot, initialize architectural state, derive a deterministic shot RNG stream from the configured seed, and run fetch/decode/execute.
6. Emit bounded trace events around state transitions. A failed trace write cannot silently erase provenance; policy determines trap or explicit downgrade before execution.
7. Stop only on normal fall-through completion, `HALT`, pause/cancellation, deterministic trap, or configured budget rejection.
8. Aggregate counts and produce typed result metadata. Preserve per-shot final state only when explicitly requested within bounds.

`HALT` produces `Halted`; reaching exactly one instruction beyond the final instruction produces `Completed`. Other out-of-range program counters trap. A validation or resource failure occurs before `Running`.

## Endianness and state-vector convention

The proposed reference convention maps logical qubit `q0` to bit 0 of the basis index (little-endian basis indexing). The amplitude array index is the integer represented by basis bits; `|q(n-1)...q0>` is used for display. Gate matrices multiply column state vectors. This decision must be locked in an ADR before engine implementation and tested across all gates, measurement, display, import/export, and differential adapters.

## Traps and errors

Source and semantic failures return spanned diagnostics before execution. Malformed executables fail safe. Runtime faults become typed traps containing code, program counter, instruction, source location if available, and non-secret context. Unsupported capabilities fail explicitly. Undefined behavior is not part of Clio ISA.

## Determinism and replay

Classical execution is deterministic. Seeded reference-engine execution targets identical measurement sequences only for the same executable, seed policy, RNG algorithm revision, compatible engine/build contract, and documented numerical conditions. External hardware receives configuration replay only. Bit-for-bit cross-platform replay is not promised without evidence.

## Resource safety

For `n` active qubits, raw state memory is checked as `16 * 2^n` bytes. Total estimates add engine, mapping, runtime, program, result, and bounded trace overhead conservatively. All exponentiation, multiplication, counters, and sizes use checked arithmetic. Admission includes qubit, raw/total memory, source/executable size, instruction, shot, time, stack, and trace limits.

## Initial decisions requiring ADRs

1. Basis order and qubit mapping.
2. Allocation/free semantics and state-vector compaction.
3. RNG algorithm, per-shot stream derivation, and replay compatibility.
4. Overflow, comparison, shift, and fall-through semantics.
5. Which observation operations are ISA instructions versus debugger/SDK queries.
6. Executable and package serialization/integrity formats.
7. Trace backpressure, truncation, and downgrade behavior.
8. Backend capability negotiation and numerical tolerances.
