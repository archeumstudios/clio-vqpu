# ADR-0001: State-vector basis and qubit order

- Status: Accepted
- Date: 2026-07-16
- Owners: Advaith Praveen / Archeum Studios

## Context

Every gate, measurement, displayed bit string, trace, and external adapter needs one basis convention. Leaving this implicit creates locally plausible but mutually incompatible results.

## Decision

Clio uses little-endian basis indexing: logical `q0` selects bit 0 of the amplitude-array index. State vectors are column vectors and gate matrices left-multiply them. Human-readable basis strings are displayed as `|q(n-1)…q0>`, with the highest allocated logical qubit on the left. For `QCX control, target`, the first operand controls the second. The first implementation allocates logical qubits contiguously and never changes an allocated handle's meaning.

## Alternatives considered

Big-endian indexing matches some diagram orderings but complicates low-bit masking. Backend-native conventions were rejected because they would make ISA-visible behavior depend on the backend.

## Consequences

Bit masks and pair iteration are simple; display order differs from logical numeric order and must be documented. External adapters must translate explicitly.

## Verification

Gate, Bell-state, measurement, display, and differential tests must cover asymmetric states so reversal cannot pass unnoticed.

## Revisit conditions

Only a proven interoperability or performance issue justifies supersession; stored artifacts then require an explicit compatibility boundary.
