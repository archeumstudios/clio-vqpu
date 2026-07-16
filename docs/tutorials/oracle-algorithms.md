# Oracle Algorithms

Clio's Bernstein–Vazirani workloads recover hidden strings exactly through phase kickback and Hadamard interference. The Deutsch–Jozsa workloads classify constant-zero, constant-one, and balanced-parity oracles.

```bash
cargo run -p clio-cli -- validate examples/bernstein-vazirani/secret-1011/main.clio
cargo run -p clio-cli -- validate examples/deutsch-jozsa/balanced-parity/main.clio
```

The checked-in validators require every shot to match the analytic result. These small noiseless state-vector workloads are correctness tests, not evidence of quantum advantage.
