# Preliminary Experimental Results

This chapter records smoke evidence, not final performance conclusions. The retained dataset contains 50 release-build measurements spanning ten qubit configurations, five depth configurations, five shot configurations, three trace configurations, and direct-engine versus full-runtime paths, each with two recorded repetitions after warm-up.

The generated figures report medians from the raw CSV. Qubit scaling combines fixed depth with exponentially growing state memory. Depth and shot studies isolate repeated gate execution and repeated complete program execution respectively. The trace study compares off, instruction, and bounded state-snapshot modes for the same Bell workload. Exact measured values remain in the processed CSV and are never manually transcribed into claims.

These results are sufficient to validate the evidence pipeline and reveal gross regressions. They are insufficient for stable overhead ratios, cross-machine claims, external-simulator comparisons, or inferential statistics. Final evaluation requires the ten-repetition protocol, a clean committed revision, additional peak-memory and trace-size instrumentation, and a mature external baseline under matched semantics.
