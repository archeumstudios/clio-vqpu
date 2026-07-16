# Resource and Trace Evaluation

The admission model computes exact raw state-vector bytes as `16 × 2^n`, adds runtime headroom, and estimates retained trace storage before allocation. The evaluation serializes actual traces for the same admitted workload and records child-process peak resident memory in platform-native units.

The first probe revealed that the original 192-byte per-event trace estimate was not conservative for JSON instruction and state-snapshot events. Instruction traces used roughly 300 KB against a 172 KB estimate, while bounded state traces used roughly 528 KB. This negative result caused the per-event bound to be raised to 640 bytes. Re-evaluation estimates 573,440 bytes: 1.91 times the measured instruction trace and 1.09 times the measured state trace. The corrected estimator therefore preserves headroom for both tested modes.

Trace-off JSON still contains a small empty trace envelope, so its serialized size is nonzero even though admitted trace-event storage is zero. Peak RSS values are useful within the recorded macOS environment but are not equated with heap allocation or portable across operating systems. Broader workload shapes remain a threat to the fixed per-event bound and should be covered by future adversarial trace-size tests.
