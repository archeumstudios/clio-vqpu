# Replay and Compatibility

Clio replay packages are self-contained JSON envelopes with a stable schema identifier. Each package retains the original source, typed assembled executable, complete deterministic result, engine semantic identity, RNG semantic identity, producer operating system and architecture, and a SHA-256 checksum over the semantic payload.

Verification proceeds in four stages. First, schema, engine, and RNG identities must match the running implementation. Second, the payload checksum must match. Third, rebuilding the retained source must produce the identical typed executable and source hash. Fourth, execution under the retained source directives must reproduce the complete result exactly, including measurement counts, final architectural state, and structured trace.

This contract intentionally rejects silent forward compatibility. A future engine or RNG may provide an explicit migration tool, but it cannot claim exact replay under the current identity. Producer OS and architecture are informational because deterministic same-build semantics are tested across the semantic package; performance replay is not promised.
