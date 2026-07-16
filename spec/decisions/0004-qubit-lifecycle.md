# ADR-0004: Initial virtual-qubit lifecycle

- Status: Accepted
- Date: 2026-07-16
- Owners: Advaith Praveen / Archeum Studios

## Context

Dynamic removal of a qubit from an entangled state is not equivalent to deleting an array dimension. Logical handles must remain stable and invalid use must be deterministic.

## Decision

Within one shot, each logical handle transitions `Unallocated -> Allocated -> Freed` and is not implicitly reusable. Initial allocation must be contiguous from `q0`; this keeps basis mapping stable for the reference implementation. Operations require `Allocated`. Measurement does not deallocate. `QRESET` measures/collapses and applies X when needed, leaving the qubit allocated in `|0>`. `QFREE` is accepted only when the qubit is provably in computational `|0>` and separable within numerical tolerance; the initial engine may conservatively reject free when it cannot prove this. Successful free removes the highest mapped qubit only, avoiding remapping live handles. Other free orders trap as unsupported lifecycle operations.

## Alternatives considered

Arbitrary compaction complicates mappings and traces. Implicit measurement on free changes programs probabilistically. Reusable logical names obscure use-after-free diagnostics.

## Consequences

The initial model is safe and inspectable but restrictive. A later accepted ADR may add a free-list or density-matrix trace-out while preserving explicit handle semantics.

## Verification

Tests cover contiguous allocation, duplicate allocation, use before allocation, use after free, measured-but-live qubits, reset, non-highest free rejection, and entangled free rejection.

## Revisit conditions

Representative programs or backend capabilities may justify safe arbitrary release with a fully specified quantum operation and mapping model.
