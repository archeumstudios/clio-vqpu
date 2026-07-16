//! Correctness validation for supported Clio execution results.

#![forbid(unsafe_code)]

use clio_runtime::ExecutionResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bell-state known-answer validation report.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BellValidationReport {
    /// Whether all required checks passed.
    pub passed: bool,
    /// Total observed shots.
    pub observed_shots: u64,
    /// Count of forbidden outcomes other than `00` and `11`.
    pub forbidden_outcomes: u64,
    /// Empirical probability of `00`.
    pub probability_00: f64,
    /// Total variation distance from the ideal Bell distribution.
    pub total_variation_distance: f64,
}

/// Validates counts against the ideal Bell distribution.
#[must_use]
#[allow(clippy::cast_precision_loss)] // Counts are bounded to 1,000,000 by runtime admission.
pub fn validate_bell_result(result: &ExecutionResult) -> BellValidationReport {
    let observed_shots: u64 = result.measurement_counts.values().sum();
    let count_00 = result.measurement_counts.get("00").copied().unwrap_or(0);
    let count_11 = result.measurement_counts.get("11").copied().unwrap_or(0);
    let forbidden_outcomes = observed_shots.saturating_sub(count_00.saturating_add(count_11));
    let denominator = observed_shots.max(1) as f64;
    let probability_00 = count_00 as f64 / denominator;
    let probability_11 = count_11 as f64 / denominator;
    let forbidden_probability = forbidden_outcomes as f64 / denominator;
    let total_variation_distance =
        0.5 * ((probability_00 - 0.5).abs() + (probability_11 - 0.5).abs() + forbidden_probability);
    BellValidationReport {
        passed: observed_shots == result.shots && observed_shots > 0 && forbidden_outcomes == 0,
        observed_shots,
        forbidden_outcomes,
        probability_00,
        total_variation_distance,
    }
}

/// GHZ known-answer distribution report for arbitrary measured width.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GhzValidationReport {
    /// Whether only all-zero and all-one outcomes were observed for every shot.
    pub passed: bool,
    /// Width of the observed bit strings.
    pub measured_qubits: usize,
    /// Total observed shots.
    pub observed_shots: u64,
    /// Count outside the two ideal GHZ outcomes.
    pub forbidden_outcomes: u64,
    /// Total variation distance from the ideal equal two-outcome distribution.
    pub total_variation_distance: f64,
}

/// Validates a fully measured GHZ result.
#[must_use]
#[allow(clippy::cast_precision_loss)] // Runtime admission bounds shot counts to exact f64 integers.
pub fn validate_ghz_result(result: &ExecutionResult) -> GhzValidationReport {
    let measured_qubits = result
        .measurement_counts
        .keys()
        .map(String::len)
        .max()
        .unwrap_or(0);
    let zeros = "0".repeat(measured_qubits);
    let ones = "1".repeat(measured_qubits);
    let observed_shots: u64 = result.measurement_counts.values().sum();
    let zero_count = result.measurement_counts.get(&zeros).copied().unwrap_or(0);
    let one_count = result.measurement_counts.get(&ones).copied().unwrap_or(0);
    let forbidden_outcomes = observed_shots.saturating_sub(zero_count.saturating_add(one_count));
    let denominator = observed_shots.max(1) as f64;
    let total_variation_distance = 0.5
        * ((zero_count as f64 / denominator - 0.5).abs()
            + (one_count as f64 / denominator - 0.5).abs()
            + forbidden_outcomes as f64 / denominator);
    GhzValidationReport {
        passed: measured_qubits >= 3 && observed_shots == result.shots && forbidden_outcomes == 0,
        measured_qubits,
        observed_shots,
        forbidden_outcomes,
        total_variation_distance,
    }
}

/// Measurement-driven branching conformance report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HybridBranchValidationReport {
    /// Whether every shot preserved measurement-to-classical agreement.
    pub passed: bool,
    /// Shots with a recorded `m0` measurement.
    pub measured_shots: u64,
    /// Shots whose `r1` write disagreed with `m0` or was missing.
    pub mismatched_shots: u64,
    /// Conditional branches observed as taken.
    pub taken_branches: u64,
    /// Conditional branches observed as not taken.
    pub not_taken_branches: u64,
}

/// Validates the trace contract of the canonical measurement-branching workload.
#[must_use]
pub fn validate_hybrid_branch_result(result: &ExecutionResult) -> HybridBranchValidationReport {
    let mut measurements = HashMap::new();
    let mut final_r1 = HashMap::new();
    let mut taken_branches = 0;
    let mut not_taken_branches = 0;

    for event in &result.trace.events {
        if let Some(measurement) = event.measurement
            && measurement.register.index() == 0
        {
            measurements.insert(event.shot, measurement.value);
        }
        if let Some(classical) = event.classical
            && classical.register.index() == 1
        {
            final_r1.insert(event.shot, classical.value);
        }
        match event.branch_taken {
            Some(true) if event.mnemonic == "JZ" => taken_branches += 1,
            Some(false) if event.mnemonic == "JZ" => not_taken_branches += 1,
            _ => {}
        }
    }

    let mismatched_shots = (0..result.shots)
        .filter(|shot| {
            let expected = measurements.get(shot).map(|value| i64::from(*value));
            expected.is_none() || final_r1.get(shot).copied() != expected
        })
        .count() as u64;
    HybridBranchValidationReport {
        passed: measurements.len() as u64 == result.shots
            && mismatched_shots == 0
            && taken_branches > 0
            && not_taken_branches > 0,
        measured_shots: measurements.len() as u64,
        mismatched_shots,
        taken_branches,
        not_taken_branches,
    }
}

/// Known-answer report for a teleportation workload with a verification measurement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TeleportationValidationReport {
    /// Whether every receiver verification bit matched the expected value.
    pub passed: bool,
    /// Expected receiver verification bit.
    pub expected_receiver_bit: bool,
    /// Total observed shots.
    pub observed_shots: u64,
    /// Shots with an incorrect receiver verification bit.
    pub failed_receiver_shots: u64,
    /// Fraction of correct receiver verification measurements.
    pub verification_success_rate: f64,
    /// Number of distinct sender measurement/correction combinations observed.
    pub correction_combinations_observed: usize,
}

/// Validates deterministic receiver verification for a canonical teleportation workload.
#[must_use]
#[allow(clippy::cast_precision_loss)] // Runtime admission bounds shot counts to exact f64 integers.
pub fn validate_teleportation_result(
    result: &ExecutionResult,
    expected_receiver_bit: bool,
) -> TeleportationValidationReport {
    let expected = if expected_receiver_bit { '1' } else { '0' };
    let observed_shots: u64 = result.measurement_counts.values().sum();
    let failed_receiver_shots = result
        .measurement_counts
        .iter()
        .filter(|(outcome, _)| !outcome.starts_with(expected))
        .map(|(_, count)| count)
        .sum();
    let correction_combinations_observed = result
        .measurement_counts
        .keys()
        .filter_map(|outcome| outcome.get(1..))
        .collect::<std::collections::HashSet<_>>()
        .len();
    let successful = observed_shots.saturating_sub(failed_receiver_shots);
    TeleportationValidationReport {
        passed: observed_shots == result.shots
            && failed_receiver_shots == 0
            && correction_combinations_observed == 4,
        expected_receiver_bit,
        observed_shots,
        failed_receiver_shots,
        verification_success_rate: successful as f64 / observed_shots.max(1) as f64,
        correction_combinations_observed,
    }
}

/// Exact Bernstein–Vazirani hidden-string recovery report.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BernsteinVaziraniValidationReport {
    /// Whether every shot recovered the hidden string.
    pub passed: bool,
    /// Declared hidden string.
    pub expected_secret: String,
    /// Total shots matching the secret.
    pub matching_shots: u64,
    /// Total observed shots.
    pub observed_shots: u64,
}

/// Validates deterministic hidden-string recovery.
#[must_use]
pub fn validate_bernstein_vazirani_result(
    result: &ExecutionResult,
    expected_secret: &str,
) -> BernsteinVaziraniValidationReport {
    let observed_shots: u64 = result.measurement_counts.values().sum();
    let matching_shots = result
        .measurement_counts
        .get(expected_secret)
        .copied()
        .unwrap_or(0);
    BernsteinVaziraniValidationReport {
        passed: observed_shots == result.shots && matching_shots == result.shots,
        expected_secret: expected_secret.to_owned(),
        matching_shots,
        observed_shots,
    }
}

/// Deutsch–Jozsa constant/balanced classification report.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeutschJozsaValidationReport {
    /// Whether every shot produced the expected classification.
    pub passed: bool,
    /// Expected oracle class.
    pub expected_constant: bool,
    /// Shots classified as constant by the all-zero result.
    pub constant_shots: u64,
    /// Shots classified as balanced by a nonzero result.
    pub balanced_shots: u64,
    /// Total observed shots.
    pub observed_shots: u64,
}

/// Validates Deutsch–Jozsa classification.
#[must_use]
pub fn validate_deutsch_jozsa_result(
    result: &ExecutionResult,
    expected_constant: bool,
) -> DeutschJozsaValidationReport {
    let observed_shots: u64 = result.measurement_counts.values().sum();
    let constant_shots = result
        .measurement_counts
        .iter()
        .filter(|(outcome, _)| outcome.chars().all(|bit| bit == '0'))
        .map(|(_, count)| count)
        .sum();
    let balanced_shots = observed_shots.saturating_sub(constant_shots);
    let passed = observed_shots == result.shots
        && if expected_constant {
            constant_shots == result.shots
        } else {
            balanced_shots == result.shots
        };
    DeutschJozsaValidationReport {
        passed,
        expected_constant,
        constant_shots,
        balanced_shots,
        observed_shots,
    }
}

/// Known-answer report for small Grover search.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroverValidationReport {
    /// Whether the marked state met the required success threshold.
    pub passed: bool,
    /// Marked output string.
    pub marked_state: String,
    /// Marked observations.
    pub marked_shots: u64,
    /// Total observations.
    pub observed_shots: u64,
    /// Empirical marked-state probability.
    pub success_probability: f64,
}

/// Validates a small Grover workload against its marked state and threshold.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn validate_grover_result(
    result: &ExecutionResult,
    marked_state: &str,
    minimum_success: f64,
) -> GroverValidationReport {
    let observed_shots: u64 = result.measurement_counts.values().sum();
    let marked_shots = result
        .measurement_counts
        .get(marked_state)
        .copied()
        .unwrap_or(0);
    let success_probability = marked_shots as f64 / observed_shots.max(1) as f64;
    GroverValidationReport {
        passed: observed_shots == result.shots && success_probability >= minimum_success,
        marked_state: marked_state.to_owned(),
        marked_shots,
        observed_shots,
        success_probability,
    }
}

/// Exact QFT/inverse-QFT round-trip report.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QftRoundtripValidationReport {
    /// Whether every shot recovered the input basis state.
    pub passed: bool,
    /// Expected recovered state.
    pub expected_state: String,
    /// Exactly matching shots.
    pub matching_shots: u64,
    /// Total observations.
    pub observed_shots: u64,
}

/// Validates exact recovery after a QFT/inverse-QFT pair.
#[must_use]
pub fn validate_qft_roundtrip_result(
    result: &ExecutionResult,
    expected_state: &str,
) -> QftRoundtripValidationReport {
    let observed_shots = result.measurement_counts.values().sum();
    let matching_shots = result
        .measurement_counts
        .get(expected_state)
        .copied()
        .unwrap_or(0);
    QftRoundtripValidationReport {
        passed: observed_shots == result.shots && matching_shots == result.shots,
        expected_state: expected_state.to_owned(),
        matching_shots,
        observed_shots,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clio_assembler::assemble;
    use clio_resource::ResourceLimits;
    use clio_runtime::run;
    use std::fmt::Write as _;

    #[test]
    fn real_bell_execution_passes_known_answer_validation() {
        let source = ".seed 42\n.shots 1024\n.trace off\nQALLOC q0\nQALLOC q1\nQH q0\nQCX q0, q1\nQMEASURE q0, m0\nQMEASURE q1, m1\nHALT\n";
        let executable = assemble(source).expect("assemble");
        let result = run(&executable, ResourceLimits::default()).expect("run");
        let report = validate_bell_result(&result);
        assert!(report.passed);
        assert_eq!(report.forbidden_outcomes, 0);
        assert!(report.total_variation_distance < 0.1);
    }

    #[test]
    fn real_hybrid_execution_preserves_measurement_control() {
        let source = ".seed 9\n.shots 128\n.trace instructions\nQALLOC q0\nQH q0\nQMEASURE q0, m0\nMOV r0, m0\nCMP r0, 0\nJZ measured_zero\nLOADI r1, 1\nJMP done\nmeasured_zero: LOADI r1, 0\ndone: HALT\n";
        let executable = assemble(source).expect("assemble");
        let result = run(&executable, ResourceLimits::default()).expect("run");
        let report = validate_hybrid_branch_result(&result);
        assert!(report.passed);
        assert_eq!(report.measured_shots, 128);
        assert_eq!(report.mismatched_shots, 0);
    }

    #[test]
    fn ghz_widths_have_only_all_zero_or_all_one_outcomes() {
        for width in 3_u16..=5 {
            let mut source = ".seed 42\n.shots 512\n.trace off\n".to_owned();
            for qubit in 0..width {
                writeln!(source, "QALLOC q{qubit}").expect("write source");
            }
            source.push_str("QH q0\n");
            for target in 1..width {
                writeln!(source, "QCX q0, q{target}").expect("write source");
            }
            for qubit in 0..width {
                writeln!(source, "QMEASURE q{qubit}, m{qubit}").expect("write source");
            }
            source.push_str("HALT\n");
            let executable = assemble(&source).expect("assemble GHZ");
            let result = run(&executable, ResourceLimits::default()).expect("run GHZ");
            let report = validate_ghz_result(&result);
            assert!(report.passed, "GHZ-{width} must pass");
            assert_eq!(report.measured_qubits, usize::from(width));
            assert_eq!(report.forbidden_outcomes, 0);
        }
    }

    #[test]
    fn teleportation_known_states_pass_receiver_verification() {
        let cases = [
            (
                include_str!("../../../examples/teleportation/zero/main.clio"),
                false,
            ),
            (
                include_str!("../../../examples/teleportation/one/main.clio"),
                true,
            ),
            (
                include_str!("../../../examples/teleportation/plus/main.clio"),
                false,
            ),
        ];
        for (source, expected) in cases {
            let executable = assemble(source).expect("assemble teleportation");
            let result = run(&executable, ResourceLimits::default()).expect("run teleportation");
            let report = validate_teleportation_result(&result, expected);
            assert!(report.passed);
            assert_eq!(report.failed_receiver_shots, 0);
            assert_eq!(report.correction_combinations_observed, 4);
        }
    }

    #[test]
    fn bernstein_vazirani_recovers_checked_in_secrets() {
        let cases = [
            (
                include_str!("../../../examples/bernstein-vazirani/secret-1011/main.clio"),
                "1011",
            ),
            (
                include_str!("../../../examples/bernstein-vazirani/secret-00101/main.clio"),
                "00101",
            ),
        ];
        for (source, secret) in cases {
            let result = run(
                &assemble(source).expect("assemble BV"),
                ResourceLimits::default(),
            )
            .expect("run BV");
            assert!(validate_bernstein_vazirani_result(&result, secret).passed);
        }
    }

    #[test]
    fn deutsch_jozsa_classifies_constant_and_balanced_oracles() {
        let cases = [
            (
                include_str!("../../../examples/deutsch-jozsa/constant-zero/main.clio"),
                true,
            ),
            (
                include_str!("../../../examples/deutsch-jozsa/constant-one/main.clio"),
                true,
            ),
            (
                include_str!("../../../examples/deutsch-jozsa/balanced-parity/main.clio"),
                false,
            ),
        ];
        for (source, expected_constant) in cases {
            let result = run(
                &assemble(source).expect("assemble DJ"),
                ResourceLimits::default(),
            )
            .expect("run DJ");
            assert!(validate_deutsch_jozsa_result(&result, expected_constant).passed);
        }
    }

    #[test]
    fn grover_examples_amplify_their_marked_states() {
        let cases = [
            (
                include_str!("../../../examples/grover/2-qubit/main.clio"),
                "11",
                1.0,
            ),
            (
                include_str!("../../../examples/grover/3-qubit/main.clio"),
                "111",
                0.70,
            ),
        ];
        for (source, marked, threshold) in cases {
            let result = run(
                &assemble(source).expect("assemble Grover"),
                ResourceLimits::default(),
            )
            .expect("run Grover");
            assert!(validate_grover_result(&result, marked, threshold).passed);
        }
    }

    #[test]
    fn qft_example_recovers_input_exactly() {
        let source = include_str!("../../../examples/qft/3-qubit-roundtrip/main.clio");
        let result = run(
            &assemble(source).expect("assemble QFT"),
            ResourceLimits::default(),
        )
        .expect("run QFT");
        assert!(validate_qft_roundtrip_result(&result, "101").passed);
    }
}
