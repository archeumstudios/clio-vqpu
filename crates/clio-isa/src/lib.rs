//! Typed Clio ISA instructions, operands, and executable metadata.

#![forbid(unsafe_code)]

use clio_core::{ClassicalRegister, MeasurementRegister, ProgramMetadata, QubitId, SourceSpan};
use serde::{Deserialize, Serialize};

/// A classical register or signed immediate input.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClassicalValue {
    /// Read a classical register.
    Register(ClassicalRegister),
    /// Use a signed immediate.
    Immediate(i64),
}

/// Checked classical binary operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClassicalBinaryOperation {
    /// Checked addition.
    Add,
    /// Checked subtraction.
    Subtract,
    /// Checked multiplication.
    Multiply,
    /// Checked division.
    Divide,
    /// Checked remainder.
    Modulo,
    /// Bitwise AND.
    And,
    /// Bitwise OR.
    Or,
    /// Bitwise XOR.
    Xor,
    /// Checked left shift.
    ShiftLeft,
    /// Arithmetic right shift.
    ShiftRight,
}

/// The executable Bell-path instruction subset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    /// Allocate a logical virtual qubit.
    QAlloc(QubitId),
    /// Reset an allocated qubit to computational zero.
    QReset(QubitId),
    /// Safely release the highest allocated qubit.
    QFree(QubitId),
    /// Apply Hadamard to an allocated qubit.
    QH(QubitId),
    /// Apply Pauli X.
    QX(QubitId),
    /// Apply Pauli Y.
    QY(QubitId),
    /// Apply Pauli Z.
    QZ(QubitId),
    /// Apply phase S.
    QS(QubitId),
    /// Apply inverse phase S.
    QSdg(QubitId),
    /// Apply phase T.
    QT(QubitId),
    /// Apply inverse phase T.
    QTdg(QubitId),
    /// Rotate around X by radians.
    QRx {
        /// Target qubit.
        qubit: QubitId,
        /// Finite angle in radians.
        angle: f64,
    },
    /// Rotate around Y by radians.
    QRy {
        /// Target qubit.
        qubit: QubitId,
        /// Finite angle in radians.
        angle: f64,
    },
    /// Rotate around Z by radians.
    QRz {
        /// Target qubit.
        qubit: QubitId,
        /// Finite angle in radians.
        angle: f64,
    },
    /// Apply controlled-X from the first operand to the second.
    QCx {
        /// Control qubit.
        control: QubitId,
        /// Target qubit.
        target: QubitId,
    },
    /// Apply controlled-Z.
    QCz {
        /// Control qubit.
        control: QubitId,
        /// Target qubit.
        target: QubitId,
    },
    /// Apply a controlled phase in radians to the `|11>` subspace.
    QCphase {
        /// Control qubit.
        control: QubitId,
        /// Target qubit.
        target: QubitId,
        /// Finite phase angle.
        angle: f64,
    },
    /// Swap two qubits.
    QSwap {
        /// First qubit.
        first: QubitId,
        /// Second qubit.
        second: QubitId,
    },
    /// Apply controlled-controlled-X.
    QCcx {
        /// First control.
        first_control: QubitId,
        /// Second control.
        second_control: QubitId,
        /// Target qubit.
        target: QubitId,
    },
    /// Measure and collapse a qubit into a measurement register.
    QMeasure {
        /// Measured qubit.
        qubit: QubitId,
        /// Destination measurement register.
        destination: MeasurementRegister,
    },
    /// Move a set measurement bit into a classical register as integer zero or one.
    MovMeasurement {
        /// Classical destination.
        destination: ClassicalRegister,
        /// Measurement source.
        source: MeasurementRegister,
    },
    /// Move one classical register into another.
    MovRegister {
        /// Destination register.
        destination: ClassicalRegister,
        /// Source register.
        source: ClassicalRegister,
    },
    /// Load a signed immediate into a classical register.
    LoadImmediate {
        /// Classical destination.
        destination: ClassicalRegister,
        /// Signed value.
        value: i64,
    },
    /// Execute a checked classical binary operation.
    ClassicalBinary {
        /// Operation.
        operation: ClassicalBinaryOperation,
        /// Destination register.
        destination: ClassicalRegister,
        /// Left register input.
        left: ClassicalRegister,
        /// Register or immediate right input.
        right: ClassicalValue,
    },
    /// Apply bitwise NOT.
    ClassicalNot {
        /// Destination register.
        destination: ClassicalRegister,
        /// Source register.
        source: ClassicalRegister,
    },
    /// Compare a classical register with a register or immediate.
    Compare {
        /// Left register.
        left: ClassicalRegister,
        /// Right input.
        right: ClassicalValue,
    },
    /// Jump unconditionally to a resolved instruction index.
    Jump {
        /// Resolved target.
        target: usize,
    },
    /// Jump when the latest comparison was equal.
    JumpZero {
        /// Resolved target.
        target: usize,
    },
    /// Jump when the latest comparison was not equal.
    JumpNotZero {
        /// Resolved target.
        target: usize,
    },
    /// Jump when the latest comparison was less.
    JumpLess {
        /// Resolved target.
        target: usize,
    },
    /// Jump when the latest comparison was greater.
    JumpGreater {
        /// Resolved target.
        target: usize,
    },
    /// Stop execution explicitly.
    Halt,
}

impl Instruction {
    /// Returns the stable source mnemonic.
    #[must_use]
    pub const fn mnemonic(&self) -> &'static str {
        match self {
            Self::QAlloc(_) => "QALLOC",
            Self::QReset(_) => "QRESET",
            Self::QFree(_) => "QFREE",
            Self::QH(_) => "QH",
            Self::QX(_) => "QX",
            Self::QY(_) => "QY",
            Self::QZ(_) => "QZ",
            Self::QS(_) => "QS",
            Self::QSdg(_) => "QSDG",
            Self::QT(_) => "QT",
            Self::QTdg(_) => "QTDG",
            Self::QRx { .. } => "QRX",
            Self::QRy { .. } => "QRY",
            Self::QRz { .. } => "QRZ",
            Self::QCx { .. } => "QCX",
            Self::QCz { .. } => "QCZ",
            Self::QCphase { .. } => "QCPHASE",
            Self::QSwap { .. } => "QSWAP",
            Self::QCcx { .. } => "QCCX",
            Self::QMeasure { .. } => "QMEASURE",
            Self::MovMeasurement { .. } | Self::MovRegister { .. } => "MOV",
            Self::LoadImmediate { .. } => "LOADI",
            Self::ClassicalBinary { operation, .. } => match operation {
                ClassicalBinaryOperation::Add => "ADD",
                ClassicalBinaryOperation::Subtract => "SUB",
                ClassicalBinaryOperation::Multiply => "MUL",
                ClassicalBinaryOperation::Divide => "DIV",
                ClassicalBinaryOperation::Modulo => "MOD",
                ClassicalBinaryOperation::And => "AND",
                ClassicalBinaryOperation::Or => "OR",
                ClassicalBinaryOperation::Xor => "XOR",
                ClassicalBinaryOperation::ShiftLeft => "SHL",
                ClassicalBinaryOperation::ShiftRight => "SHR",
            },
            Self::ClassicalNot { .. } => "NOT",
            Self::Compare { .. } => "CMP",
            Self::Jump { .. } => "JMP",
            Self::JumpZero { .. } => "JZ",
            Self::JumpNotZero { .. } => "JNZ",
            Self::JumpLess { .. } => "JLT",
            Self::JumpGreater { .. } => "JGT",
            Self::Halt => "HALT",
        }
    }
}

/// An assembled instruction and optional source location.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocatedInstruction {
    /// Instruction payload.
    pub instruction: Instruction,
    /// Source span of the mnemonic.
    pub span: Option<SourceSpan>,
}

/// A validated in-memory executable.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Executable {
    /// Internal architecture identity.
    pub architecture: String,
    /// Internal ISA compatibility identity.
    pub isa_revision: String,
    /// SHA-256 hash of normalized source bytes.
    pub source_sha256: String,
    /// Program directives.
    pub metadata: ProgramMetadata,
    /// Validated instruction stream.
    pub instructions: Vec<LocatedInstruction>,
    /// Maximum simultaneously allocated qubits proved for this straight-line subset.
    pub required_qubits: u16,
    /// Whether any resolved branch target moves backward or to itself.
    pub has_backward_branches: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bell_subset_mnemonics_are_stable() {
        assert_eq!(Instruction::QAlloc(QubitId::new(0)).mnemonic(), "QALLOC");
        assert_eq!(Instruction::Halt.mnemonic(), "HALT");
    }
}
