# ADR-0002: Classical arithmetic, flags, branches, and completion

- Status: Accepted
- Date: 2026-07-16
- Owners: Advaith Praveen / Archeum Studios

## Context

Processor execution must not depend on Rust build mode or host-language overflow behavior.

## Decision

Classical registers are signed 64-bit integers initialized to zero. Arithmetic is checked. Overflow, division or modulo by zero, `i64::MIN / -1`, and shift amounts outside `0..64` produce typed runtime traps. Right shift is arithmetic. `CMP a, b` records signed ordering: `ZERO` iff equal and `NEGATIVE` iff `a < b`. `JZ`, `JNZ`, `JLT`, and `JGT` consume this comparison state; using them before `CMP` traps. Successful measurement sets exactly one measurement flag. Reaching the instruction count by normal increment produces `Completed`; `HALT` produces `Halted`; any other out-of-range target traps.

## Alternatives considered

Wrapping arithmetic is predictable but hides likely program errors. Saturating arithmetic loses information. Updating comparison flags after every arithmetic operation is familiar in hardware but makes dependencies less explicit.

## Consequences

Programs are portable and failures deterministic. Extra checked operations have acceptable reference-runtime cost. The first Bell subset uses only `HALT`, but later classical implementation must follow this contract.

## Verification

Boundary tests cover every overflow, invalid shift, comparison ordering, branch-before-compare, explicit halt, fall-through, and malformed target.

## Revisit conditions

Only a documented workload requirement may add explicit wrapping or saturating opcodes; existing instruction semantics remain unchanged.
