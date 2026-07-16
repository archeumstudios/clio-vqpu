# Classical ISA conformance workload

This internal workload exercises register movement, arithmetic, bitwise logic, shifts, register comparison, conditional branches, and a bounded backward loop. Successful execution leaves `r13 = 5` and `r14 = 1`.

```bash
cargo run -p clio-cli -- run examples/classical-conformance/main.clio --instruction-limit 100
cargo run -p clio-cli -- disasm examples/classical-conformance/main.clio
```
