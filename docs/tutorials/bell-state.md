# Bell State Through Clio VQPU

The current internal executable path supports the Bell workload in `examples/bell-state/main.clio`. This is a verified vertical slice of the eventual processor, not the complete Clio ISA or definitive release.

Check and estimate it before execution:

```bash
cargo run -p clio-cli -- check examples/bell-state/main.clio
cargo run -p clio-cli -- estimate examples/bell-state/main.clio
```

Execute through parser, semantic validation, assembly, resource admission, Clio Runtime, and the built-in Clio Engine:

```bash
cargo run -p clio-cli -- run examples/bell-state/main.clio
```

The engine initializes `|00>`, applies H to logical `q0`, then applies controlled-X from `q0` to `q1`. Under Clio's little-endian basis-index convention, the only nonzero amplitudes before measurement are indices 0 and 3, each `1/sqrt(2)`. Measuring `q0` collapses the shared state, so measuring `q1` must agree. Across many shots, only `00` and `11` are valid; their exact counts are seeded samples and need not be equal.

Use JSON for structured results or request the complete admitted trace/result object:

```bash
cargo run -p clio-cli -- run examples/bell-state/main.clio --json
cargo run -p clio-cli -- trace examples/bell-state/main.clio
```

The result records source hash, engine and RNG semantic identities, seed, shots, resource plan, final architectural state, counts, and real instruction transitions. Clio still performs classical state-vector simulation; this workflow does not use physical quantum hardware.
