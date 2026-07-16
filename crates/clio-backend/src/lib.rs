//! Capability-oriented quantum execution backend contract.

#![forbid(unsafe_code)]

use clio_core::QubitId;
use thiserror::Error;

/// A backend-level single-qubit operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SingleQubitGate {
    /// Pauli X.
    X,
    /// Pauli Y.
    Y,
    /// Pauli Z.
    Z,
    /// Hadamard.
    H,
    /// Phase S.
    S,
    /// Inverse phase S.
    Sdg,
    /// Phase T.
    T,
    /// Inverse phase T.
    Tdg,
    /// X-axis rotation in radians.
    Rx(f64),
    /// Y-axis rotation in radians.
    Ry(f64),
    /// Z-axis rotation in radians.
    Rz(f64),
}

/// Errors visible at the runtime/backend boundary.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum BackendError {
    /// Allocation order violates the initial lifecycle contract.
    #[error("qubit allocation must be contiguous; expected q{expected}, found {found}")]
    NonContiguousAllocation {
        /// Required next logical index.
        expected: usize,
        /// Requested logical index.
        found: usize,
    },
    /// The logical qubit is not allocated.
    #[error("{0} is not allocated")]
    Unallocated(QubitId),
    /// A multi-qubit gate reused one operand.
    #[error("control and target must be distinct")]
    DuplicateOperand,
    /// A probability or norm is outside numerical tolerance.
    #[error("numerical state error: {0}")]
    Numerical(String),
    /// State allocation cannot be represented safely.
    #[error("state allocation is too large")]
    AllocationOverflow,
    /// Safe release currently requires the highest mapped logical qubit.
    #[error(
        "only the highest allocated qubit can be released; expected q{expected}, found {found}"
    )]
    NonHighestFree {
        /// Highest mapped qubit.
        expected: usize,
        /// Requested qubit.
        found: usize,
    },
    /// Release requires a qubit proven to be in computational zero.
    #[error("{0} cannot be released because it is not in computational |0>")]
    QubitNotZero(QubitId),
}

/// Quantum operations required by the current Clio runtime.
pub trait QuantumBackend {
    /// Allocates one contiguous logical qubit.
    fn allocate(&mut self, qubit: QubitId) -> Result<(), BackendError>;
    /// Applies a supported single-qubit gate.
    fn apply_single(&mut self, qubit: QubitId, gate: SingleQubitGate) -> Result<(), BackendError>;
    /// Applies controlled-X.
    fn controlled_x(&mut self, control: QubitId, target: QubitId) -> Result<(), BackendError>;
    /// Applies controlled-Z.
    fn controlled_z(&mut self, control: QubitId, target: QubitId) -> Result<(), BackendError>;
    /// Applies a controlled phase of `angle` radians to the `|11>` subspace.
    fn controlled_phase(
        &mut self,
        control: QubitId,
        target: QubitId,
        angle: f64,
    ) -> Result<(), BackendError>;
    /// Swaps two logical qubit states.
    fn swap(&mut self, first: QubitId, second: QubitId) -> Result<(), BackendError>;
    /// Applies controlled-controlled-X.
    fn controlled_controlled_x(
        &mut self,
        first_control: QubitId,
        second_control: QubitId,
        target: QubitId,
    ) -> Result<(), BackendError>;
    /// Returns the current probability of measuring one.
    fn probability_one(&self, qubit: QubitId) -> Result<f64, BackendError>;
    /// Collapses to a caller-selected valid outcome.
    fn collapse(&mut self, qubit: QubitId, outcome: bool) -> Result<(), BackendError>;
    /// Releases the highest mapped qubit when it is provably `|0>`.
    fn free(&mut self, qubit: QubitId) -> Result<(), BackendError>;
    /// Returns allocated qubit count.
    fn qubit_count(&self) -> usize;
}
