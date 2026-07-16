#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

test -f output/pdf/clio-vqpu-research-paper.pdf
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --release --locked -p clio-cli -p clio-studio -p clio-replay -p clio-bench
python3 research/benchmarks/scripts/evidence_integrity.py

out=output/release
stage="$out/stage"
rm -rf "$stage"
mkdir -p "$stage/bin" "$stage/docs"
cp target/release/clio "$stage/bin/"
cp target/release/clio-studio "$stage/bin/"
cp target/release/clio-replay "$stage/bin/"
cp target/release/clio-bench "$stage/bin/"
cp README.md LICENSE CITATION.cff SECURITY.md CLAIMS_POLICY.md "$stage/docs/"
cp output/pdf/clio-vqpu-research-paper.pdf "$stage/docs/"

platform="$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
tar -C "$stage" -czf "$out/clio-vqpu-definitive-$platform.tar.gz" .
git archive --format=tar.gz --output="$out/clio-vqpu-definitive-source.tar.gz" HEAD
python3 packaging/scripts/generate_sbom.py
(cd "$out" && shasum -a 256 clio-vqpu-definitive-*.tar.gz clio-vqpu-sbom.cdx.json > SHA256SUMS)
rm -rf "$stage"
echo "Release candidate artifacts written to $out"
