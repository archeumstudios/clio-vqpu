# Clio Studio

Clio Studio is a local visual processor laboratory compiled into the Rust workspace. Its HTTP server binds to loopback by default and embeds all HTML, CSS, and JavaScript. There are no external assets, remote services, accounts, or telemetry. A restrictive content-security policy allows only same-origin resources.

The workspace centers the assembly editor and execution timeline. Canonical examples can be loaded without leaving the application. Build, run, and validate controls call Clio SDK endpoints. The editor reports line and column and retains local working source. Keyboard execution uses the conventional command/control-enter gesture.

The timeline exposes instruction step, mnemonic, measurement, branch outcome, and lifecycle transition. A scrubber selects one real trace event and updates the complete bounded state-vector view. Basis rows show complex amplitude and probability. The processor inspector displays status, program counter, backend, seed, shot, virtual qubits, classical registers, measurements, and resource plan. Results display empirical counts and typed validation reports.

Replay export creates a real checksummed package. Import performs server-side checksum, identity, executable, and exact-result verification before loading source. Studio contains no duplicate quantum simulator. Presentation logic never fabricates amplitudes or reimplements gate semantics.

The current product is a desktop-style local web application rather than a signed native shell. This preserves a small trusted surface and cross-platform operation but delegates window integration to the browser. Packaging includes the Studio binary and launch instructions.
