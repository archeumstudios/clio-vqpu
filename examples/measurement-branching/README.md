# Measurement-driven classical branching

This workload verifies that a quantum measurement becomes architectural state and controls later classical execution. H places `q0` in an equal superposition; `QMEASURE` collapses it into `m0`; `MOV` transfers that set bit to `r0`; `CMP` and `JZ` select the path that writes the same value into `r1`.

The initial control-flow subset permits only forward branches. This makes the executed instruction count statically bounded while loop budgets and cancellation are still under development.
