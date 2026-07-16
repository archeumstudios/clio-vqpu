# Clio Research Plan

Status: pre-experimental plan. It contains no results.

## Central question

Can a software-defined VQPU provide a coherent, inspectable, reproducible, and resource-aware machine-level execution model for hybrid classical–quantum programs while maintaining correctness against established simulator baselines?

## Research questions

1. Can the processor state and ISA express representative hybrid programs?
2. Does Clio produce numerically and statistically correct results within predeclared tolerances?
3. What overhead does the runtime abstraction add relative to direct Clio Engine calls?
4. What time and storage costs arise at each trace level?
5. Does admission prevent predictable state-vector memory failures without understating raw memory?
6. Under which compatibility conditions do deterministic and seeded executions replay?
7. Which useful machine states become inspectable compared with circuit-only execution?

## Pre-evaluation hypotheses

- Supported workloads will agree with analytic results and mature references within frozen tolerances.
- Raw state memory estimates will never be below `16 × 2^n` bytes and rejected workloads will not allocate the state vector.
- Runtime and tracing introduce measurable overhead; magnitude and practical acceptability are empirical questions.
- Same-build seeded reference runs will reproduce measurement sequences under the frozen RNG contract.
- Measurement-driven classical branching workloads will demonstrate hybrid expressiveness.

Hypotheses may be refined before final evaluation only, with the change and reason recorded. Conclusions will not be predetermined.

## Evidence phases

1. Literature: verified primary sources, bibliography, literature map, related-work matrix, and claim boundaries.
2. Correctness: instruction conformance, analytic known-answer programs, inverse/unitarity properties, measurement/collapse tests, and differential distributions.
3. Performance: direct engine versus full runtime across controlled qubit, depth, gate, shot, and classical-control scales.
4. Observability: trace-off, summary, instruction, and safe snapshot configurations measuring time, peak memory, and artifact size.
5. Resources/replay: boundary and adversarial admission tests, estimator error, deterministic re-execution, and explicitly scoped seeded replay.
6. Inspectability: predefined tasks and exposed-state inventory; any user study requires a separate ethical and methodological protocol.

## Workloads

Known-answer workloads include basis/gate cases, Bell, GHZ, teleportation, Deutsch–Jozsa, Bernstein–Vazirani, small Grover, small QFT, reset, repeat measurement, inverse recovery, and measurement branching. Performance families include random single-qubit, controlled-heavy, Clifford, Clifford+T, allocation stress, and classical-control-heavy programs. Generators, parameters, and seeds must be checked in.

## Baselines

- Clio Engine directly
- Complete Clio Runtime
- At least one mature state-vector simulator selected after literature and dependency review
- Trace levels compared within the same build and workload

External systems are reference baselines, not Clio's implementation source. Comparisons must use equivalent gates, endianness, precision, optimization, sampling, and measurement semantics or disclose differences.

## Metrics

Parse, validation, assembly, initialization, engine, total, validation, and replay time; peak and estimated memory; trace memory/size; instruction, gate, and shot throughput; numerical error; probability normalization; total variation distance; Jensen–Shannon divergence where justified; expectation error; confidence intervals; abstraction/trace overhead; estimator error; and failures.

## Protocol freeze before final data

Record CPU, RAM, GPU if used, OS, architecture, Rust/Python/tool versions, dependency lockfiles, compiler profile and flags, commit hash, commands, timestamps, relevant environment variables, warm-up count, repetition count, seeds, timeouts, memory limits, workload scales, outlier policy, statistical method, and analysis revision. Benchmark code and the protocol are committed before collection.

## Data lineage

Every result maps to workload source, command, configuration, seed, machine manifest, raw output, checksum, processing script, processed record, plot/table generator, and final artifact. Raw data is immutable. Final values are generated, not manually copied. Exploratory and final datasets remain distinct.

The final paper follows `research/paper/PAPER_REQUIREMENTS.md` and targets a substantial systems monograph with extensive architecture diagrams, real execution visuals, benchmark plots, validation figures, and evidence-linked tables. Visual quantity never overrides accuracy: all quantitative figures are generated from retained data, and all product visuals show the real runtime.

## Reproducibility targets

Provide a bounded smoke command for common machines and a complete research command with documented cost. The final pipeline must use only public repository content and declared dependencies. Exact cross-platform performance equality is not expected; correctness tolerances and replay compatibility are explicit.

## Threats and limitations to report

Classical exponential simulation, finite workloads and ISA, baseline equivalence, floating-point and RNG differences, warm-up and machine noise, compiler effects, trace perturbation, limited external backends, selection bias, and the gap between inspectability claims and measured usability.
