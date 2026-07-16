.PHONY: check fmt lint test evidence

check: fmt lint test

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-targets

evidence:
	python3 research/benchmarks/scripts/evidence_integrity.py
