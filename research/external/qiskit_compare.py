#!/usr/bin/env python3
"""Exact Clio/Qiskit state-vector comparison under matched little-endian semantics."""

import csv
import json
import math
import subprocess
import sys
from pathlib import Path

from qiskit import QuantumCircuit
from qiskit.quantum_info import Statevector

ROOT = Path(__file__).resolve().parents[2]
CLIO = ROOT / "target/debug/clio"
OUTPUT = ROOT / "research/benchmarks/external"


def cases():
    bell = QuantumCircuit(2); bell.h(0); bell.cx(0, 1)
    ghz = QuantumCircuit(3); ghz.h(0); ghz.cx(0, 1); ghz.cx(0, 2)
    grover = QuantumCircuit(2); grover.h([0, 1]); grover.cz(0, 1)
    for q in range(2): grover.h(q); grover.x(q)
    grover.cz(0, 1)
    for q in range(2): grover.x(q); grover.h(q)
    qft = QuantumCircuit(3); qft.x(0); qft.x(2)
    qft.h(0); qft.cp(math.pi/2, 1, 0); qft.cp(math.pi/4, 2, 0); qft.h(1); qft.cp(math.pi/2, 2, 1); qft.h(2); qft.swap(0, 2)
    qft.swap(0, 2); qft.h(2); qft.cp(-math.pi/2, 2, 1); qft.h(1); qft.cp(-math.pi/4, 2, 0); qft.cp(-math.pi/2, 1, 0); qft.h(0)
    bv = QuantumCircuit(5); bv.x(4)
    for q in range(5): bv.h(q)
    for q in [0, 1, 3]: bv.cx(q, 4)
    for q in range(4): bv.h(q)
    dj = QuantumCircuit(4); dj.x(3)
    for q in range(4): dj.h(q)
    for q in range(3): dj.cx(q, 3)
    for q in range(3): dj.h(q)
    return [
        ("bell", bell, "QALLOC q0\nQALLOC q1\nQH q0\nQCX q0, q1\nHALT\n"),
        ("ghz3", ghz, "QALLOC q0\nQALLOC q1\nQALLOC q2\nQH q0\nQCX q0, q1\nQCX q0, q2\nHALT\n"),
        ("grover2", grover, strip_measurements("examples/grover/2-qubit/main.clio")),
        ("qft_roundtrip", qft, strip_measurements("examples/qft/3-qubit-roundtrip/main.clio")),
        ("bv_1011", bv, strip_measurements("examples/bernstein-vazirani/secret-1011/main.clio")),
        ("deutsch_jozsa_balanced", dj, strip_measurements("examples/deutsch-jozsa/balanced-parity/main.clio")),
    ]


def strip_measurements(relative):
    lines = (ROOT / relative).read_text(encoding="utf-8").splitlines()
    return "\n".join(line for line in lines if not line.startswith("QMEASURE") and not line.startswith(".shots") and not line.startswith(".trace")) + "\n"


def clio_state(source):
    temporary = OUTPUT / "comparison-input.clio"
    temporary.write_text(source, encoding="utf-8")
    completed = subprocess.run([str(CLIO), "inspect", str(temporary)], check=True, capture_output=True, text=True)
    snapshots = json.loads(completed.stdout)
    final = snapshots[-1][2]
    return [complex(item["real"], item["imaginary"]) for item in final]


def metrics(clio, qiskit):
    pivot = next((i for i, (a, b) in enumerate(zip(clio, qiskit)) if abs(a) > 1e-14 and abs(b) > 1e-14), None)
    phase = clio[pivot] / qiskit[pivot] if pivot is not None else 1
    if abs(phase): phase /= abs(phase)
    amplitude_error = max(abs(a - phase*b) for a, b in zip(clio, qiskit))
    p = [abs(value)**2 for value in clio]; q = [abs(value)**2 for value in qiskit]
    tvd = 0.5 * sum(abs(a-b) for a, b in zip(p, q))
    midpoint = [(a+b)/2 for a, b in zip(p, q)]
    def kl(left, right): return sum(a * math.log2(a/b) for a, b in zip(left, right) if a > 0 and b > 0)
    jsd = max(0.0, 0.5 * kl(p, midpoint) + 0.5 * kl(q, midpoint))
    return amplitude_error, tvd, jsd, abs(sum(p)-1), abs(sum(q)-1)


def main():
    if not CLIO.exists():
        sys.exit("build target/debug/clio before running comparison")
    OUTPUT.mkdir(parents=True, exist_ok=True)
    rows = []
    for name, circuit, source in cases():
        external = list(Statevector.from_instruction(circuit).data)
        internal = clio_state(source)
        amplitude_error, tvd, jsd, clio_norm_error, qiskit_norm_error = metrics(internal, external)
        rows.append({"schema":"clio-qiskit-comparison-1", "algorithm":name, "qubits":circuit.num_qubits, "max_amplitude_error":float(amplitude_error), "total_variation_distance":float(tvd), "jensen_shannon_divergence":float(jsd), "clio_norm_error":float(clio_norm_error), "qiskit_norm_error":float(qiskit_norm_error), "passed":bool(amplitude_error <= 1e-12 and tvd <= 1e-12)})
    (OUTPUT / "comparison-input.clio").unlink(missing_ok=True)
    path = OUTPUT / "qiskit-comparison.csv"
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=rows[0].keys()); writer.writeheader(); writer.writerows(rows)
    (OUTPUT / "qiskit-environment.json").write_text(json.dumps({"schema":"clio-external-environment-1", "qiskit_version":__import__("qiskit").__version__, "python":sys.version, "endianness":"q0 is least-significant basis bit", "global_phase":"aligned before amplitude error"}, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(rows, indent=2))
    if not all(row["passed"] for row in rows): sys.exit(1)


if __name__ == "__main__": main()
