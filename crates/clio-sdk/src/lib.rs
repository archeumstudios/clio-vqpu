//! Typed Rust developer API for the executable Bell-state workflow.

#![forbid(unsafe_code)]

use clio_assembler::{assemble, disassemble as disassemble_executable};
use clio_core::{Diagnostic, TraceLevel};
use clio_isa::{Executable, Instruction};
use clio_resource::{ExecutionPlan, ResourceError, ResourceLimits, plan};
use clio_runtime::{ExecutionResult, RuntimeError, run};
use clio_trace::StateAmplitude;
use clio_validation::{
    BellValidationReport, BernsteinVaziraniValidationReport, DeutschJozsaValidationReport,
    GhzValidationReport, GroverValidationReport, HybridBranchValidationReport,
    QftRoundtripValidationReport, TeleportationValidationReport, validate_bell_result,
    validate_bernstein_vazirani_result, validate_deutsch_jozsa_result, validate_ghz_result,
    validate_grover_result, validate_hybrid_branch_result, validate_qft_roundtrip_result,
    validate_teleportation_result,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Unified SDK error preserving validation and runtime categories.
#[derive(Debug, Error)]
pub enum SdkError {
    /// Source parsing or semantic validation failed.
    #[error("program validation failed with {} diagnostic(s)", .0.len())]
    Validation(Vec<Diagnostic>),
    /// Resource planning failed.
    #[error(transparent)]
    Resource(#[from] ResourceError),
    /// Runtime execution failed.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

impl SdkError {
    /// Returns a stable machine-readable category or trap code.
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Validation(_) => "V100",
            Self::Resource(_) => "R100",
            Self::Runtime(error) => error.code(),
        }
    }
}

/// Parses, validates, and assembles source.
pub fn build(source: &str) -> Result<Executable, SdkError> {
    assemble(source).map_err(SdkError::Validation)
}

/// Produces a checked execution plan without allocating a state vector.
pub fn estimate(source: &str, limits: ResourceLimits) -> Result<ExecutionPlan, SdkError> {
    let executable = build(source)?;
    Ok(plan(&executable, limits)?)
}

/// Builds source and returns a numbered typed instruction listing.
pub fn disassemble(source: &str) -> Result<String, SdkError> {
    Ok(disassemble_executable(&build(source)?))
}

/// Executes source through parser, assembler, admission, runtime, and Clio Engine.
pub fn execute(source: &str, limits: ResourceLimits) -> Result<ExecutionResult, SdkError> {
    let executable = build(source)?;
    Ok(run(&executable, limits)?)
}

/// Runs one shot with bounded small-state snapshots enabled.
pub fn inspect(source: &str, limits: ResourceLimits) -> Result<ExecutionResult, SdkError> {
    let mut executable = build(source)?;
    executable.metadata.shots = 1;
    executable.metadata.trace_level = TraceLevel::StateSmall;
    Ok(run(&executable, limits)?)
}

/// Non-mutating final-state observations for a bounded (at most eight-qubit) program.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObservationReport {
    /// Squared state norm.
    pub norm_squared: f64,
    /// Exact basis probabilities in little-endian basis-index order.
    pub basis_probabilities: Vec<f64>,
    /// `[P(0), P(1)]` for each logical qubit.
    pub marginals: Vec<[f64; 2]>,
    /// Single-qubit `[<X>, <Y>, <Z>]` expectations.
    pub pauli_expectations: Vec<[f64; 3]>,
    /// Reduced single-qubit density matrices, encoded as `[real, imaginary]` pairs.
    pub reduced_qubit_states: Vec<[[[f64; 2]; 2]; 2]>,
}

/// Executes one unmeasured shot and computes final-state debugger observations.
pub fn observe(source: &str, limits: ResourceLimits) -> Result<ObservationReport, SdkError> {
    let result = inspect(source, limits)?;
    let snapshot = result
        .trace
        .events
        .iter()
        .rev()
        .find_map(|event| event.state_snapshot.as_ref())
        .cloned()
        .unwrap_or_default();
    Ok(observe_snapshot(&snapshot))
}

fn observe_snapshot(snapshot: &[StateAmplitude]) -> ObservationReport {
    let amplitudes = snapshot
        .iter()
        .map(|value| (value.real, value.imaginary))
        .collect::<Vec<_>>();
    let qubits = amplitudes.len().ilog2() as usize;
    let basis_probabilities = snapshot
        .iter()
        .map(|value| value.probability)
        .collect::<Vec<_>>();
    let norm_squared = basis_probabilities.iter().sum();
    let mut marginals = Vec::with_capacity(qubits);
    let mut pauli_expectations = Vec::with_capacity(qubits);
    let mut reduced_qubit_states = Vec::with_capacity(qubits);
    for qubit in 0..qubits {
        let bit = 1_usize << qubit;
        let p1 = basis_probabilities
            .iter()
            .enumerate()
            .filter(|(index, _)| index & bit != 0)
            .map(|(_, value)| value)
            .sum::<f64>();
        let mut coherence = (0.0, 0.0);
        for index in 0..amplitudes.len() {
            if index & bit == 0 {
                let a = amplitudes[index];
                let b = amplitudes[index | bit];
                coherence.0 += a.0 * b.0 + a.1 * b.1;
                coherence.1 += a.1 * b.0 - a.0 * b.1;
            }
        }
        marginals.push([norm_squared - p1, p1]);
        pauli_expectations.push([
            2.0 * coherence.0,
            -2.0 * coherence.1,
            norm_squared - 2.0 * p1,
        ]);
        reduced_qubit_states.push([
            [[norm_squared - p1, 0.0], [coherence.0, coherence.1]],
            [[coherence.0, -coherence.1], [p1, 0.0]],
        ]);
    }
    ObservationReport {
        norm_squared,
        basis_probabilities,
        marginals,
        pauli_expectations,
        reduced_qubit_states,
    }
}

/// Validation report for a recognized canonical workload family.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "report", rename_all = "kebab-case")]
pub enum SupportedValidationReport {
    /// Bell/GHZ-style correlation report.
    Bell(BellValidationReport),
    /// Multi-qubit GHZ correlation report.
    Ghz(GhzValidationReport),
    /// Measurement-driven classical-control report.
    HybridBranch(HybridBranchValidationReport),
    /// Quantum teleportation known-answer report.
    Teleportation(TeleportationValidationReport),
    /// Bernstein–Vazirani hidden-string report.
    BernsteinVazirani(BernsteinVaziraniValidationReport),
    /// Deutsch–Jozsa classification report.
    DeutschJozsa(DeutschJozsaValidationReport),
    /// Small Grover marked-state amplification report.
    Grover(GroverValidationReport),
    /// QFT followed by inverse-QFT recovery report.
    QftRoundtrip(QftRoundtripValidationReport),
}

/// Executes and validates a currently supported canonical program.
pub fn validate(
    source: &str,
    limits: ResourceLimits,
) -> Result<SupportedValidationReport, SdkError> {
    let executable = build(source)?;
    let hybrid = executable
        .instructions
        .iter()
        .any(|located| matches!(located.instruction, Instruction::MovMeasurement { .. }));
    let result = run(&executable, limits)?;
    let teleportation = executable.metadata.name.starts_with("teleport_");
    let bernstein_vazirani = executable.metadata.name.starts_with("bv_");
    let deutsch_jozsa = executable.metadata.name.starts_with("deutsch_jozsa_");
    let grover = executable.metadata.name.starts_with("grover_");
    let qft = executable.metadata.name.starts_with("qft_roundtrip_");
    Ok(if grover {
        let marked = executable.metadata.name.trim_start_matches("grover_");
        let threshold = if marked.len() == 2 { 1.0 } else { 0.70 };
        SupportedValidationReport::Grover(validate_grover_result(&result, marked, threshold))
    } else if qft {
        let expected = executable
            .metadata
            .name
            .trim_start_matches("qft_roundtrip_");
        SupportedValidationReport::QftRoundtrip(validate_qft_roundtrip_result(&result, expected))
    } else if bernstein_vazirani {
        let secret = executable.metadata.name.trim_start_matches("bv_");
        SupportedValidationReport::BernsteinVazirani(validate_bernstein_vazirani_result(
            &result, secret,
        ))
    } else if deutsch_jozsa {
        let expected_constant = executable.metadata.name.contains("constant");
        SupportedValidationReport::DeutschJozsa(validate_deutsch_jozsa_result(
            &result,
            expected_constant,
        ))
    } else if teleportation {
        let expected_receiver_bit = executable.metadata.name == "teleport_one";
        SupportedValidationReport::Teleportation(validate_teleportation_result(
            &result,
            expected_receiver_bit,
        ))
    } else if hybrid {
        SupportedValidationReport::HybridBranch(validate_hybrid_branch_result(&result))
    } else if executable.required_qubits >= 3 {
        SupportedValidationReport::Ghz(validate_ghz_result(&result))
    } else {
        SupportedValidationReport::Bell(validate_bell_result(&result))
    })
}

/// Re-exported default resource-limit type for SDK consumers.
pub use clio_resource::ResourceLimits as ExecutionLimits;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_source_to_result_workflow_executes() {
        let source = ".seed 7\n.shots 32\n.trace off\nQALLOC q0\nQALLOC q1\nQH q0\nQCX q0, q1\nQMEASURE q0, m0\nQMEASURE q1, m1\nHALT\n";
        let result = execute(source, ResourceLimits::default()).expect("execute Bell");
        assert_eq!(result.measurement_counts.values().sum::<u64>(), 32);
    }

    #[test]
    fn inspection_enables_small_state_snapshots() {
        let result =
            inspect("QALLOC q0\nQH q0\nHALT\n", ResourceLimits::default()).expect("inspect state");
        assert!(
            result
                .trace
                .events
                .iter()
                .any(|event| event.state_snapshot.is_some())
        );
    }

    #[test]
    fn observation_reports_plus_state_without_measurement() {
        let report =
            observe("QALLOC q0\nQH q0\nHALT\n", ResourceLimits::default()).expect("observe plus");
        assert!((report.norm_squared - 1.0).abs() < 1.0e-12);
        assert!((report.marginals[0][1] - 0.5).abs() < 1.0e-12);
        assert!((report.pauli_expectations[0][0] - 1.0).abs() < 1.0e-12);
    }
}
