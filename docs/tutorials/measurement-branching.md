# Measurement-Driven Classical Control

Run the verified hybrid example:

```bash
cargo run -p clio-cli -- run examples/measurement-branching/main.clio
```

The program applies H to `q0`, measures into three-state register `m0`, moves the set result into signed classical register `r0`, compares it with zero, and follows a forward branch. Both paths write the measurement value to `r1` before halting.

An unset measurement register is not zero. `MOV r0, m0` traps if `m0` has not been written during the shot. `JZ` similarly traps without an executed `CMP`. These rules prevent hidden initialization assumptions.

The trace records the measurement change, classical register mutations, and whether each branch was taken. The program is reinitialized for every shot, and its seeded outcome stream follows ADR-0003. This remains classical simulation of quantum-state evolution, not physical quantum execution.
