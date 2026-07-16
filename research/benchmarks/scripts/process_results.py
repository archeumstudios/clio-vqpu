#!/usr/bin/env python3
"""Process retained Clio benchmark records and generate editable SVG plots."""

import argparse
import csv
import hashlib
import json
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FIGURES = ROOT / "research/figures/generated"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dataset", default="research/benchmarks")
    parser.add_argument("--suffix", default="")
    args = parser.parse_args()
    dataset = ROOT / args.dataset
    raw = dataset / "raw/benchmark-results.csv"
    processed = dataset / "processed/benchmark-summary.csv"
    with raw.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    grouped: dict[tuple[str, int], list[int]] = {}
    for row in rows:
        grouped.setdefault((row["family"], int(row["parameter"])), []).append(int(row["elapsed_ns"]))
    summaries = []
    for (family, parameter), values in sorted(grouped.items()):
        summaries.append({
            "schema": "clio-benchmark-summary-1",
            "family": family,
            "parameter": parameter,
            "samples": len(values),
            "median_ns": int(statistics.median(values)),
            "min_ns": min(values),
            "max_ns": max(values),
        })
    processed.parent.mkdir(parents=True, exist_ok=True)
    with processed.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=summaries[0].keys())
        writer.writeheader()
        writer.writerows(summaries)
    FIGURES.mkdir(parents=True, exist_ok=True)
    checksums = {}
    labels = {"qubits": "Allocated qubits", "depth": "Circuit depth", "shots": "Shots", "trace": "Trace level (0=off, 1=instructions, 2=state)", "abstraction": "Execution path (0=direct engine, 1=full SDK/runtime)"}
    for family in labels:
        points = [(row["parameter"], row["median_ns"] / 1_000_000) for row in summaries if row["family"] == family]
        suffix = f"-{args.suffix}" if args.suffix else ""
        path = FIGURES / f"benchmark-{family}{suffix}.svg"
        path.write_text(svg_plot(points, labels[family], "Median runtime (ms)"), encoding="utf-8")
        checksums[str(path.relative_to(ROOT))] = hashlib.sha256(path.read_bytes()).hexdigest()
    checksums[str(processed.relative_to(ROOT))] = hashlib.sha256(processed.read_bytes()).hexdigest()
    (dataset / "processed/artifact-checksums.json").write_text(
        json.dumps(checksums, indent=2) + "\n", encoding="utf-8"
    )


def svg_plot(points: list[tuple[int, float]], x_label: str, y_label: str) -> str:
    width, height = 800, 460
    left, right, top, bottom = 90, 30, 30, 70
    x_max = max(x for x, _ in points) or 1
    y_max = max(y for _, y in points) or 1
    coords = []
    for x, y in points:
        px = left + (width - left - right) * x / x_max
        py = height - bottom - (height - top - bottom) * y / y_max
        coords.append((px, py, x, y))
    polyline = " ".join(f"{x:.2f},{y:.2f}" for x, y, _, _ in coords)
    marks = "".join(f'<circle cx="{x:.2f}" cy="{y:.2f}" r="4"/><text x="{x:.2f}" y="{y-10:.2f}" text-anchor="middle">{value:.3g}</text>' for x, y, _, value in coords)
    ticks = "".join(f'<text x="{x:.2f}" y="{height-bottom+24}" text-anchor="middle">{value}</text>' for x, _, value, _ in coords)
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-labelledby="title desc">
<title id="title">{y_label} versus {x_label}</title><desc id="desc">Release-build medians generated from retained Clio benchmark records.</desc>
<style>text{{font:14px sans-serif;fill:#222}} .axis{{stroke:#444;stroke-width:1.5}} polyline{{fill:none;stroke:#1769aa;stroke-width:2.5}} circle{{fill:#1769aa}}</style>
<line class="axis" x1="{left}" y1="{height-bottom}" x2="{width-right}" y2="{height-bottom}"/><line class="axis" x1="{left}" y1="{top}" x2="{left}" y2="{height-bottom}"/>
<polyline points="{polyline}"/>{marks}{ticks}
<text x="{(left+width-right)/2}" y="{height-18}" text-anchor="middle">{x_label}</text>
<text transform="translate(24 {(top+height-bottom)/2}) rotate(-90)" text-anchor="middle">{y_label}</text>
<text x="{left-8}" y="{top+5}" text-anchor="end">{y_max:.3g}</text><text x="{left-8}" y="{height-bottom+5}" text-anchor="end">0</text>
</svg>'''


if __name__ == "__main__":
    main()
