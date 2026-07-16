# Studio architecture

Clio Studio is a loopback HTTP application embedded in the Rust workspace. Static HTML, CSS, and JavaScript are compiled into the `clio-studio` binary. JSON endpoints call Clio SDK, Clio Resource, and Clio Replay directly. The frontend contains presentation and interaction logic only; it does not simulate gates, measurements, registers, resources, validation, or replay.

Primary execution and inspection are separate requests. `run` preserves declared shots and trace settings for results. `inspect` forces one bounded state-enabled shot for timeline debugging. This prevents the debugger from rewriting the evidence-bearing execution configuration. Replay verification occurs server-side before imported source is accepted.
