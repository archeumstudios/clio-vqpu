//! Bounded structured execution trace model for the first executable path.

#![forbid(unsafe_code)]

use clio_core::{ClassicalRegister, MeasurementRegister, QubitId, SourceSpan, TraceLevel};
use serde::{Deserialize, Serialize};

/// Trace-visible processor lifecycle status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TraceStatus {
    /// Ready to run.
    Ready,
    /// Executing an instruction.
    Running,
    /// Completed by fall-through.
    Completed,
    /// Stopped by `HALT`.
    Halted,
    /// Failed with a typed runtime trap.
    Trapped,
}

/// A changed measurement register.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MeasurementChange {
    /// Destination register.
    pub register: MeasurementRegister,
    /// Measured bit.
    pub value: bool,
}

/// A changed classical register.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassicalChange {
    /// Destination register.
    pub register: ClassicalRegister,
    /// New signed value.
    pub value: i64,
}

/// One amplitude in a bounded state snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateAmplitude {
    /// Basis-array index under ADR-0001.
    pub basis_index: usize,
    /// Real component.
    pub real: f64,
    /// Imaginary component.
    pub imaginary: f64,
    /// Squared magnitude.
    pub probability: f64,
}

/// Trace-visible signed comparison state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TraceComparison {
    /// Left was less than right.
    Less,
    /// Operands were equal.
    Equal,
    /// Left was greater than right.
    Greater,
}

/// One real processor instruction transition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Shot index, starting at zero.
    pub shot: u64,
    /// Monotonic logical step across this run.
    pub step: u64,
    /// Program counter before execution.
    pub program_counter: usize,
    /// Executed mnemonic.
    pub mnemonic: String,
    /// Optional source location.
    pub source_span: Option<SourceSpan>,
    /// Status before execution.
    pub status_before: TraceStatus,
    /// Status after execution.
    pub status_after: TraceStatus,
    /// Allocated logical qubit, when applicable.
    pub allocated_qubit: Option<QubitId>,
    /// Released logical qubit, when applicable.
    pub freed_qubit: Option<QubitId>,
    /// Sampled value encountered while resetting, when applicable.
    pub reset_outcome: Option<bool>,
    /// Measurement mutation, when applicable.
    pub measurement: Option<MeasurementChange>,
    /// Classical register mutation, when applicable.
    pub classical: Option<ClassicalChange>,
    /// Branch decision, when applicable.
    pub branch_taken: Option<bool>,
    /// Comparison/flag state after the instruction.
    pub comparison: Option<TraceComparison>,
    /// Complete state for at most eight qubits at state-enabled trace levels.
    pub state_snapshot: Option<Vec<StateAmplitude>>,
}

/// Version-identified trace produced by the runtime.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Internal trace schema identifier.
    pub format: String,
    /// Requested detail level.
    pub level: TraceLevel,
    /// Events retained within the admitted bound.
    pub events: Vec<TraceEvent>,
}

impl ExecutionTrace {
    /// Creates an empty trace.
    #[must_use]
    pub fn new(level: TraceLevel) -> Self {
        Self {
            format: "clio-trace-bell-path-1".to_owned(),
            level,
            events: Vec::new(),
        }
    }

    /// Records an event unless tracing is off.
    pub fn push(&mut self, event: TraceEvent) {
        if self.level != TraceLevel::Off {
            self.events.push(event);
        }
    }
}
