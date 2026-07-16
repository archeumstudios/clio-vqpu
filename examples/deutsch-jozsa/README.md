# Deutsch–Jozsa

The workload family includes constant-zero, constant-one, and balanced-parity oracles over three input qubits. Constant functions must measure `000`; the parity oracle must produce a nonzero string (specifically `111` for this oracle).

```bash
cargo run -p clio-cli -- validate examples/deutsch-jozsa/balanced-parity/main.clio
```
