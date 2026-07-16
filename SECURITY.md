# Security Policy

## Current status

Clio has not reached its definitive public release. Until a public security contact is verified, do not publish suspected vulnerabilities. Contact the project lead through a private, verified Archeum Studios channel and include “Clio security” in the subject. This file will be updated with a dedicated address before launch.

## Scope

Treat Clio source (`.clio`), executables, traces, replay/result packages, benchmark inputs, archives, and project files as untrusted. Relevant reports include memory or CPU exhaustion, unchecked arithmetic, infinite execution, parser or deserializer crashes, invalid handle access, use-after-free logical qubits, path traversal, archive extraction issues, checksum bypass, frontend denial of service, and accidental host-code execution.

## Design requirements

- Checked arithmetic and bounded allocation
- Source, executable, instruction, shot, qubit, memory, trace, time, and stack limits
- Deterministic rejection or traps for illegal programs
- Safe, integrity-checked serialization with explicit compatibility rules
- Cancellation for long-running execution
- No arbitrary host code in Clio Assembly
- No secrets or private data in traces and result packages without explicit user action

## Report contents

Provide the affected revision, platform, minimal reproducer, observed impact, expected behavior, and any suggested mitigation. Avoid destructive testing against systems you do not own.
