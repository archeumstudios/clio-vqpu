//! Semantic validation and assembly for the Bell-path Clio ISA subset.

#![forbid(unsafe_code)]

use clio_core::{
    ClassicalRegister, Diagnostic, MeasurementRegister, ProgramMetadata, QubitId, TraceLevel,
};
use clio_isa::{
    ClassicalBinaryOperation, ClassicalValue, Executable, Instruction, LocatedInstruction,
};
use clio_parser::{ParsedInstruction, parse};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write as _,
    str::FromStr,
};

/// Parses, validates, and assembles source into a typed executable.
pub fn assemble(source: &str) -> Result<Executable, Vec<Diagnostic>> {
    let parsed = parse(source)?;
    let mut diagnostics = Vec::new();
    let metadata = parse_metadata(&parsed.directives, &mut diagnostics);
    let mut labels = HashMap::new();
    for label in &parsed.labels {
        if labels
            .insert(label.name.as_str(), label.instruction_index)
            .is_some()
        {
            diagnostics.push(Diagnostic::error(
                "E107",
                format!("duplicate label `{}`", label.name),
                label.span,
            ));
        }
    }
    let mut allocated = HashSet::new();
    let mut allocation_closed = false;
    let mut instructions = Vec::with_capacity(parsed.instructions.len());
    let mut required_qubits = 0_u16;

    for parsed_instruction in &parsed.instructions {
        match assemble_instruction(
            parsed_instruction,
            &labels,
            &mut allocated,
            &mut allocation_closed,
            &mut required_qubits,
        ) {
            Ok(instruction) => instructions.push(LocatedInstruction {
                instruction,
                span: Some(parsed_instruction.span),
            }),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    if instructions.is_empty() {
        diagnostics.push(Diagnostic {
            code: "E100".to_owned(),
            message: "program contains no instructions".to_owned(),
            span: None,
            help: Some("add at least HALT".to_owned()),
        });
    }

    if diagnostics.is_empty() {
        let has_backward_branches = instructions.iter().enumerate().any(|(index, located)| {
            branch_target(&located.instruction).is_some_and(|target| target <= index)
        });
        Ok(Executable {
            architecture: "clio-vqpu".to_owned(),
            isa_revision: "hybrid-path-1".to_owned(),
            source_sha256: format!("{:x}", Sha256::digest(source.as_bytes())),
            metadata,
            instructions,
            required_qubits,
            has_backward_branches,
        })
    } else {
        Err(diagnostics)
    }
}

/// Produces a stable human-readable numbered instruction listing.
#[must_use]
pub fn disassemble(executable: &Executable) -> String {
    let mut output = String::new();
    for (program_counter, located) in executable.instructions.iter().enumerate() {
        let operands = instruction_operands(&located.instruction);
        writeln!(
            output,
            "{program_counter:04} {:<10} {operands}",
            located.instruction.mnemonic()
        )
        .expect("writing to String cannot fail");
    }
    output
}

fn instruction_operands(instruction: &Instruction) -> String {
    match instruction {
        Instruction::QAlloc(q)
        | Instruction::QReset(q)
        | Instruction::QFree(q)
        | Instruction::QH(q)
        | Instruction::QX(q)
        | Instruction::QY(q)
        | Instruction::QZ(q)
        | Instruction::QS(q)
        | Instruction::QSdg(q)
        | Instruction::QT(q)
        | Instruction::QTdg(q) => q.to_string(),
        Instruction::QRx { qubit, angle }
        | Instruction::QRy { qubit, angle }
        | Instruction::QRz { qubit, angle } => format!("{qubit}, {angle}"),
        Instruction::QCx { control, target } | Instruction::QCz { control, target } => {
            format!("{control}, {target}")
        }
        Instruction::QCphase {
            control,
            target,
            angle,
        } => format!("{control}, {target}, {angle}"),
        Instruction::QSwap { first, second } => format!("{first}, {second}"),
        Instruction::QCcx {
            first_control,
            second_control,
            target,
        } => format!("{first_control}, {second_control}, {target}"),
        Instruction::QMeasure { qubit, destination } => format!("{qubit}, {destination}"),
        Instruction::MovMeasurement {
            destination,
            source,
        } => format!("{destination}, {source}"),
        Instruction::MovRegister {
            destination,
            source,
        }
        | Instruction::ClassicalNot {
            destination,
            source,
        } => format!("{destination}, {source}"),
        Instruction::LoadImmediate { destination, value } => format!("{destination}, {value}"),
        Instruction::ClassicalBinary {
            destination,
            left,
            right,
            ..
        } => format!("{destination}, {left}, {}", display_value(*right)),
        Instruction::Compare { left, right } => format!("{left}, {}", display_value(*right)),
        Instruction::Jump { target }
        | Instruction::JumpZero { target }
        | Instruction::JumpNotZero { target }
        | Instruction::JumpLess { target }
        | Instruction::JumpGreater { target } => format!("@{target}"),
        Instruction::Halt => String::new(),
    }
}

fn display_value(value: ClassicalValue) -> String {
    match value {
        ClassicalValue::Register(register) => register.to_string(),
        ClassicalValue::Immediate(immediate) => immediate.to_string(),
    }
}

fn branch_target(instruction: &Instruction) -> Option<usize> {
    match instruction {
        Instruction::Jump { target }
        | Instruction::JumpZero { target }
        | Instruction::JumpNotZero { target }
        | Instruction::JumpLess { target }
        | Instruction::JumpGreater { target } => Some(*target),
        _ => None,
    }
}

fn parse_metadata(
    directives: &[clio_parser::Directive],
    diagnostics: &mut Vec<Diagnostic>,
) -> ProgramMetadata {
    let mut metadata = ProgramMetadata::default();
    let mut seen_directives = HashSet::new();
    for directive in directives {
        if !seen_directives.insert(directive.name.as_str()) {
            diagnostics.push(Diagnostic::error(
                "E101",
                format!("duplicate .{} directive", directive.name),
                directive.span,
            ));
            continue;
        }
        match directive.name.as_str() {
            "program" => metadata.name.clone_from(&directive.value),
            "seed" => match directive.value.parse() {
                Ok(seed) => metadata.seed = seed,
                Err(_) => diagnostics.push(Diagnostic::error(
                    "E102",
                    "seed must be an unsigned 64-bit integer",
                    directive.span,
                )),
            },
            "shots" => match directive.value.parse::<u64>() {
                Ok(0) | Err(_) => diagnostics.push(Diagnostic::error(
                    "E103",
                    "shots must be a positive unsigned integer",
                    directive.span,
                )),
                Ok(shots) => metadata.shots = shots,
            },
            "trace" => match directive.value.to_ascii_lowercase().as_str() {
                "off" => metadata.trace_level = TraceLevel::Off,
                "summary" => metadata.trace_level = TraceLevel::Summary,
                "instructions" => metadata.trace_level = TraceLevel::Instructions,
                "state-small" => metadata.trace_level = TraceLevel::StateSmall,
                "full-debug" => metadata.trace_level = TraceLevel::FullDebug,
                _ => diagnostics.push(Diagnostic::error(
                    "E104",
                    "unknown trace level",
                    directive.span,
                )),
            },
            "budget" => match parse_memory_budget(&directive.value) {
                Some(bytes) => metadata.memory_budget_bytes = Some(bytes),
                None => diagnostics.push(
                    Diagnostic::error(
                        "E105",
                        "budget must have the form memory=<positive size>",
                        directive.span,
                    )
                    .with_help("use bytes, KB, MB, GB, KiB, MiB, or GiB"),
                ),
            },
            _ => diagnostics.push(Diagnostic::error(
                "E106",
                format!("unknown directive .{}", directive.name),
                directive.span,
            )),
        }
    }
    metadata
}

fn parse_memory_budget(value: &str) -> Option<u64> {
    let raw = value.strip_prefix("memory=")?;
    let digit_count = raw.bytes().take_while(u8::is_ascii_digit).count();
    let (number, suffix) = raw.split_at(digit_count);
    let number = number.parse::<u64>().ok()?;
    let multiplier = match suffix.to_ascii_lowercase().as_str() {
        "" | "b" | "bytes" => 1,
        "kb" => 1_000,
        "mb" => 1_000_000,
        "gb" => 1_000_000_000,
        "kib" => 1 << 10,
        "mib" => 1 << 20,
        "gib" => 1 << 30,
        _ => return None,
    };
    number.checked_mul(multiplier).filter(|bytes| *bytes > 0)
}

fn assemble_instruction(
    parsed: &ParsedInstruction,
    labels: &HashMap<&str, usize>,
    allocated: &mut HashSet<QubitId>,
    allocation_closed: &mut bool,
    required_qubits: &mut u16,
) -> Result<Instruction, Diagnostic> {
    let wrong_arity = |expected: usize| {
        Diagnostic::error(
            "E110",
            format!(
                "{} expects {expected} operand(s), found {}",
                parsed.mnemonic,
                parsed.operands.len()
            ),
            parsed.span,
        )
    };
    match parsed.mnemonic.as_str() {
        "QALLOC" => {
            if parsed.operands.len() != 1 {
                return Err(wrong_arity(1));
            }
            let qubit = parse_qubit(&parsed.operands[0], parsed)?;
            if *allocation_closed {
                return Err(Diagnostic::error(
                    "E125",
                    "allocation after QFREE is not supported by the initial lifecycle model",
                    parsed.span,
                ));
            }
            if qubit.index() != usize::from(*required_qubits) {
                return Err(Diagnostic::error(
                    "E111",
                    format!("expected contiguous allocation of q{required_qubits}"),
                    parsed.span,
                ));
            }
            if !allocated.insert(qubit) {
                return Err(Diagnostic::error(
                    "E112",
                    format!("{qubit} is already allocated"),
                    parsed.span,
                ));
            }
            *required_qubits = required_qubits.saturating_add(1);
            Ok(Instruction::QAlloc(qubit))
        }
        "QRESET" => {
            if parsed.operands.len() != 1 {
                return Err(wrong_arity(1));
            }
            Ok(Instruction::QReset(parse_allocated(
                &parsed.operands[0],
                parsed,
                allocated,
            )?))
        }
        "QFREE" => {
            if parsed.operands.len() != 1 {
                return Err(wrong_arity(1));
            }
            let qubit = parse_allocated(&parsed.operands[0], parsed, allocated)?;
            let highest_allocated = allocated.iter().map(|candidate| candidate.index()).max();
            if highest_allocated != Some(qubit.index()) {
                return Err(Diagnostic::error(
                    "E126",
                    "QFREE currently requires the highest mapped qubit",
                    parsed.span,
                ));
            }
            allocated.remove(&qubit);
            *allocation_closed = true;
            Ok(Instruction::QFree(qubit))
        }
        "QH" => {
            if parsed.operands.len() != 1 {
                return Err(wrong_arity(1));
            }
            let qubit = parse_allocated(&parsed.operands[0], parsed, allocated)?;
            Ok(Instruction::QH(qubit))
        }
        "QX" | "QY" | "QZ" | "QS" | "QSDG" | "QT" | "QTDG" => {
            if parsed.operands.len() != 1 {
                return Err(wrong_arity(1));
            }
            let qubit = parse_allocated(&parsed.operands[0], parsed, allocated)?;
            Ok(match parsed.mnemonic.as_str() {
                "QX" => Instruction::QX(qubit),
                "QY" => Instruction::QY(qubit),
                "QZ" => Instruction::QZ(qubit),
                "QS" => Instruction::QS(qubit),
                "QSDG" => Instruction::QSdg(qubit),
                "QT" => Instruction::QT(qubit),
                "QTDG" => Instruction::QTdg(qubit),
                _ => unreachable!("matched fixed gate mnemonic"),
            })
        }
        "QRX" | "QRY" | "QRZ" => {
            if parsed.operands.len() != 2 {
                return Err(wrong_arity(2));
            }
            let qubit = parse_allocated(&parsed.operands[0], parsed, allocated)?;
            let angle = parse_angle(&parsed.operands[1], parsed)?;
            Ok(match parsed.mnemonic.as_str() {
                "QRX" => Instruction::QRx { qubit, angle },
                "QRY" => Instruction::QRy { qubit, angle },
                "QRZ" => Instruction::QRz { qubit, angle },
                _ => unreachable!("matched rotation mnemonic"),
            })
        }
        "QCX" => {
            if parsed.operands.len() != 2 {
                return Err(wrong_arity(2));
            }
            let control = parse_allocated(&parsed.operands[0], parsed, allocated)?;
            let target = parse_allocated(&parsed.operands[1], parsed, allocated)?;
            if control == target {
                return Err(Diagnostic::error(
                    "E113",
                    "QCX control and target must be distinct",
                    parsed.span,
                ));
            }
            Ok(Instruction::QCx { control, target })
        }
        "QCPHASE" => {
            if parsed.operands.len() != 3 { return Err(wrong_arity(3)); }
            let control = parse_allocated(&parsed.operands[0], parsed, allocated)?;
            let target = parse_allocated(&parsed.operands[1], parsed, allocated)?;
            if control == target { return Err(Diagnostic::error("E113", "QCPHASE control and target must be distinct", parsed.span)); }
            let angle = parse_angle(&parsed.operands[2], parsed)?;
            Ok(Instruction::QCphase { control, target, angle })
        }
        "QCZ" | "QSWAP" => {
            if parsed.operands.len() != 2 {
                return Err(wrong_arity(2));
            }
            let first = parse_allocated(&parsed.operands[0], parsed, allocated)?;
            let second = parse_allocated(&parsed.operands[1], parsed, allocated)?;
            if first == second {
                return Err(Diagnostic::error(
                    "E113",
                    "multi-qubit operands must be distinct",
                    parsed.span,
                ));
            }
            if parsed.mnemonic == "QCZ" {
                Ok(Instruction::QCz {
                    control: first,
                    target: second,
                })
            } else {
                Ok(Instruction::QSwap { first, second })
            }
        }
        "QCCX" => {
            if parsed.operands.len() != 3 {
                return Err(wrong_arity(3));
            }
            let first_control = parse_allocated(&parsed.operands[0], parsed, allocated)?;
            let second_control = parse_allocated(&parsed.operands[1], parsed, allocated)?;
            let target = parse_allocated(&parsed.operands[2], parsed, allocated)?;
            if first_control == second_control
                || first_control == target
                || second_control == target
            {
                return Err(Diagnostic::error(
                    "E113",
                    "QCCX operands must be distinct",
                    parsed.span,
                ));
            }
            Ok(Instruction::QCcx {
                first_control,
                second_control,
                target,
            })
        }
        "QMEASURE" => {
            if parsed.operands.len() != 2 {
                return Err(wrong_arity(2));
            }
            let qubit = parse_allocated(&parsed.operands[0], parsed, allocated)?;
            let destination = MeasurementRegister::from_str(&parsed.operands[1]).map_err(|_| {
                Diagnostic::error(
                    "E114",
                    "expected a measurement register m0..m15",
                    parsed.span,
                )
            })?;
            Ok(Instruction::QMeasure { qubit, destination })
        }
        "MOV" => {
            if parsed.operands.len() != 2 {
                return Err(wrong_arity(2));
            }
            let destination = parse_classical(&parsed.operands[0], parsed)?;
            if let Ok(source) = MeasurementRegister::from_str(&parsed.operands[1]) {
                Ok(Instruction::MovMeasurement {
                    destination,
                    source,
                })
            } else {
                Ok(Instruction::MovRegister {
                    destination,
                    source: parse_classical(&parsed.operands[1], parsed)?,
                })
            }
        }
        "LOADI" => {
            if parsed.operands.len() != 2 {
                return Err(wrong_arity(2));
            }
            Ok(Instruction::LoadImmediate {
                destination: parse_classical(&parsed.operands[0], parsed)?,
                value: parse_immediate(&parsed.operands[1], parsed)?,
            })
        }
        "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "AND" | "OR" | "XOR" | "SHL"
        | "SHR" => {
            if parsed.operands.len() != 3 {
                return Err(wrong_arity(3));
            }
            let operation = match parsed.mnemonic.as_str() {
                "ADD" => ClassicalBinaryOperation::Add,
                "SUB" => ClassicalBinaryOperation::Subtract,
                "MUL" => ClassicalBinaryOperation::Multiply,
                "DIV" => ClassicalBinaryOperation::Divide,
                "MOD" => ClassicalBinaryOperation::Modulo,
                "AND" => ClassicalBinaryOperation::And,
                "OR" => ClassicalBinaryOperation::Or,
                "XOR" => ClassicalBinaryOperation::Xor,
                "SHL" => ClassicalBinaryOperation::ShiftLeft,
                "SHR" => ClassicalBinaryOperation::ShiftRight,
                _ => unreachable!("matched classical binary mnemonic"),
            };
            Ok(Instruction::ClassicalBinary {
                operation,
                destination: parse_classical(&parsed.operands[0], parsed)?,
                left: parse_classical(&parsed.operands[1], parsed)?,
                right: parse_classical_value(&parsed.operands[2], parsed)?,
            })
        }
        "NOT" => {
            if parsed.operands.len() != 2 {
                return Err(wrong_arity(2));
            }
            Ok(Instruction::ClassicalNot {
                destination: parse_classical(&parsed.operands[0], parsed)?,
                source: parse_classical(&parsed.operands[1], parsed)?,
            })
        }
        "CMP" => {
            if parsed.operands.len() != 2 {
                return Err(wrong_arity(2));
            }
            Ok(Instruction::Compare {
                left: parse_classical(&parsed.operands[0], parsed)?,
                right: parse_classical_value(&parsed.operands[1], parsed)?,
            })
        }
        "JMP" | "JZ" | "JNZ" | "JLT" | "JGT" => {
            if parsed.operands.len() != 1 {
                return Err(wrong_arity(1));
            }
            let label = parsed.operands[0].as_str();
            let target = labels.get(label).copied().ok_or_else(|| {
                Diagnostic::error(
                    "E119",
                    format!("unresolved branch label `{label}`"),
                    parsed.span,
                )
            })?;
            match parsed.mnemonic.as_str() {
                "JMP" => Ok(Instruction::Jump { target }),
                "JZ" => Ok(Instruction::JumpZero { target }),
                "JNZ" => Ok(Instruction::JumpNotZero { target }),
                "JLT" => Ok(Instruction::JumpLess { target }),
                "JGT" => Ok(Instruction::JumpGreater { target }),
                _ => unreachable!("matched branch mnemonic"),
            }
        }
        "HALT" => {
            if !parsed.operands.is_empty() {
                return Err(wrong_arity(0));
            }
            Ok(Instruction::Halt)
        }
        _ => Err(Diagnostic::error(
            "E115",
            format!("unknown or unsupported mnemonic {}", parsed.mnemonic),
            parsed.span,
        )
        .with_help(
            "supported mnemonics include QALLOC, single-qubit gates, QCX, QMEASURE, MOV, LOADI, CMP, JMP, JZ, and HALT",
        )),
    }
}

fn parse_classical(
    value: &str,
    parsed: &ParsedInstruction,
) -> Result<ClassicalRegister, Diagnostic> {
    ClassicalRegister::from_str(value).map_err(|_| {
        Diagnostic::error(
            "E120",
            format!("expected a classical register r0..r15, found `{value}`"),
            parsed.span,
        )
    })
}

fn parse_immediate(value: &str, parsed: &ParsedInstruction) -> Result<i64, Diagnostic> {
    value.parse::<i64>().map_err(|_| {
        Diagnostic::error(
            "E121",
            format!("expected a signed 64-bit integer, found `{value}`"),
            parsed.span,
        )
    })
}

fn parse_classical_value(
    value: &str,
    parsed: &ParsedInstruction,
) -> Result<ClassicalValue, Diagnostic> {
    if let Ok(immediate) = value.parse::<i64>() {
        Ok(ClassicalValue::Immediate(immediate))
    } else {
        parse_classical(value, parsed).map(ClassicalValue::Register)
    }
}

fn parse_angle(value: &str, parsed: &ParsedInstruction) -> Result<f64, Diagnostic> {
    let angle = value.parse::<f64>().map_err(|_| {
        Diagnostic::error(
            "E123",
            format!("expected an angle in radians, found `{value}`"),
            parsed.span,
        )
    })?;
    if angle.is_finite() {
        Ok(angle)
    } else {
        Err(Diagnostic::error(
            "E124",
            "rotation angle must be finite",
            parsed.span,
        ))
    }
}

fn parse_qubit(value: &str, parsed: &ParsedInstruction) -> Result<QubitId, Diagnostic> {
    QubitId::from_str(value).map_err(|_| {
        Diagnostic::error(
            "E116",
            format!("expected a virtual-qubit reference, found `{value}`"),
            parsed.span,
        )
    })
}

fn parse_allocated(
    value: &str,
    parsed: &ParsedInstruction,
    allocated: &HashSet<QubitId>,
) -> Result<QubitId, Diagnostic> {
    let qubit = parse_qubit(value, parsed)?;
    if allocated.contains(&qubit) {
        Ok(qubit)
    } else {
        Err(Diagnostic::error(
            "E117",
            format!("{qubit} is used before allocation"),
            parsed.span,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BELL: &str = ".program bell\n.seed 42\n.shots 16\nQALLOC q0\nQALLOC q1\nQH q0\nQCX q0, q1\nQMEASURE q0, m0\nQMEASURE q1, m1\nHALT\n";

    #[test]
    fn assembles_bell_subset() {
        let executable = assemble(BELL).expect("Bell source assembles");
        assert_eq!(executable.instructions.len(), 7);
        assert_eq!(executable.required_qubits, 2);
        assert_eq!(executable.metadata.seed, 42);
    }

    #[test]
    fn rejects_wrong_measurement_destination() {
        let source = "QALLOC q0\nQMEASURE q0, r0\nHALT\n";
        let diagnostics = assemble(source).expect_err("r0 is not a measurement register");
        assert!(diagnostics.iter().any(|item| item.code == "E114"));
    }

    #[test]
    fn rejects_use_before_allocation() {
        let diagnostics = assemble("QH q0\nHALT\n").expect_err("q0 is unallocated");
        assert!(diagnostics.iter().any(|item| item.code == "E117"));
    }

    #[test]
    fn resolves_hybrid_branch_labels() {
        let source = "QALLOC q0\nQMEASURE q0, m0\nMOV r0, m0\nCMP r0, 0\nJZ zero\nLOADI r1, 1\nJMP done\nzero: LOADI r1, 0\ndone: HALT\n";
        let executable = assemble(source).expect("hybrid source assembles");
        assert_eq!(
            executable.instructions[4].instruction,
            Instruction::JumpZero { target: 7 }
        );
        assert_eq!(
            executable.instructions[6].instruction,
            Instruction::Jump { target: 8 }
        );
    }

    #[test]
    fn rejects_unknown_branch_labels() {
        let diagnostics = assemble("JMP missing\nHALT\n").expect_err("missing label");
        assert!(diagnostics.iter().any(|item| item.code == "E119"));
    }

    #[test]
    fn resolves_and_marks_backward_branches() {
        let executable = assemble("again: LOADI r0, 1\nJMP again\nHALT\n")
            .expect("backward branch assembles under runtime budgets");
        assert!(executable.has_backward_branches);
    }

    #[test]
    fn assembles_fixed_and_rotation_gates() {
        let source = "QALLOC q0\nQX q0\nQY q0\nQZ q0\nQS q0\nQSDG q0\nQT q0\nQTDG q0\nQRX q0, 0.5\nQRY q0, -1.25\nQRZ q0, 3.0\nHALT\n";
        let executable = assemble(source).expect("gate program assembles");
        assert_eq!(executable.instructions.len(), 12);
        assert!(matches!(
            executable.instructions[8].instruction,
            Instruction::QRx { angle, .. } if (angle - 0.5).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn rejects_non_finite_rotation_angles() {
        let diagnostics =
            assemble("QALLOC q0\nQRX q0, NaN\nHALT\n").expect_err("NaN angle must be rejected");
        assert!(diagnostics.iter().any(|item| item.code == "E124"));
    }

    #[test]
    fn assembles_lifecycle_and_multi_qubit_operations() {
        let source = "QALLOC q0\nQALLOC q1\nQALLOC q2\nQCZ q0, q1\nQSWAP q1, q2\nQCCX q0, q1, q2\nQRESET q2\nQFREE q2\nHALT\n";
        let executable = assemble(source).expect("lifecycle program assembles");
        assert!(matches!(
            executable.instructions[5].instruction,
            Instruction::QCcx { .. }
        ));
        assert!(matches!(
            executable.instructions[7].instruction,
            Instruction::QFree(_)
        ));
    }

    #[test]
    fn free_closes_allocation_and_prevents_use_after_free() {
        let use_after_free =
            assemble("QALLOC q0\nQFREE q0\nQH q0\nHALT\n").expect_err("freed qubit use must fail");
        assert!(use_after_free.iter().any(|item| item.code == "E117"));
        let reallocate = assemble("QALLOC q0\nQFREE q0\nQALLOC q1\nHALT\n")
            .expect_err("allocation after free must fail");
        assert!(reallocate.iter().any(|item| item.code == "E125"));
        assemble("QALLOC q0\nQALLOC q1\nQFREE q1\nQFREE q0\nHALT\n")
            .expect("sequential highest-first release is valid");
    }

    #[test]
    fn assembles_complete_classical_subset() {
        let source = "LOADI r0, 9\nMOV r1, r0\nADD r2, r0, 3\nSUB r2, r2, r1\nMUL r3, r0, r1\nDIV r4, r3, r0\nMOD r5, r3, 8\nAND r6, r0, r1\nOR r7, r0, 2\nXOR r8, r0, r1\nNOT r9, r0\nSHL r10, r0, 2\nSHR r11, r10, r1\nCMP r0, r1\nJNZ ne\nJLT lt\nJGT gt\nne: HALT\nlt: HALT\ngt: HALT\n";
        let executable = assemble(source).expect("classical subset assembles");
        assert_eq!(executable.instructions[2].instruction.mnemonic(), "ADD");
        assert_eq!(executable.instructions[14].instruction.mnemonic(), "JNZ");
    }

    #[test]
    fn disassembly_contains_numbered_typed_operands() {
        let executable = assemble("LOADI r0, 2\nADD r1, r0, 3\nHALT\n").expect("assemble");
        let listing = disassemble(&executable);
        assert!(listing.contains("0000 LOADI      r0, 2"));
        assert!(listing.contains("0001 ADD        r1, r0, 3"));
        assert!(listing.contains("0002 HALT"));
    }
}
