#!/usr/bin/env python3
"""Synchronize figure checksums and implementation commit identities."""

import csv
import hashlib
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[3]
MANIFEST=ROOT/"research/figures/figure-manifest.csv"
commit=subprocess.run(["git","rev-parse","HEAD"],cwd=ROOT,check=True,capture_output=True,text=True).stdout.strip()
with MANIFEST.open(newline="",encoding="utf-8") as handle:
    reader=csv.DictReader(handle); rows=list(reader); fields=reader.fieldnames
for row in rows:
    output=ROOT/row["output_path"] if row.get("output_path") else None
    if output and output.is_file(): row["checksum"]=hashlib.sha256(output.read_bytes()).hexdigest()
    if row.get("raw_data") or row.get("processed_data"): row["commit_hash"]=commit
with MANIFEST.open("w",newline="",encoding="utf-8") as handle:
    writer=csv.DictWriter(handle,fieldnames=fields);writer.writeheader();writer.writerows(rows)
print(f"synchronized {len(rows)} figure records to {commit}")
