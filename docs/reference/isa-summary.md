# Clio ISA summary

Clio Assembly is line-oriented and case-insensitive for mnemonics and typed identifiers. Qubit zero is the least-significant basis bit.

| Family | Instructions | Semantics |
|---|---|---|
| Lifecycle | `QALLOC QRESET QFREE` | Contiguous allocation, measured reset, highest-zero release |
| Single qubit | `QX QY QZ QH QS QSDG QT QTDG QRX QRY QRZ` | Double-precision unitary state transition |
| Multi qubit | `QCX QCZ QCPHASE QSWAP QCCX` | Distinct allocated operands; controlled phase uses radians |
| Observation | `QMEASURE` | Probability sampling followed by collapse and renormalization |
| Movement | `MOV LOADI` | Measurement/register movement and signed immediates |
| Arithmetic | `ADD SUB MUL DIV MOD` | Checked signed 64-bit operations |
| Logic | `AND OR XOR NOT SHL SHR` | Signed bitwise operations with checked shifts |
| Control | `CMP JMP JZ JNZ JLT JGT HALT` | Explicit comparison state and resolved branch targets |

Program directives declare name, seed, shots, memory budget, and trace level. Debugger observations (`observe`, state snapshots, reduced states) remain outside the executable ISA so inspection cannot change program behavior.
