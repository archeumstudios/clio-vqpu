#!/usr/bin/env python3
"""Create or verify the repository-wide evidence checksum manifest."""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "research/evidence-checksums.json"
PATTERNS = [
    "research/benchmarks/raw/*",
    "research/benchmarks/processed/*.csv",
    "research/benchmarks/final/raw/*",
    "research/benchmarks/final/processed/*.csv",
    "research/benchmarks/external/*",
    "research/replay/*.json",
    "research/figures/generated/*.svg",
    "research/figures/generated/*.png",
    "output/pdf/*.pdf",
]


def inventory():
    paths = sorted({path for pattern in PATTERNS for path in ROOT.glob(pattern) if path.is_file()})
    return {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--update", action="store_true"); args = parser.parse_args()
    current = inventory()
    if args.update:
        MANIFEST.write_text(json.dumps({"schema":"clio-evidence-checksums-1", "artifacts":current}, indent=2) + "\n", encoding="utf-8")
        print(f"recorded {len(current)} evidence artifacts")
        return
    expected = json.loads(MANIFEST.read_text(encoding="utf-8"))["artifacts"]
    if current != expected:
        missing = sorted(set(expected) - set(current)); added = sorted(set(current) - set(expected)); changed = sorted(path for path in set(current) & set(expected) if current[path] != expected[path])
        raise SystemExit(f"evidence integrity failed: missing={missing}, added={added}, changed={changed}")
    print(f"verified {len(current)} evidence artifacts")


if __name__ == "__main__": main()
