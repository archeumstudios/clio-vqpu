//! Independent built-in double-precision state-vector quantum engine.

#![forbid(unsafe_code)]

use clio_backend::{BackendError, QuantumBackend, SingleQubitGate};
use clio_core::QubitId;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};

const NUMERICAL_TOLERANCE: f64 = 1.0e-12;
const INVERSE_SQRT_2: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// Clio's built-in state-vector backend.
#[derive(Clone, Debug)]
pub struct StateVectorEngine {
    amplitudes: Vec<Complex64>,
    qubits: usize,
}

/// One Pauli factor in a tensor-product observable.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Pauli {
    /// Identity.
    I,
    /// Pauli X.
    X,
    /// Pauli Y.
    Y,
    /// Pauli Z.
    Z,
}

/// Reduced density matrix of one qubit, in computational-basis order.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReducedQubitState {
    /// Real components of the 2x2 matrix.
    pub real: [[f64; 2]; 2],
    /// Imaginary components of the 2x2 matrix.
    pub imaginary: [[f64; 2]; 2],
}

impl Default for StateVectorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateVectorEngine {
    /// Creates the zero-qubit scalar state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            amplitudes: vec![Complex64::new(1.0, 0.0)],
            qubits: 0,
        }
    }

    /// Returns a read-only amplitude snapshot for bounded inspection callers.
    #[must_use]
    pub fn amplitudes(&self) -> &[Complex64] {
        &self.amplitudes
    }

    /// Returns the squared norm.
    #[must_use]
    pub fn norm_sqr(&self) -> f64 {
        self.amplitudes.iter().map(Complex64::norm_sqr).sum()
    }

    /// Returns exact computational-basis probabilities in basis-index order.
    #[must_use]
    pub fn basis_probabilities(&self) -> Vec<f64> {
        self.amplitudes.iter().map(Complex64::norm_sqr).collect()
    }

    /// Returns `[P(0), P(1)]` for one qubit.
    pub fn marginal_probabilities(&self, qubit: QubitId) -> Result<[f64; 2], BackendError> {
        let one = self.probability_one(qubit)?;
        Ok([1.0 - one, one])
    }

    /// Returns the expectation of a tensor product; omitted qubits are identity.
    pub fn pauli_expectation(&self, factors: &[(QubitId, Pauli)]) -> Result<f64, BackendError> {
        let mut masks = [0_usize; 3];
        for &(qubit, pauli) in factors {
            let bit = 1_usize << self.require(qubit)?;
            if pauli != Pauli::I && masks.iter().any(|mask| mask & bit != 0) {
                return Err(BackendError::DuplicateOperand);
            }
            match pauli {
                Pauli::I => {}
                Pauli::X => masks[0] |= bit,
                Pauli::Y => masks[1] |= bit,
                Pauli::Z => masks[2] |= bit,
            }
        }
        let flip = masks[0] | masks[1];
        let mut expectation = Complex64::new(0.0, 0.0);
        for (index, amplitude) in self.amplitudes.iter().enumerate() {
            let mut phase = if (index & masks[2]).count_ones() % 2 == 0 {
                Complex64::new(1.0, 0.0)
            } else {
                Complex64::new(-1.0, 0.0)
            };
            for bit_index in 0..self.qubits {
                let bit = 1_usize << bit_index;
                if masks[1] & bit != 0 {
                    phase *= if index & bit == 0 {
                        Complex64::new(0.0, 1.0)
                    } else {
                        Complex64::new(0.0, -1.0)
                    };
                }
            }
            expectation += amplitude.conj() * phase * self.amplitudes[index ^ flip];
        }
        if expectation.im.abs() > NUMERICAL_TOLERANCE {
            return Err(BackendError::Numerical(
                "Pauli expectation acquired an imaginary component".to_owned(),
            ));
        }
        Ok(expectation.re.clamp(-1.0, 1.0))
    }

    /// Returns the reduced 2x2 density matrix for one qubit.
    pub fn reduced_qubit_state(&self, qubit: QubitId) -> Result<ReducedQubitState, BackendError> {
        let bit = 1_usize << self.require(qubit)?;
        let mut matrix = [[Complex64::new(0.0, 0.0); 2]; 2];
        for index in 0..self.amplitudes.len() {
            if index & bit == 0 {
                let zero = self.amplitudes[index];
                let one = self.amplitudes[index | bit];
                matrix[0][0] += zero * zero.conj();
                matrix[0][1] += zero * one.conj();
                matrix[1][0] += one * zero.conj();
                matrix[1][1] += one * one.conj();
            }
        }
        Ok(ReducedQubitState {
            real: matrix.map(|row| row.map(|value| value.re)),
            imaginary: matrix.map(|row| row.map(|value| value.im)),
        })
    }

    fn require(&self, qubit: QubitId) -> Result<usize, BackendError> {
        let index = qubit.index();
        if index < self.qubits {
            Ok(index)
        } else {
            Err(BackendError::Unallocated(qubit))
        }
    }

    fn ensure_normalized(&self) -> Result<(), BackendError> {
        let norm = self.norm_sqr();
        if norm.is_finite() && (norm - 1.0).abs() <= NUMERICAL_TOLERANCE {
            Ok(())
        } else {
            Err(BackendError::Numerical(format!(
                "state norm {norm:.16} is outside tolerance"
            )))
        }
    }

    fn apply_matrix(
        &mut self,
        qubit: QubitId,
        matrix: [[Complex64; 2]; 2],
    ) -> Result<(), BackendError> {
        let bit = 1_usize << self.require(qubit)?;
        for base in (0..self.amplitudes.len()).step_by(bit * 2) {
            for offset in 0..bit {
                let zero = base + offset;
                let one = zero + bit;
                let a = self.amplitudes[zero];
                let b = self.amplitudes[one];
                self.amplitudes[zero] = matrix[0][0] * a + matrix[0][1] * b;
                self.amplitudes[one] = matrix[1][0] * a + matrix[1][1] * b;
            }
        }
        self.ensure_normalized()
    }
}

impl QuantumBackend for StateVectorEngine {
    fn allocate(&mut self, qubit: QubitId) -> Result<(), BackendError> {
        if qubit.index() != self.qubits {
            return Err(BackendError::NonContiguousAllocation {
                expected: self.qubits,
                found: qubit.index(),
            });
        }
        let new_len = self
            .amplitudes
            .len()
            .checked_mul(2)
            .ok_or(BackendError::AllocationOverflow)?;
        self.amplitudes.resize(new_len, Complex64::new(0.0, 0.0));
        self.qubits += 1;
        Ok(())
    }

    fn apply_single(&mut self, qubit: QubitId, gate: SingleQubitGate) -> Result<(), BackendError> {
        let zero = Complex64::new(0.0, 0.0);
        let one = Complex64::new(1.0, 0.0);
        let i = Complex64::new(0.0, 1.0);
        let matrix = match gate {
            SingleQubitGate::X => [[zero, one], [one, zero]],
            SingleQubitGate::Y => [[zero, -i], [i, zero]],
            SingleQubitGate::Z => [[one, zero], [zero, -one]],
            SingleQubitGate::H => {
                let s = Complex64::new(INVERSE_SQRT_2, 0.0);
                [[s, s], [s, -s]]
            }
            SingleQubitGate::S => [[one, zero], [zero, i]],
            SingleQubitGate::Sdg => [[one, zero], [zero, -i]],
            SingleQubitGate::T => {
                let phase = Complex64::from_polar(1.0, std::f64::consts::FRAC_PI_4);
                [[one, zero], [zero, phase]]
            }
            SingleQubitGate::Tdg => {
                let phase = Complex64::from_polar(1.0, -std::f64::consts::FRAC_PI_4);
                [[one, zero], [zero, phase]]
            }
            SingleQubitGate::Rx(angle) => {
                validate_angle(angle)?;
                let cosine = Complex64::new((angle / 2.0).cos(), 0.0);
                let sine = Complex64::new(0.0, -(angle / 2.0).sin());
                [[cosine, sine], [sine, cosine]]
            }
            SingleQubitGate::Ry(angle) => {
                validate_angle(angle)?;
                let cosine = Complex64::new((angle / 2.0).cos(), 0.0);
                let sine = Complex64::new((angle / 2.0).sin(), 0.0);
                [[cosine, -sine], [sine, cosine]]
            }
            SingleQubitGate::Rz(angle) => {
                validate_angle(angle)?;
                let negative = Complex64::from_polar(1.0, -angle / 2.0);
                let positive = Complex64::from_polar(1.0, angle / 2.0);
                [[negative, zero], [zero, positive]]
            }
        };
        self.apply_matrix(qubit, matrix)
    }

    fn controlled_x(&mut self, control: QubitId, target: QubitId) -> Result<(), BackendError> {
        let control_bit = 1_usize << self.require(control)?;
        let target_bit = 1_usize << self.require(target)?;
        if control_bit == target_bit {
            return Err(BackendError::DuplicateOperand);
        }
        for index in 0..self.amplitudes.len() {
            if index & control_bit != 0 && index & target_bit == 0 {
                self.amplitudes.swap(index, index | target_bit);
            }
        }
        self.ensure_normalized()
    }

    fn controlled_z(&mut self, control: QubitId, target: QubitId) -> Result<(), BackendError> {
        let control_bit = 1_usize << self.require(control)?;
        let target_bit = 1_usize << self.require(target)?;
        if control_bit == target_bit {
            return Err(BackendError::DuplicateOperand);
        }
        for (index, amplitude) in self.amplitudes.iter_mut().enumerate() {
            if index & control_bit != 0 && index & target_bit != 0 {
                *amplitude = -*amplitude;
            }
        }
        self.ensure_normalized()
    }

    fn controlled_phase(
        &mut self,
        control: QubitId,
        target: QubitId,
        angle: f64,
    ) -> Result<(), BackendError> {
        validate_angle(angle)?;
        let control_bit = 1_usize << self.require(control)?;
        let target_bit = 1_usize << self.require(target)?;
        if control_bit == target_bit {
            return Err(BackendError::DuplicateOperand);
        }
        let phase = Complex64::from_polar(1.0, angle);
        for (index, amplitude) in self.amplitudes.iter_mut().enumerate() {
            if index & control_bit != 0 && index & target_bit != 0 {
                *amplitude *= phase;
            }
        }
        self.ensure_normalized()
    }

    fn swap(&mut self, first: QubitId, second: QubitId) -> Result<(), BackendError> {
        let first_bit = 1_usize << self.require(first)?;
        let second_bit = 1_usize << self.require(second)?;
        if first_bit == second_bit {
            return Err(BackendError::DuplicateOperand);
        }
        for index in 0..self.amplitudes.len() {
            if index & first_bit == 0 && index & second_bit != 0 {
                self.amplitudes.swap(index, index ^ first_bit ^ second_bit);
            }
        }
        self.ensure_normalized()
    }

    fn controlled_controlled_x(
        &mut self,
        first_control: QubitId,
        second_control: QubitId,
        target: QubitId,
    ) -> Result<(), BackendError> {
        let first_bit = 1_usize << self.require(first_control)?;
        let second_bit = 1_usize << self.require(second_control)?;
        let target_bit = 1_usize << self.require(target)?;
        if first_bit == second_bit || first_bit == target_bit || second_bit == target_bit {
            return Err(BackendError::DuplicateOperand);
        }
        for index in 0..self.amplitudes.len() {
            if index & first_bit != 0 && index & second_bit != 0 && index & target_bit == 0 {
                self.amplitudes.swap(index, index | target_bit);
            }
        }
        self.ensure_normalized()
    }

    fn probability_one(&self, qubit: QubitId) -> Result<f64, BackendError> {
        let bit = 1_usize << self.require(qubit)?;
        let probability = self
            .amplitudes
            .iter()
            .enumerate()
            .filter(|(index, _)| index & bit != 0)
            .map(|(_, amplitude)| amplitude.norm_sqr())
            .sum::<f64>();
        if probability.is_finite()
            && (-NUMERICAL_TOLERANCE..=1.0 + NUMERICAL_TOLERANCE).contains(&probability)
        {
            Ok(probability.clamp(0.0, 1.0))
        } else {
            Err(BackendError::Numerical(format!(
                "measurement probability {probability:.16} is invalid"
            )))
        }
    }

    fn collapse(&mut self, qubit: QubitId, outcome: bool) -> Result<(), BackendError> {
        let probability_one = self.probability_one(qubit)?;
        let selected_probability = if outcome {
            probability_one
        } else {
            1.0 - probability_one
        };
        if selected_probability <= NUMERICAL_TOLERANCE {
            return Err(BackendError::Numerical(
                "cannot collapse to a zero-probability outcome".to_owned(),
            ));
        }
        let bit = 1_usize << self.require(qubit)?;
        let scale = selected_probability.sqrt().recip();
        for (index, amplitude) in self.amplitudes.iter_mut().enumerate() {
            if (index & bit != 0) == outcome {
                *amplitude *= scale;
            } else {
                *amplitude = Complex64::new(0.0, 0.0);
            }
        }
        self.ensure_normalized()
    }

    fn free(&mut self, qubit: QubitId) -> Result<(), BackendError> {
        let index = self.require(qubit)?;
        let expected = self.qubits.saturating_sub(1);
        if index != expected {
            return Err(BackendError::NonHighestFree {
                expected,
                found: index,
            });
        }
        if self.probability_one(qubit)? > NUMERICAL_TOLERANCE {
            return Err(BackendError::QubitNotZero(qubit));
        }
        self.amplitudes.truncate(self.amplitudes.len() / 2);
        self.qubits -= 1;
        self.ensure_normalized()
    }

    fn qubit_count(&self) -> usize {
        self.qubits
    }
}

fn validate_angle(angle: f64) -> Result<(), BackendError> {
    if angle.is_finite() {
        Ok(())
    } else {
        Err(BackendError::Numerical(
            "rotation angle must be finite".to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bell_engine() -> StateVectorEngine {
        let mut engine = StateVectorEngine::new();
        engine.allocate(QubitId::new(0)).expect("allocate q0");
        engine.allocate(QubitId::new(1)).expect("allocate q1");
        engine
            .apply_single(QubitId::new(0), SingleQubitGate::H)
            .expect("H q0");
        engine
            .controlled_x(QubitId::new(0), QubitId::new(1))
            .expect("CX q0 q1");
        engine
    }

    #[test]
    fn creates_exact_bell_support_in_little_endian_order() {
        let engine = bell_engine();
        assert!((engine.amplitudes()[0].re - INVERSE_SQRT_2).abs() < NUMERICAL_TOLERANCE);
        assert_eq!(engine.amplitudes()[1], Complex64::new(0.0, 0.0));
        assert_eq!(engine.amplitudes()[2], Complex64::new(0.0, 0.0));
        assert!((engine.amplitudes()[3].re - INVERSE_SQRT_2).abs() < NUMERICAL_TOLERANCE);
        assert!((engine.norm_sqr() - 1.0).abs() < NUMERICAL_TOLERANCE);
    }

    #[test]
    fn measurement_collapse_preserves_correlation() {
        let mut engine = bell_engine();
        engine
            .collapse(QubitId::new(0), true)
            .expect("measure q0=1");
        assert!((engine.probability_one(QubitId::new(1)).expect("p(q1)") - 1.0).abs() < 1e-12);
        engine
            .collapse(QubitId::new(1), true)
            .expect("measure q1=1");
        assert_eq!(engine.amplitudes()[3], Complex64::new(1.0, 0.0));
    }

    #[test]
    fn observations_match_bell_correlations_and_reduced_state() {
        let engine = bell_engine();
        assert_eq!(engine.basis_probabilities().len(), 4);
        assert!(
            (engine
                .marginal_probabilities(QubitId::new(0))
                .expect("marginal")[1]
                - 0.5)
                .abs()
                < NUMERICAL_TOLERANCE
        );
        assert!(
            (engine
                .pauli_expectation(&[(QubitId::new(0), Pauli::X), (QubitId::new(1), Pauli::X)])
                .expect("XX")
                - 1.0)
                .abs()
                < NUMERICAL_TOLERANCE
        );
        assert!(
            (engine
                .pauli_expectation(&[(QubitId::new(0), Pauli::Y), (QubitId::new(1), Pauli::Y)])
                .expect("YY")
                + 1.0)
                .abs()
                < NUMERICAL_TOLERANCE
        );
        assert!(
            (engine
                .pauli_expectation(&[(QubitId::new(0), Pauli::Z), (QubitId::new(1), Pauli::Z)])
                .expect("ZZ")
                - 1.0)
                .abs()
                < NUMERICAL_TOLERANCE
        );
        let reduced = engine
            .reduced_qubit_state(QubitId::new(0))
            .expect("reduced state");
        assert!((reduced.real[0][0] - 0.5).abs() < NUMERICAL_TOLERANCE);
        assert!((reduced.real[1][1] - 0.5).abs() < NUMERICAL_TOLERANCE);
        assert!(reduced.real[0][1].abs() < NUMERICAL_TOLERANCE);
    }

    #[test]
    fn controlled_phase_and_inverse_restore_state() {
        let mut engine = bell_engine();
        let original = engine.amplitudes().to_vec();
        engine
            .controlled_phase(QubitId::new(0), QubitId::new(1), 0.37)
            .expect("phase");
        engine
            .controlled_phase(QubitId::new(0), QubitId::new(1), -0.37)
            .expect("inverse");
        assert_state_close(engine.amplitudes(), &original);
    }

    #[allow(clippy::cast_precision_loss)] // Test widths are tiny and powers of two are exact in f64.
    fn qft(engine: &mut StateVectorEngine, qubits: &[QubitId]) {
        for (target_index, &target) in qubits.iter().enumerate() {
            engine
                .apply_single(target, SingleQubitGate::H)
                .expect("QFT H");
            for (control_index, &control) in qubits.iter().enumerate().skip(target_index + 1) {
                let denominator = (1_u64 << (control_index - target_index)) as f64;
                engine
                    .controlled_phase(control, target, std::f64::consts::PI / denominator)
                    .expect("QFT phase");
            }
        }
        for index in 0..qubits.len() / 2 {
            engine
                .swap(qubits[index], qubits[qubits.len() - 1 - index])
                .expect("QFT swap");
        }
    }

    #[allow(clippy::cast_precision_loss)] // Test widths are tiny and powers of two are exact in f64.
    fn inverse_qft(engine: &mut StateVectorEngine, qubits: &[QubitId]) {
        for index in 0..qubits.len() / 2 {
            engine
                .swap(qubits[index], qubits[qubits.len() - 1 - index])
                .expect("IQFT swap");
        }
        for target_index in (0..qubits.len()).rev() {
            let target = qubits[target_index];
            for control_index in (target_index + 1..qubits.len()).rev() {
                let denominator = (1_u64 << (control_index - target_index)) as f64;
                engine
                    .controlled_phase(
                        qubits[control_index],
                        target,
                        -std::f64::consts::PI / denominator,
                    )
                    .expect("IQFT phase");
            }
            engine
                .apply_single(target, SingleQubitGate::H)
                .expect("IQFT H");
        }
    }

    #[test]
    fn qft_inverse_roundtrip_restores_three_qubit_state() {
        let mut engine = StateVectorEngine::new();
        let qubits = [QubitId::new(0), QubitId::new(1), QubitId::new(2)];
        for qubit in qubits {
            engine.allocate(qubit).expect("allocate");
        }
        engine
            .apply_single(qubits[0], SingleQubitGate::H)
            .expect("H");
        engine
            .apply_single(qubits[1], SingleQubitGate::Ry(0.73))
            .expect("Ry");
        engine.controlled_x(qubits[1], qubits[2]).expect("CX");
        let original = engine.amplitudes().to_vec();
        qft(&mut engine, &qubits);
        inverse_qft(&mut engine, &qubits);
        assert_state_close(engine.amplitudes(), &original);
    }

    #[test]
    fn two_qubit_grover_amplifies_marked_state_exactly() {
        let mut engine = StateVectorEngine::new();
        let q0 = QubitId::new(0);
        let q1 = QubitId::new(1);
        engine.allocate(q0).expect("q0");
        engine.allocate(q1).expect("q1");
        engine.apply_single(q0, SingleQubitGate::H).expect("H");
        engine.apply_single(q1, SingleQubitGate::H).expect("H");
        engine.controlled_z(q0, q1).expect("oracle 11");
        for q in [q0, q1] {
            engine.apply_single(q, SingleQubitGate::H).expect("H");
            engine.apply_single(q, SingleQubitGate::X).expect("X");
        }
        engine.controlled_z(q0, q1).expect("diffusion phase");
        for q in [q0, q1] {
            engine.apply_single(q, SingleQubitGate::X).expect("X");
            engine.apply_single(q, SingleQubitGate::H).expect("H");
        }
        assert!((engine.basis_probabilities()[3] - 1.0).abs() < NUMERICAL_TOLERANCE);
    }

    fn next_random(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *state
    }

    fn reference_single(amplitudes: &mut [Complex64], qubit: usize, matrix: [[Complex64; 2]; 2]) {
        let bit = 1_usize << qubit;
        let before = amplitudes.to_vec();
        for (index, amplitude) in amplitudes.iter_mut().enumerate() {
            let source_zero = index & !bit;
            let row = usize::from(index & bit != 0);
            *amplitude =
                matrix[row][0] * before[source_zero] + matrix[row][1] * before[source_zero | bit];
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)] // Deterministic test PRNG maps u64 samples into f64 angles.
    fn deterministic_random_gate_sequences_preserve_norm_and_inverse_recover() {
        let mut seed = 0x434c_494f_5052_4f50;
        for width in 1..=6 {
            let mut engine = StateVectorEngine::new();
            for qubit in 0..width {
                engine.allocate(QubitId::new(qubit)).expect("allocate");
            }
            let initial = engine.amplitudes().to_vec();
            let mut operations = Vec::new();
            for _ in 0..128 {
                let index = u16::try_from(next_random(&mut seed) % u64::from(width))
                    .expect("bounded qubit index");
                let qubit = QubitId::new(index);
                let angle = (next_random(&mut seed) as f64 / u64::MAX as f64 - 0.5) * 2.0;
                engine
                    .apply_single(qubit, SingleQubitGate::Ry(angle))
                    .expect("random Ry");
                operations.push((qubit, angle));
                assert!((engine.norm_sqr() - 1.0).abs() < NUMERICAL_TOLERANCE);
            }
            for &(qubit, angle) in operations.iter().rev() {
                engine
                    .apply_single(qubit, SingleQubitGate::Ry(-angle))
                    .expect("inverse Ry");
            }
            assert_state_close(engine.amplitudes(), &initial);
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)] // Deterministic test PRNG maps u64 samples into f64 angles.
    fn independent_dense_reference_matches_random_single_qubit_sequences() {
        let mut seed = 0x5245_4645_5245_4e43;
        let mut engine = StateVectorEngine::new();
        for qubit in 0..4 {
            engine.allocate(QubitId::new(qubit)).expect("allocate");
        }
        let mut reference = engine.amplitudes().to_vec();
        for _ in 0..256 {
            let qubit = usize::try_from(next_random(&mut seed) % 4).expect("bounded index");
            let angle = (next_random(&mut seed) as f64 / u64::MAX as f64) * 2.0 - 1.0;
            let cosine = Complex64::new((angle / 2.0).cos(), 0.0);
            let sine = Complex64::new((angle / 2.0).sin(), 0.0);
            reference_single(&mut reference, qubit, [[cosine, -sine], [sine, cosine]]);
            engine
                .apply_single(
                    QubitId::new(u16::try_from(qubit).expect("bounded index")),
                    SingleQubitGate::Ry(angle),
                )
                .expect("engine Ry");
        }
        assert_state_close(engine.amplitudes(), &reference);
    }

    #[test]
    #[allow(clippy::cast_precision_loss)] // Deterministic test PRNG maps u64 samples into f64 angles.
    fn probability_and_reduced_state_invariants_hold_for_random_states() {
        let mut seed = 0x494e_5641_5249_414e;
        let mut engine = StateVectorEngine::new();
        for qubit in 0..5 {
            engine.allocate(QubitId::new(qubit)).expect("allocate");
        }
        for _ in 0..200 {
            let qubit = QubitId::new(
                u16::try_from(next_random(&mut seed) % 5).expect("bounded qubit index"),
            );
            let angle = next_random(&mut seed) as f64 / u64::MAX as f64;
            engine
                .apply_single(qubit, SingleQubitGate::Rx(angle))
                .expect("Rx");
        }
        assert!((engine.basis_probabilities().iter().sum::<f64>() - 1.0).abs() < 1.0e-11);
        for qubit in 0..5 {
            let marginal = engine
                .marginal_probabilities(QubitId::new(qubit))
                .expect("marginal");
            let reduced = engine
                .reduced_qubit_state(QubitId::new(qubit))
                .expect("reduced");
            assert!((marginal[0] + marginal[1] - 1.0).abs() < 1.0e-11);
            assert!((reduced.real[0][0] + reduced.real[1][1] - 1.0).abs() < 1.0e-11);
            assert!((reduced.real[0][1] - reduced.real[1][0]).abs() < 1.0e-11);
            assert!((reduced.imaginary[0][1] + reduced.imaginary[1][0]).abs() < 1.0e-11);
        }
    }

    #[test]
    fn rejects_non_contiguous_allocation() {
        let mut engine = StateVectorEngine::new();
        assert!(matches!(
            engine.allocate(QubitId::new(1)),
            Err(BackendError::NonContiguousAllocation { .. })
        ));
    }

    fn one_qubit() -> StateVectorEngine {
        let mut engine = StateVectorEngine::new();
        engine.allocate(QubitId::new(0)).expect("allocate q0");
        engine
    }

    fn assert_state_close(actual: &[Complex64], expected: &[Complex64]) {
        assert_eq!(actual.len(), expected.len());
        for (left, right) in actual.iter().zip(expected) {
            assert!((*left - *right).norm() < NUMERICAL_TOLERANCE);
        }
    }

    fn states_equal_up_to_global_phase(left: &[Complex64], right: &[Complex64]) -> bool {
        let pivot = left
            .iter()
            .zip(right)
            .find(|(a, b)| a.norm() > NUMERICAL_TOLERANCE && b.norm() > NUMERICAL_TOLERANCE);
        let Some((left_pivot, right_pivot)) = pivot else {
            return left.iter().all(|value| value.norm() <= NUMERICAL_TOLERANCE)
                && right
                    .iter()
                    .all(|value| value.norm() <= NUMERICAL_TOLERANCE);
        };
        let phase = *left_pivot / *right_pivot;
        left.iter()
            .zip(right)
            .all(|(a, b)| (*a - phase * *b).norm() < NUMERICAL_TOLERANCE)
    }

    #[test]
    fn pauli_gates_follow_the_declared_matrices() {
        let mut x = one_qubit();
        x.apply_single(QubitId::new(0), SingleQubitGate::X)
            .expect("X");
        assert_state_close(
            x.amplitudes(),
            &[Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
        );

        let mut y = one_qubit();
        y.apply_single(QubitId::new(0), SingleQubitGate::Y)
            .expect("Y");
        assert_state_close(
            y.amplitudes(),
            &[Complex64::new(0.0, 0.0), Complex64::new(0.0, 1.0)],
        );

        x.apply_single(QubitId::new(0), SingleQubitGate::Z)
            .expect("Z");
        assert_state_close(
            x.amplitudes(),
            &[Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0)],
        );
    }

    #[test]
    fn inverse_phase_and_rotation_pairs_restore_state() {
        let pairs = [
            (SingleQubitGate::S, SingleQubitGate::Sdg),
            (SingleQubitGate::T, SingleQubitGate::Tdg),
            (SingleQubitGate::Rx(0.731), SingleQubitGate::Rx(-0.731)),
            (SingleQubitGate::Ry(-1.219), SingleQubitGate::Ry(1.219)),
            (SingleQubitGate::Rz(2.041), SingleQubitGate::Rz(-2.041)),
        ];
        for (gate, inverse) in pairs {
            let mut engine = one_qubit();
            engine
                .apply_single(QubitId::new(0), SingleQubitGate::H)
                .expect("prepare plus");
            let expected = engine.amplitudes().to_vec();
            engine.apply_single(QubitId::new(0), gate).expect("gate");
            engine
                .apply_single(QubitId::new(0), inverse)
                .expect("inverse");
            assert_state_close(engine.amplitudes(), &expected);
        }
    }

    #[test]
    fn non_finite_rotation_is_rejected_without_mutation() {
        let mut engine = one_qubit();
        let before = engine.amplitudes().to_vec();
        assert!(matches!(
            engine.apply_single(QubitId::new(0), SingleQubitGate::Rx(f64::NAN)),
            Err(BackendError::Numerical(_))
        ));
        assert_eq!(engine.amplitudes(), before);
    }

    #[test]
    fn constructs_four_qubit_ghz_state() {
        let mut engine = StateVectorEngine::new();
        for index in 0..4 {
            engine
                .allocate(QubitId::new(index))
                .expect("contiguous allocation");
        }
        engine
            .apply_single(QubitId::new(0), SingleQubitGate::H)
            .expect("H");
        for target in 1..4 {
            engine
                .controlled_x(QubitId::new(0), QubitId::new(target))
                .expect("CX");
        }
        for (index, amplitude) in engine.amplitudes().iter().enumerate() {
            if matches!(index, 0 | 15) {
                assert!((amplitude.re - INVERSE_SQRT_2).abs() < NUMERICAL_TOLERANCE);
            } else {
                assert_eq!(*amplitude, Complex64::new(0.0, 0.0));
            }
        }
    }

    #[test]
    fn global_phase_equivalent_states_are_recognized() {
        let mut y_state = one_qubit();
        y_state
            .apply_single(QubitId::new(0), SingleQubitGate::Y)
            .expect("Y");
        let mut x_state = one_qubit();
        x_state
            .apply_single(QubitId::new(0), SingleQubitGate::X)
            .expect("X");
        assert!(states_equal_up_to_global_phase(
            y_state.amplitudes(),
            x_state.amplitudes()
        ));
    }

    #[test]
    fn rotation_boundaries_match_known_states() {
        let identity = one_qubit();
        for gate in [
            SingleQubitGate::Rx(0.0),
            SingleQubitGate::Ry(0.0),
            SingleQubitGate::Rz(0.0),
        ] {
            let mut engine = one_qubit();
            engine
                .apply_single(QubitId::new(0), gate)
                .expect("zero rotation");
            assert_state_close(engine.amplitudes(), identity.amplitudes());
        }

        let mut rx_pi = one_qubit();
        rx_pi
            .apply_single(QubitId::new(0), SingleQubitGate::Rx(std::f64::consts::PI))
            .expect("RX pi");
        assert_state_close(
            rx_pi.amplitudes(),
            &[Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0)],
        );
    }

    #[test]
    fn controlled_z_swap_and_toffoli_match_basis_behavior() {
        let mut cz = StateVectorEngine::new();
        for index in 0..2 {
            cz.allocate(QubitId::new(index)).expect("allocate");
            cz.apply_single(QubitId::new(index), SingleQubitGate::X)
                .expect("X");
        }
        cz.controlled_z(QubitId::new(0), QubitId::new(1))
            .expect("CZ");
        assert_eq!(cz.amplitudes()[3], Complex64::new(-1.0, 0.0));
        cz.swap(QubitId::new(0), QubitId::new(1)).expect("swap");
        assert_eq!(cz.amplitudes()[3], Complex64::new(-1.0, 0.0));

        let mut ccx = StateVectorEngine::new();
        for index in 0..3 {
            ccx.allocate(QubitId::new(index)).expect("allocate");
        }
        ccx.apply_single(QubitId::new(0), SingleQubitGate::X)
            .expect("X q0");
        ccx.apply_single(QubitId::new(1), SingleQubitGate::X)
            .expect("X q1");
        ccx.controlled_controlled_x(QubitId::new(0), QubitId::new(1), QubitId::new(2))
            .expect("CCX");
        assert_eq!(ccx.amplitudes()[7], Complex64::new(1.0, 0.0));
    }

    #[test]
    fn free_requires_highest_qubit_in_zero() {
        let mut engine = StateVectorEngine::new();
        engine.allocate(QubitId::new(0)).expect("q0");
        engine.allocate(QubitId::new(1)).expect("q1");
        assert!(matches!(
            engine.free(QubitId::new(0)),
            Err(BackendError::NonHighestFree { .. })
        ));
        engine
            .apply_single(QubitId::new(1), SingleQubitGate::X)
            .expect("X q1");
        assert!(matches!(
            engine.free(QubitId::new(1)),
            Err(BackendError::QubitNotZero(_))
        ));
        engine
            .apply_single(QubitId::new(1), SingleQubitGate::X)
            .expect("restore q1");
        engine.free(QubitId::new(1)).expect("safe free q1");
        assert_eq!(engine.qubit_count(), 1);
        assert_eq!(engine.amplitudes().len(), 2);
    }
}
