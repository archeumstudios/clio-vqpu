# Bell-state first target

`main.clio` is the first internal end-to-end workload. It is source material for the parser and conformance path; the current foundation scaffold does not yet execute it.

When implemented, the program must create `( |00> + |11> ) / sqrt(2)`, produce only correlated `00` and `11` outcomes, derive measurement from the state vector, repeat deterministically for the same supported seeded configuration, emit real instruction events, and pass analytic plus differential validation. A many-shot run should approach 50/50 statistically but is not required to contain exactly equal counts.

See `spec/architecture/first-end-to-end-checklist.md` for completion evidence.
