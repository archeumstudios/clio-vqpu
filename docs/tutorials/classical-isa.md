# Classical Instructions and Guarded Loops

Clio classical registers are signed 64-bit integers initialized to zero. Arithmetic uses checked semantics: overflow, division or modulo by zero, and invalid shift counts trap deterministically. Bitwise operations act on the signed register bit representation, and right shift is arithmetic.

Binary instructions use `OP destination, left-register, right-register-or-immediate`. `CMP` stores explicit signed comparison state consumed by `JZ`, `JNZ`, `JLT`, and `JGT`. A conditional branch before `CMP` traps.

Backward branches are supported under global instruction and wall-clock budgets:

```clio
LOADI r0, 0
loop:
ADD r0, r0, 1
CMP r0, 5
JLT loop
HALT
```

Configure CLI safeguards with `--instruction-limit N` and `--time-limit-ms N`. Backward-branch programs receive conservative trace/resource admission because their iteration count is not statically assumed.
