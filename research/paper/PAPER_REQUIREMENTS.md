# Clio Research Paper Requirements

The final paper is a major research artifact released with the complete product. It must be technically substantial, visually rich, reproducible, and based only on the finished implementation and retained experimental evidence.

## Intended scale

Target a full-length systems research monograph rather than a short product paper: approximately 60–100 pages including references and appendices, when the available implementation and evidence justify that length. Page count is not a substitute for substance. Repetition, inflated prose, fabricated findings, and decorative filler are prohibited.

The main narrative should remain readable while appendices preserve the complete ISA, grammar, opcode contracts, conformance mapping, workload definitions, extended results, environment manifests, and reproduction instructions.

## Required visual program

The final paper must include publication-quality, consistently styled visuals covering:

1. Complete Clio system architecture
2. Processor-state model
3. Source-to-execution pipeline
4. Fetch/decode/execute lifecycle
5. Virtual-qubit lifecycle and logical mapping
6. State-vector layout and basis ordering
7. Gate application indexing
8. Measurement probability, sampling, collapse, and renormalization
9. Measurement-driven classical branch flow
10. Trace event and timeline model
11. Replay/result package structure
12. Resource-admission decision flow
13. Validation and differential-testing pipeline
14. Clio Studio frontend/backend architecture
15. Benchmark and evidence-generation pipeline
16. Representative real execution traces
17. Runtime versus qubit count
18. Raw and estimated memory versus qubit count
19. Runtime versus circuit depth
20. Runtime versus shots
21. Runtime abstraction overhead
22. Trace runtime and storage overhead
23. Resource-estimator accuracy
24. Numerical error and normalization
25. Distribution-distance comparisons
26. Failure and rejection behavior

Architecture figures must be generated from the real design. Quantitative plots must be generated from retained raw data by checked-in scripts. Screenshots must show the real product connected to the Rust runtime. Placeholder values, manually typed benchmark series, and fictional interface states are forbidden.

## Visual formats and accessibility

- Prefer editable SVG, PDF, Mermaid source, or plotting source for diagrams and charts.
- Use vector output in the paper where practical and high-resolution raster output only where necessary.
- Maintain a shared color, typography, line-weight, and annotation system.
- Make charts readable in grayscale and for common color-vision deficiencies.
- Provide descriptive captions, units, sample counts, uncertainty, and source-data identifiers.
- Supply concise alt text or long descriptions for major figures.
- Never use misleading axes, omitted baselines, or visually exaggerated differences.

## Evidence linkage

Every table and quantitative figure receives a stable artifact identifier and a manifest entry containing:

- research question and claim supported;
- generating script and exact command;
- raw and processed data paths;
- workload, seed, repetitions, and statistical method;
- machine/environment manifest;
- Clio commit hash;
- output checksum;
- verification status and reviewer.

No result enters the abstract, conclusion, figure, or table until its evidence linkage is complete.

## Depth expectations

The paper must explain design alternatives and trade-offs, not merely describe code. It must include complete related work, precise semantics, algorithms or pseudocode where useful, numerical methodology, experimental design, negative results, threats to validity, security boundaries, responsible claims, limitations, and reproducibility instructions.

The paper and visuals are developed alongside verified subsystems, but final quantitative claims are written only after the evaluation protocol and benchmark implementation are frozen.
