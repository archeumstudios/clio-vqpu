# Clio Assembly Grammar

Status: initial lexical and syntactic contract for `.clio` source. Semantic operand alternatives remain governed by the ISA specification.

## Conventions

- UTF-8 source; identifiers and mnemonics are ASCII in the initial grammar.
- Lines end with LF or CRLF.
- `;` begins a comment through end of line.
- Mnemonics and directives are case-insensitive; labels and program names are case-sensitive.
- Horizontal whitespace separates tokens. Newlines terminate directives and instructions.
- Numeric angles are finite decimal/scientific literals interpreted as radians.
- The implementation enforces configured source, line, token, identifier, instruction, and literal-size limits.

## EBNF

```ebnf
program          = { blank_line | statement_line }, EOF ;
statement_line   = ws, [ label, ws ], [ directive | instruction ], ws,
                   [ comment ], newline ;
blank_line       = ws, [ comment ], newline ;

label            = identifier, ":" ;
directive        = ".", directive_name, [ ws1, directive_value ] ;
directive_name   = "program" | "seed" | "shots" | "budget" | "trace" ;
directive_value  = string | identifier | signed_integer | key_value_list ;
key_value_list   = key_value, { ws, ",", ws, key_value } ;
key_value        = identifier, ws, "=", ws, scalar ;

instruction      = mnemonic, [ ws1, operand_list ] ;
operand_list     = operand, { ws, ",", ws, operand } ;
operand          = classical_register | measurement_register | qubit_ref
                 | signed_integer | float | identifier | string ;

classical_register   = ("r" | "R"), register_index ;
measurement_register = ("m" | "M"), register_index ;
register_index       = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7"
                     | "8" | "9" | "10" | "11" | "12" | "13" | "14" | "15" ;
qubit_ref            = ("q" | "Q"), unsigned_integer ;

signed_integer   = [ "+" | "-" ], unsigned_integer ;
unsigned_integer = digit, { digit | "_" } ;
float            = [ "+" | "-" ],
                   (digits, ".", [digits] | ".", digits | digits),
                   [ ("e" | "E"), [ "+" | "-" ], digits ] ;
digits           = digit, { digit | "_" } ;
digit            = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;

identifier       = ident_start, { ident_continue } ;
ident_start      = "A"…"Z" | "a"…"z" | "_" ;
ident_continue   = ident_start | digit | "-" ;
string           = '"', { string_character | escape }, '"' ;
escape           = "\\", ("\\" | '"' | "n" | "r" | "t") ;
comment          = ";", { any_character_except_newline } ;
ws               = { " " | "\t" } ;
ws1              = (" " | "\t"), ws ;
newline          = "\n" | "\r\n" ;
```

## Structural rules

- A label may share a line with an instruction or directive and points to that instruction; labels on otherwise empty lines point to the next instruction.
- Duplicate labels and unresolved targets are semantic errors.
- `.program` occurs at most once. `.seed`, `.shots`, `.budget`, and `.trace` occur at most once unless the final directive contract explicitly permits merging.
- Directives do not emit instructions and therefore do not change label addresses.
- Registers outside `r0..r15` and `m0..m15` are lexical/semantic diagnostics, never truncated.
- Qubit reference bounds and lifecycle correctness are semantic/runtime concerns.
- Unknown mnemonics and directives receive suggestions only when edit distance is unambiguous.

## Diagnostics

All failures carry a stable code, message, file identifier, byte span, one-based line/column rendering, explanation, and a safe correction hint when possible. Invalid UTF-8, oversized input, integer overflow, non-finite float, and token-limit failures have distinct codes. Recovery must be bounded and must not cascade indefinitely.

## Example

```clio
.program bell_state
.seed 42
.shots 1024
.budget memory=1GB
.trace instructions

QALLOC q0
QALLOC q1
QH q0
QCX q0, q1
QMEASURE q0, m0
QMEASURE q1, m1
HALT
```
