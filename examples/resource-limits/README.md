# Trap and execution-limit examples

These programs deliberately produce signed-overflow, division-by-zero, and instruction-budget traps. They are negative conformance inputs, not failing test fixtures.

```bash
cargo run -p clio-cli -- run examples/resource-limits/infinite-loop/main.clio --instruction-limit 100 --json
```
