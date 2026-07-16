# Clio VQPU Project Contract

## Mission

Design, implement, validate, document, benchmark, and publicly release one complete, original, programmable Virtual Quantum Processing Unit under Archeum Studios. The final system must make the statement “I designed, built, validated, documented, researched, and publicly released a programmable Virtual Quantum Processing Unit” technically defensible for Advaith Praveen.

## Product identity

Clio is a software-defined processor for gate-model and hybrid classical–quantum programs. It owns its processor state, Clio ISA, Clio Assembly, executable representation, runtime lifecycle, built-in state-vector engine, resource model, trace and supported replay formats, validation system, developer interfaces, and Clio Studio.

## Non-negotiable properties

1. Clio executes through an independently functioning built-in engine; optional frameworks cannot be its execution identity.
2. Processor state, register semantics, typed virtual-qubit lifecycle, control flow, traps, and limits are explicit and testable.
3. Measurement derives from the simulated quantum state and performs specified collapse; results are never hard-coded or substituted with unconstrained random bits.
4. Every advertised instruction and feature is implemented, specified, tested, and integrated.
5. Predictable resource failures are rejected before dangerous allocation.
6. Traces and replay packages carry bounded, structured, integrity-checked provenance with documented guarantees.
7. Clio Studio reads and controls the Rust runtime; it does not duplicate or fake processor logic.
8. Research claims derive from retained evidence. Citations, benchmark values, screenshots, and test results are never fabricated.
9. Classical simulation and exponential state-vector scaling are disclosed wherever Clio is presented.
10. Untrusted source, executable, trace, replay, and benchmark input is processed with explicit bounds and safe deserialization.
11. The final research paper is a substantial, figure-rich systems monograph whose visuals and quantitative claims are generated from the real implementation and retained evidence.

## Single-release policy

Milestones in [ROADMAP.md](ROADMAP.md) are private engineering controls, not public product versions. Essential scope is not deferred to a later public Clio product. Public launch occurs only after the complete implementation, Studio, specification, benchmarks, paper, reproducibility artifacts, repository, product page, archive, and DOI pass final audit.

## Required completion evidence

Completion requires executable source-to-result workflows; instruction-linked conformance tests; known-answer, numerical, statistical, property, fuzz, and differential validation; resource and security tests; documented SDK and CLI behavior; Studio integration against real state; frozen experimental protocol; retained raw benchmark data; generated analyses; a complete paper; dependency and asset audits; release checksums; and a verified archive DOI.

## Change control

Architecture changes must update the normative specification, affected tests, examples, documentation, and an architecture decision record when they alter a durable contract. If implementation and specification conflict, work stops at the boundary until one is deliberately corrected. Unsupported behavior must fail explicitly.

## Attribution

Clio VQPU is an Archeum Studios project created and led by Advaith Praveen. Contributors retain attribution according to the repository license and contribution records.
