#!/usr/bin/env python3
"""Measure estimator accuracy, serialized trace size, and child-process peak RSS."""

import csv
import json
import platform
import resource
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CLIO = ROOT / "target/release/clio"
OUTPUT = ROOT / "research/benchmarks/processed/resource-trace-results.csv"


def main():
    cases = [("trace-off", "off"), ("trace-instructions", "instructions"), ("trace-state", "state-small")]
    body = "QALLOC q0\nQALLOC q1\nQH q0\nQCX q0, q1\nQMEASURE q0, m0\nQMEASURE q1, m1\nHALT\n"
    rows = []
    temporary = ROOT / "research/benchmarks/processed/resource-probe.clio"
    for name, trace in cases:
        temporary.write_text(f".shots 128\n.trace {trace}\n" + body, encoding="utf-8")
        plan = json.loads(subprocess.run([str(CLIO), "estimate", str(temporary), "--json"], check=True, capture_output=True, text=True).stdout)
        before = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
        result = json.loads(subprocess.run([str(CLIO), "run", str(temporary), "--json"], check=True, capture_output=True, text=True).stdout)
        after = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
        actual_trace = len(json.dumps(result["trace"], separators=(",", ":")).encode())
        estimated_trace = plan["estimated_trace_bytes"]
        rows.append({"schema":"clio-resource-trace-1", "case":name, "estimated_trace_bytes":estimated_trace, "actual_trace_bytes":actual_trace, "trace_estimate_ratio":estimated_trace / max(actual_trace, 1), "estimated_total_bytes":plan["estimated_total_bytes"], "peak_rss_native_units":max(before, after), "peak_rss_units":"bytes" if platform.system() == "Darwin" else "KiB"})
    temporary.unlink(missing_ok=True)
    with OUTPUT.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=rows[0].keys()); writer.writeheader(); writer.writerows(rows)
    print(json.dumps(rows, indent=2))


if __name__ == "__main__": main()
