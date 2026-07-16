# Clio ISA Overview

Status: initial instruction inventory and semantic rules. The current executable subset is explicitly identified below; an instruction is supported only after its execution and tests land.

## Design rules

Clio ISA is typed, compact, and orthogonal. Instructions operate only on declared operand kinds: classical register (`r0..r15`), measurement register (`m0..m15`), logical qubit (`qN` within configured bounds), signed integer immediate, finite floating-point angle, or resolved instruction target. Opcodes have stable internal numeric assignments only after an ADR freezes the executable format.

Every final instruction entry must define mnemonic, opcode, operands, state transition, deterministic/probabilistic class, flags, errors, trap behavior, trace encoding, source example, and linked conformance cases.

## Families

| Family | Mnemonics | Initial semantics |
|---|---|---|
| Control | `NOP`, `HALT`, `TRAP` | No-op, explicit halt, explicit typed trap |
| Movement | `MOV`, `LOADI` | Register/measurement movement and signed immediate load |
| Arithmetic | `ADD`, `SUB`, `MUL`, `DIV`, `MOD` | Checked signed 64-bit operations |
| Logic | `AND`, `OR`, `XOR`, `NOT`, `SHL`, `SHR` | Signed-register bit operations; shifts are validated |
| Compare/branch | `CMP`, `JMP`, `JZ`, `JNZ`, `JLT`, `JGT` | Compare flags and resolved instruction targets |
| Qubit lifecycle | `QALLOC`, `QFREE`, `QRESET` | Typed logical lifecycle with engine synchronization |
| Single-qubit | `QX`, `QY`, `QZ`, `QH`, `QS`, `QSDG`, `QT`, `QTDG`, `QRX`, `QRY`, `QRZ` | Standard unitary gates using specified matrix convention |
| Multi-qubit | `QCX`, `QCZ`, `QSWAP`, `QCCX` | Distinct allocated operands and specified control order |
| Measurement | `QMEASURE` | State-derived binary measurement and collapse into `mN` |
| Synchronization | `QBARRIER` | Trace/scheduling boundary; no false physical timing claim |

## Currently executable internal subset

`QALLOC`, `QRESET`, `QFREE`, the complete listed single-qubit gate set, `QCX`, `QCZ`, `QSWAP`, `QCCX`, `QMEASURE`, classical `MOV`/`LOADI`, `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `AND`, `OR`, `XOR`, `NOT`, `SHL`, `SHR`, register/immediate `CMP`, `JMP`, `JZ`, `JNZ`, `JLT`, `JGT`, and `HALT` execute through the Rust runtime. Arithmetic is checked and illegal operations trap. Backward branches execute under instruction and wall-clock limits. Safe release currently requires highest-first computational-zero qubits and closes further allocation for that shot. Other inventory entries remain specified targets and are not advertised as executable.

`CALL` and `RET` are deferred from the accepted inventory until a bounded stack contract is approved. `QPROB`, `QEXPECT`, and `QSAMPLE` initially belong to debugger/SDK observation APIs because embedding variable-sized results in processor registers is not yet coherently defined. An ADR may introduce typed ISA forms later without advertising them prematurely.

## Operand sketches

```text
MOV   rD, rS | mS
LOADI rD, immediate
ADD   rD, rA, rB | immediate
CMP   rA, rB | immediate
JZ    label
QALLOC qN
QRX   qN, finite-angle-radians
QCX   qControl, qTarget
QMEASURE qN, mD
TRAP  nonnegative-code
```

The exact accepted arithmetic operand grammar will be frozen before parser implementation. The assembler rejects ambiguous or wrong-typed forms.

## Flags

- `ZERO`, `NEGATIVE`: written by `CMP` and, if approved, arithmetic operations; branch dependencies must be explicit in the full table.
- `MEASURED_ZERO`, `MEASURED_ONE`: mutually exclusive and written by successful `QMEASURE`.
- `RESOURCE_WARNING`, `TRACE_ENABLED`: runtime indicators, not substitutes for admission decisions.

Flags start cleared. Instructions that do not define a flag write preserve it.

## Quantum semantics

All gate names refer to matrices written in the architecture's basis and orientation convention. Angles are radians and must be finite. Multi-qubit operands must be distinct. Operations on unallocated or freed handles trap. Measurement probability is derived from normalized amplitudes; the selected outcome collapses and renormalizes the state. Numerical drift beyond specified tolerance is a numerical trap, not silently repaired unless an explicitly bounded normalization policy is specified.

## Directives are not instructions

`.program`, `.seed`, `.shots`, `.budget`, and `.trace` configure metadata and execution. Labels are assembly symbols. Breakpoints and checkpoints are debugger facilities. These do not consume program counters unless a later normative ISA entry explicitly says otherwise.

## Conformance rule

Each accepted mnemonic receives positive, boundary, wrong-type, invalid-state, resource, trace, and serialization tests as applicable. Parser recognition alone never qualifies as ISA support.
## Controlled phase

`QCPHASE control, target, angle` multiplies only computational-basis amplitudes whose control and target bits are both one by `exp(i × angle)`. Operands must be distinct and the angle must be a finite number of radians. `QCZ` is the exact special case at π. This primitive supports QFT decompositions while preserving the little-endian convention in ADR-0001.

Observation is deliberately not an ISA instruction: basis probabilities, marginals, Pauli expectations, norm, and reduced single-qubit states are debugger/SDK queries and cannot affect architectural execution.
