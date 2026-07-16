# Clio SDK reference

The Rust SDK exposes a source-first API:

- `build(source)` parses, validates, and returns the typed executable.
- `disassemble(source)` returns a numbered typed instruction listing.
- `estimate(source, limits)` performs resource admission without state allocation.
- `execute(source, limits)` runs all declared shots.
- `inspect(source, limits)` forces one shot with bounded state snapshots.
- `observe(source, limits)` returns norm, basis probabilities, marginals, Pauli expectations, and reduced single-qubit states.
- `validate(source, limits)` executes a recognized canonical workload and returns a typed report.

Every fallible operation returns `SdkError`, preserving validation diagnostics, resource rejection, and runtime traps. Hosts must set `ResourceLimits` deliberately for untrusted programs. The default is a local-development ceiling, not a universal service policy.

Observation and inspection are debugger operations, not ISA instructions. They cannot be inserted into a program to affect architectural state.
