# ADR-0003: RNG, shot seeding, and replay boundary

- Status: Accepted
- Date: 2026-07-16
- Owners: Advaith Praveen / Archeum Studios

## Context

Seeded measurement must be reproducible without accidentally promising identical behavior across arbitrary engines or future incompatible algorithms.

## Decision

The built-in engine uses ChaCha8 with a 256-bit seed. A user `u64` seed is expanded deterministically: four little-endian `u64` words formed by SplitMix64 from the user seed. Each shot creates an independent stream by applying SplitMix64 to `seed XOR shot_index` before expansion. RNG identity is recorded as `chacha8-splitmix64-shot-v1`. Probability sampling consumes one `f64` draw in `[0,1)` per non-deterministic measurement; deterministic probability-zero/one measurements consume no draw.

Seeded replay promises the same measurement sequence only for the same executable hash, shot count, RNG identity, reference-engine semantic revision, and compatible floating-point behavior. External backends receive configuration replay only. Cross-platform bit-for-bit packages are not promised without validation.

## Alternatives considered

One shared stream across shots makes parallel execution order observable. Host-default RNGs lack a stable identity. Recording random outcomes alone would imitate replay without reproducing execution.

## Consequences

Shots can be scheduled independently while retaining deterministic results. RNG changes require a new identity and explicit compatibility handling.

## Verification

Golden seed-expansion tests, repeated execution tests, independent-shot tests, and package metadata checks cover the contract.

## Revisit conditions

Security requirements, parallelism evidence, or a validated portability issue may introduce a new RNG revision without redefining revision 1.
