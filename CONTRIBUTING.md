# Contributing to Clio VQPU

Clio is developed as one coherent research product. Contributions should preserve the project contract and honest simulation boundary.

## Before changing behavior

Read `PROJECT_CONTRACT.md`, `CLAIMS_POLICY.md`, and the relevant file under `spec/`. For durable architectural changes, add an ADR using `spec/decisions/0000-template.md`. Do not silently make implementation behavior diverge from the specification.

## Development

Use stable Rust and keep crates focused. Processor execution belongs in Rust, not frontend code. Avoid unsafe Rust unless a reviewed ADR establishes necessity and invariants. Treat parsed files as untrusted and apply explicit size, memory, instruction, time, shot, trace, and recursion limits.

Before submitting a change, run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

Features require unit tests plus the relevant integration, conformance, numerical, property, fuzz, or regression coverage. Update specifications, examples, and user documentation in the same change.

## Research contributions

Use primary sources, record DOI or official URLs, and never fabricate citation fields. Preserve raw observations and generate processed data, tables, and figures through checked-in scripts. Clearly label exploratory results and do not enter them manually into final artifacts.

## Commit and review scope

Prefer cohesive changes with an explicit reason, behavior contract, tests, and risk assessment. Do not include proprietary or uncertain-license code, workloads, fonts, icons, or other assets.

## Conduct and security

Be respectful and technically rigorous. Report vulnerabilities privately using `SECURITY.md`, not a public issue.
