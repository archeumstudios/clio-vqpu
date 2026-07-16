//! Clio processor state machine for the executable Bell-state path.

#![forbid(unsafe_code)]

use clio_backend::{BackendError, QuantumBackend, SingleQubitGate};
use clio_core::{QubitId, REGISTER_COUNT};
use clio_engine::StateVectorEngine;
use clio_isa::{ClassicalBinaryOperation, ClassicalValue, Executable, Instruction};
use clio_resource::{ExecutionPlan, ResourceError, ResourceLimits, plan};
use clio_trace::{
    ClassicalChange, ExecutionTrace, MeasurementChange, StateAmplitude, TraceComparison,
    TraceEvent, TraceStatus,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
use thiserror::Error;

/// Stable RNG semantic identity from ADR-0003.
pub const RNG_IDENTITY: &str = "chacha8-splitmix64-shot-v1";
/// Reference engine semantic identity for result/replay compatibility.
pub const ENGINE_IDENTITY: &str = "clio-engine-hybrid-path-1";

/// Architectural execution status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProcessorStatus {
    /// Program loaded and ready.
    Ready,
    /// Instruction execution is active.
    Running,
    /// Paused by an external controller.
    Paused,
    /// Completed by fall-through.
    Completed,
    /// Stopped by `HALT`.
    Halted,
    /// Stopped by a runtime trap.
    Trapped,
    /// Resource admission failed.
    ResourceRejected,
    /// Semantic validation failed before runtime construction.
    ValidationFailed,
}

/// Three-state architectural measurement value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum MeasurementValue {
    /// Never written in this shot.
    #[default]
    Unset,
    /// Measured zero.
    Zero,
    /// Measured one.
    One,
}

/// Result of the latest explicit signed comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ComparisonValue {
    /// Left operand was smaller.
    Less,
    /// Operands were equal.
    Equal,
    /// Left operand was greater.
    Greater,
}

/// Serializable architectural state retained for the final shot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProcessorState {
    /// Next instruction index.
    pub program_counter: usize,
    /// Processor lifecycle status.
    pub status: ProcessorStatus,
    /// Signed classical registers.
    pub classical_registers: [i64; REGISTER_COUNT],
    /// Three-state measurement registers.
    pub measurement_registers: [MeasurementValue; REGISTER_COUNT],
    /// Allocated logical qubits.
    pub virtual_qubits: Vec<QubitId>,
    /// Latest comparison, unset before `CMP`.
    pub comparison: Option<ComparisonValue>,
    /// Executed instruction count in this shot.
    pub instruction_counter: u64,
    /// Current shot index.
    pub shot_index: u64,
    /// Total shots.
    pub total_shots: u64,
}

impl ProcessorState {
    fn new(shot_index: u64, total_shots: u64) -> Self {
        Self {
            program_counter: 0,
            status: ProcessorStatus::Ready,
            classical_registers: [0; REGISTER_COUNT],
            measurement_registers: [MeasurementValue::Unset; REGISTER_COUNT],
            virtual_qubits: Vec::new(),
            comparison: None,
            instruction_counter: 0,
            shot_index,
            total_shots,
        }
    }
}

/// Complete result from the built-in reference runtime.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Result schema identity.
    pub format: String,
    /// Source hash.
    pub source_sha256: String,
    /// Reference engine semantic identity.
    pub engine: String,
    /// RNG semantic identity.
    pub rng: String,
    /// User seed.
    pub seed: u64,
    /// Executed shots.
    pub shots: u64,
    /// Counts keyed by the written measurement registers, highest index first.
    pub measurement_counts: BTreeMap<String, u64>,
    /// Admitted execution plan.
    pub plan: ExecutionPlan,
    /// Final architectural state of the last shot.
    pub final_state: ProcessorState,
    /// Structured instruction trace.
    pub trace: ExecutionTrace,
}

/// Runtime failure after semantic validation.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Resource admission rejected the executable before engine allocation.
    #[error(transparent)]
    Resource(#[from] ResourceError),
    /// Engine/backend operation failed.
    #[error("runtime trap at shot {shot}, pc {program_counter}: {source}")]
    Backend {
        /// Shot index.
        shot: u64,
        /// Program counter.
        program_counter: usize,
        /// Backend cause.
        #[source]
        source: BackendError,
    },
    /// Program did not explicitly halt within its assembled stream.
    #[error("program completed without HALT at shot {shot}")]
    MissingHalt {
        /// Shot index.
        shot: u64,
    },
    /// An architectural precondition failed deterministically.
    #[error("runtime trap at shot {shot}, pc {program_counter}: {message}")]
    ArchitecturalTrap {
        /// Shot index.
        shot: u64,
        /// Program counter.
        program_counter: usize,
        /// Failure reason.
        message: String,
    },
    /// The global instruction budget was exhausted.
    #[error("instruction budget of {limit} was exhausted at shot {shot}, pc {program_counter}")]
    InstructionBudget {
        /// Configured limit.
        limit: u64,
        /// Shot index.
        shot: u64,
        /// Program counter.
        program_counter: usize,
    },
    /// The wall-clock execution budget was exhausted.
    #[error(
        "execution time budget of {limit_millis} ms was exhausted at shot {shot}, pc {program_counter}"
    )]
    TimeBudget {
        /// Configured milliseconds.
        limit_millis: u64,
        /// Shot index.
        shot: u64,
        /// Program counter.
        program_counter: usize,
    },
}

impl RuntimeError {
    /// Returns a stable machine-readable runtime error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Resource(_) => "R200",
            Self::Backend { .. } => "T201",
            Self::MissingHalt { .. } => "T202",
            Self::ArchitecturalTrap { .. } => "T203",
            Self::InstructionBudget { .. } => "T204",
            Self::TimeBudget { .. } => "T205",
        }
    }
}

/// Executes a validated program after resource admission.
pub fn run(
    executable: &Executable,
    limits: ResourceLimits,
) -> Result<ExecutionResult, RuntimeError> {
    let execution_plan = plan(executable, limits)?;
    let trace_capacity = if executable.metadata.trace_level == clio_core::TraceLevel::Off {
        0
    } else {
        usize::try_from(execution_plan.estimated_instructions).unwrap_or(0)
    };
    let mut trace = ExecutionTrace::new(executable.metadata.trace_level);
    trace.events.reserve(trace_capacity);
    let mut counts = BTreeMap::new();
    let mut logical_step = 0_u64;
    let measurement_width = measurement_width(executable);
    let mut final_state = ProcessorState::new(0, executable.metadata.shots);
    let started = Instant::now();
    let time_limit = Duration::from_millis(limits.max_execution_millis);

    for shot in 0..executable.metadata.shots {
        let mut state = ProcessorState::new(shot, executable.metadata.shots);
        let mut engine = StateVectorEngine::new();
        let mut rng = shot_rng(executable.metadata.seed, shot);

        state.status = ProcessorStatus::Running;
        while state.program_counter < executable.instructions.len() {
            if logical_step >= limits.max_instructions {
                state.status = ProcessorStatus::Trapped;
                return Err(RuntimeError::InstructionBudget {
                    limit: limits.max_instructions,
                    shot,
                    program_counter: state.program_counter,
                });
            }
            if started.elapsed() >= time_limit {
                state.status = ProcessorStatus::Trapped;
                return Err(RuntimeError::TimeBudget {
                    limit_millis: limits.max_execution_millis,
                    shot,
                    program_counter: state.program_counter,
                });
            }
            let pc = state.program_counter;
            let located = &executable.instructions[pc];
            let status_before = state.status;
            let mut allocated_qubit = None;
            let mut freed_qubit = None;
            let mut reset_outcome = None;
            let mut measurement = None;
            let mut classical = None;
            let mut branch_taken = None;
            state.program_counter += 1;
            state.instruction_counter += 1;
            logical_step += 1;

            let backend_result = match &located.instruction {
                Instruction::QAlloc(qubit) => {
                    allocated_qubit = Some(*qubit);
                    let result = engine.allocate(*qubit);
                    if result.is_ok() {
                        state.virtual_qubits.push(*qubit);
                    }
                    result
                }
                Instruction::QReset(qubit) => {
                    let probability_one =
                        engine
                            .probability_one(*qubit)
                            .map_err(|source| RuntimeError::Backend {
                                shot,
                                program_counter: pc,
                                source,
                            })?;
                    let outcome = if probability_one <= f64::EPSILON {
                        false
                    } else if (1.0 - probability_one) <= f64::EPSILON {
                        true
                    } else {
                        rng.random::<f64>() < probability_one
                    };
                    let collapse_result = engine.collapse(*qubit, outcome);
                    if collapse_result.is_ok() {
                        reset_outcome = Some(outcome);
                        if outcome {
                            engine.apply_single(*qubit, SingleQubitGate::X)
                        } else {
                            Ok(())
                        }
                    } else {
                        collapse_result
                    }
                }
                Instruction::QFree(qubit) => {
                    let result = engine.free(*qubit);
                    if result.is_ok() {
                        freed_qubit = Some(*qubit);
                        state.virtual_qubits.retain(|candidate| candidate != qubit);
                    }
                    result
                }
                Instruction::QH(qubit) => engine.apply_single(*qubit, SingleQubitGate::H),
                Instruction::QX(qubit) => engine.apply_single(*qubit, SingleQubitGate::X),
                Instruction::QY(qubit) => engine.apply_single(*qubit, SingleQubitGate::Y),
                Instruction::QZ(qubit) => engine.apply_single(*qubit, SingleQubitGate::Z),
                Instruction::QS(qubit) => engine.apply_single(*qubit, SingleQubitGate::S),
                Instruction::QSdg(qubit) => engine.apply_single(*qubit, SingleQubitGate::Sdg),
                Instruction::QT(qubit) => engine.apply_single(*qubit, SingleQubitGate::T),
                Instruction::QTdg(qubit) => engine.apply_single(*qubit, SingleQubitGate::Tdg),
                Instruction::QRx { qubit, angle } => {
                    engine.apply_single(*qubit, SingleQubitGate::Rx(*angle))
                }
                Instruction::QRy { qubit, angle } => {
                    engine.apply_single(*qubit, SingleQubitGate::Ry(*angle))
                }
                Instruction::QRz { qubit, angle } => {
                    engine.apply_single(*qubit, SingleQubitGate::Rz(*angle))
                }
                Instruction::QCx { control, target } => engine.controlled_x(*control, *target),
                Instruction::QCz { control, target } => engine.controlled_z(*control, *target),
                Instruction::QCphase {
                    control,
                    target,
                    angle,
                } => engine.controlled_phase(*control, *target, *angle),
                Instruction::QSwap { first, second } => engine.swap(*first, *second),
                Instruction::QCcx {
                    first_control,
                    second_control,
                    target,
                } => engine.controlled_controlled_x(*first_control, *second_control, *target),
                Instruction::QMeasure { qubit, destination } => {
                    let probability_one =
                        engine
                            .probability_one(*qubit)
                            .map_err(|source| RuntimeError::Backend {
                                shot,
                                program_counter: pc,
                                source,
                            })?;
                    let outcome = if probability_one <= f64::EPSILON {
                        false
                    } else if (1.0 - probability_one) <= f64::EPSILON {
                        true
                    } else {
                        rng.random::<f64>() < probability_one
                    };
                    let result = engine.collapse(*qubit, outcome);
                    if result.is_ok() {
                        state.measurement_registers[destination.index()] = if outcome {
                            MeasurementValue::One
                        } else {
                            MeasurementValue::Zero
                        };
                        measurement = Some(MeasurementChange {
                            register: *destination,
                            value: outcome,
                        });
                    }
                    result
                }
                Instruction::MovMeasurement {
                    destination,
                    source,
                } => {
                    let value = match state.measurement_registers[source.index()] {
                        MeasurementValue::Zero => 0,
                        MeasurementValue::One => 1,
                        MeasurementValue::Unset => {
                            state.status = ProcessorStatus::Trapped;
                            return Err(RuntimeError::ArchitecturalTrap {
                                shot,
                                program_counter: pc,
                                message: format!("cannot read unset measurement register {source}"),
                            });
                        }
                    };
                    state.classical_registers[destination.index()] = value;
                    classical = Some(ClassicalChange {
                        register: *destination,
                        value,
                    });
                    Ok(())
                }
                Instruction::MovRegister {
                    destination,
                    source,
                } => {
                    let value = state.classical_registers[source.index()];
                    state.classical_registers[destination.index()] = value;
                    classical = Some(ClassicalChange {
                        register: *destination,
                        value,
                    });
                    Ok(())
                }
                Instruction::LoadImmediate { destination, value } => {
                    state.classical_registers[destination.index()] = *value;
                    classical = Some(ClassicalChange {
                        register: *destination,
                        value: *value,
                    });
                    Ok(())
                }
                Instruction::ClassicalBinary {
                    operation,
                    destination,
                    left,
                    right,
                } => {
                    let left_value = state.classical_registers[left.index()];
                    let right_value = classical_value(&state, *right);
                    let value =
                        execute_binary(*operation, left_value, right_value).map_err(|message| {
                            RuntimeError::ArchitecturalTrap {
                                shot,
                                program_counter: pc,
                                message,
                            }
                        })?;
                    state.classical_registers[destination.index()] = value;
                    classical = Some(ClassicalChange {
                        register: *destination,
                        value,
                    });
                    Ok(())
                }
                Instruction::ClassicalNot {
                    destination,
                    source,
                } => {
                    let value = !state.classical_registers[source.index()];
                    state.classical_registers[destination.index()] = value;
                    classical = Some(ClassicalChange {
                        register: *destination,
                        value,
                    });
                    Ok(())
                }
                Instruction::Compare { left, right } => {
                    let right_value = classical_value(&state, *right);
                    state.comparison = Some(
                        match state.classical_registers[left.index()].cmp(&right_value) {
                            std::cmp::Ordering::Less => ComparisonValue::Less,
                            std::cmp::Ordering::Equal => ComparisonValue::Equal,
                            std::cmp::Ordering::Greater => ComparisonValue::Greater,
                        },
                    );
                    Ok(())
                }
                Instruction::Jump { target } => {
                    state.program_counter = *target;
                    branch_taken = Some(true);
                    Ok(())
                }
                Instruction::JumpZero { target } => {
                    let Some(comparison) = state.comparison else {
                        state.status = ProcessorStatus::Trapped;
                        return Err(RuntimeError::ArchitecturalTrap {
                            shot,
                            program_counter: pc,
                            message: "JZ requires an earlier CMP in the executed path".to_owned(),
                        });
                    };
                    let taken = comparison == ComparisonValue::Equal;
                    if taken {
                        state.program_counter = *target;
                    }
                    branch_taken = Some(taken);
                    Ok(())
                }
                Instruction::JumpNotZero { target }
                | Instruction::JumpLess { target }
                | Instruction::JumpGreater { target } => {
                    let Some(comparison) = state.comparison else {
                        state.status = ProcessorStatus::Trapped;
                        return Err(RuntimeError::ArchitecturalTrap {
                            shot,
                            program_counter: pc,
                            message: format!(
                                "{} requires an earlier CMP",
                                located.instruction.mnemonic()
                            ),
                        });
                    };
                    let taken = match located.instruction {
                        Instruction::JumpNotZero { .. } => comparison != ComparisonValue::Equal,
                        Instruction::JumpLess { .. } => comparison == ComparisonValue::Less,
                        Instruction::JumpGreater { .. } => comparison == ComparisonValue::Greater,
                        _ => unreachable!("matched conditional branch"),
                    };
                    if taken {
                        state.program_counter = *target;
                    }
                    branch_taken = Some(taken);
                    Ok(())
                }
                Instruction::Halt => {
                    state.status = ProcessorStatus::Halted;
                    Ok(())
                }
            };

            if let Err(source) = backend_result {
                state.status = ProcessorStatus::Trapped;
                return Err(RuntimeError::Backend {
                    shot,
                    program_counter: pc,
                    source,
                });
            }

            trace.push(TraceEvent {
                shot,
                step: logical_step,
                program_counter: pc,
                mnemonic: located.instruction.mnemonic().to_owned(),
                source_span: located.span,
                status_before: trace_status(status_before),
                status_after: trace_status(state.status),
                allocated_qubit,
                freed_qubit,
                reset_outcome,
                measurement,
                classical,
                branch_taken,
                comparison: state.comparison.map(trace_comparison),
                state_snapshot: state_snapshot(&engine, executable.metadata.trace_level),
            });

            if state.status == ProcessorStatus::Halted {
                break;
            }
        }

        if state.status != ProcessorStatus::Halted {
            state.status = ProcessorStatus::Completed;
            return Err(RuntimeError::MissingHalt { shot });
        }
        let key = measurement_key(&state.measurement_registers, measurement_width);
        *counts.entry(key).or_insert(0) += 1;
        final_state = state;
    }

    Ok(ExecutionResult {
        format: "clio-result-hybrid-path-1".to_owned(),
        source_sha256: executable.source_sha256.clone(),
        engine: ENGINE_IDENTITY.to_owned(),
        rng: RNG_IDENTITY.to_owned(),
        seed: executable.metadata.seed,
        shots: executable.metadata.shots,
        measurement_counts: counts,
        plan: execution_plan,
        final_state,
        trace,
    })
}

const fn trace_comparison(value: ComparisonValue) -> TraceComparison {
    match value {
        ComparisonValue::Less => TraceComparison::Less,
        ComparisonValue::Equal => TraceComparison::Equal,
        ComparisonValue::Greater => TraceComparison::Greater,
    }
}

fn classical_value(state: &ProcessorState, value: ClassicalValue) -> i64 {
    match value {
        ClassicalValue::Register(register) => state.classical_registers[register.index()],
        ClassicalValue::Immediate(immediate) => immediate,
    }
}

fn execute_binary(
    operation: ClassicalBinaryOperation,
    left: i64,
    right: i64,
) -> Result<i64, String> {
    match operation {
        ClassicalBinaryOperation::Add => left
            .checked_add(right)
            .ok_or_else(|| "signed addition overflow".to_owned()),
        ClassicalBinaryOperation::Subtract => left
            .checked_sub(right)
            .ok_or_else(|| "signed subtraction overflow".to_owned()),
        ClassicalBinaryOperation::Multiply => left
            .checked_mul(right)
            .ok_or_else(|| "signed multiplication overflow".to_owned()),
        ClassicalBinaryOperation::Divide => {
            if right == 0 {
                Err("division by zero".to_owned())
            } else {
                left.checked_div(right)
                    .ok_or_else(|| "signed division overflow".to_owned())
            }
        }
        ClassicalBinaryOperation::Modulo => {
            if right == 0 {
                Err("modulo by zero".to_owned())
            } else {
                left.checked_rem(right)
                    .ok_or_else(|| "signed modulo overflow".to_owned())
            }
        }
        ClassicalBinaryOperation::And => Ok(left & right),
        ClassicalBinaryOperation::Or => Ok(left | right),
        ClassicalBinaryOperation::Xor => Ok(left ^ right),
        ClassicalBinaryOperation::ShiftLeft => checked_shift(right)
            .and_then(|shift| left.checked_shl(shift))
            .ok_or_else(|| "invalid or overflowing left shift".to_owned()),
        ClassicalBinaryOperation::ShiftRight => checked_shift(right)
            .and_then(|shift| left.checked_shr(shift))
            .ok_or_else(|| "invalid right shift".to_owned()),
    }
}

fn checked_shift(value: i64) -> Option<u32> {
    u32::try_from(value).ok().filter(|shift| *shift < 64)
}

fn state_snapshot(
    engine: &StateVectorEngine,
    level: clio_core::TraceLevel,
) -> Option<Vec<StateAmplitude>> {
    if engine.qubit_count() > 8
        || !matches!(
            level,
            clio_core::TraceLevel::StateSmall | clio_core::TraceLevel::FullDebug
        )
    {
        return None;
    }
    Some(
        engine
            .amplitudes()
            .iter()
            .enumerate()
            .map(|(basis_index, amplitude)| StateAmplitude {
                basis_index,
                real: amplitude.re,
                imaginary: amplitude.im,
                probability: amplitude.norm_sqr(),
            })
            .collect(),
    )
}

fn trace_status(status: ProcessorStatus) -> TraceStatus {
    match status {
        ProcessorStatus::Ready => TraceStatus::Ready,
        ProcessorStatus::Running | ProcessorStatus::Paused => TraceStatus::Running,
        ProcessorStatus::Completed => TraceStatus::Completed,
        ProcessorStatus::Halted => TraceStatus::Halted,
        ProcessorStatus::Trapped
        | ProcessorStatus::ResourceRejected
        | ProcessorStatus::ValidationFailed => TraceStatus::Trapped,
    }
}

fn measurement_width(executable: &Executable) -> usize {
    executable
        .instructions
        .iter()
        .filter_map(|located| match located.instruction {
            Instruction::QMeasure { destination, .. } => Some(destination.index() + 1),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

fn measurement_key(
    registers: &[MeasurementValue; REGISTER_COUNT],
    measurement_width: usize,
) -> String {
    registers[..measurement_width]
        .iter()
        .rev()
        .copied()
        .map(|value| match value {
            MeasurementValue::Zero => '0',
            MeasurementValue::One => '1',
            MeasurementValue::Unset => 'x',
        })
        .collect()
}

fn shot_rng(seed: u64, shot: u64) -> ChaCha8Rng {
    let mut state = splitmix64(seed ^ shot);
    let mut bytes = [0_u8; 32];
    for chunk in bytes.chunks_exact_mut(8) {
        state = splitmix64(state);
        chunk.copy_from_slice(&state.to_le_bytes());
    }
    ChaCha8Rng::from_seed(bytes)
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut result = value;
    result = (result ^ (result >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    result = (result ^ (result >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    result ^ (result >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clio_assembler::assemble;

    const BELL: &str = ".seed 42\n.shots 1024\n.trace instructions\nQALLOC q0\nQALLOC q1\nQH q0\nQCX q0, q1\nQMEASURE q0, m0\nQMEASURE q1, m1\nHALT\n";

    #[test]
    fn bell_run_has_only_correlated_outcomes() {
        let executable = assemble(BELL).expect("assemble Bell");
        let result = run(&executable, ResourceLimits::default()).expect("run Bell");
        assert_eq!(result.measurement_counts.values().sum::<u64>(), 1024);
        assert!(
            result
                .measurement_counts
                .keys()
                .all(|key| key == "00" || key == "11")
        );
        assert!(result.measurement_counts.contains_key("00"));
        assert!(result.measurement_counts.contains_key("11"));
    }

    #[test]
    fn same_seed_reproduces_counts_and_trace() {
        let executable = assemble(BELL).expect("assemble Bell");
        let first = run(&executable, ResourceLimits::default()).expect("first run");
        let second = run(&executable, ResourceLimits::default()).expect("second run");
        assert_eq!(first.measurement_counts, second.measurement_counts);
        assert_eq!(first.trace, second.trace);
    }

    #[test]
    fn final_measurements_are_not_unset() {
        let executable = assemble(BELL).expect("assemble Bell");
        let result = run(&executable, ResourceLimits::default()).expect("run Bell");
        assert_ne!(
            result.final_state.measurement_registers[0],
            MeasurementValue::Unset
        );
        assert_ne!(
            result.final_state.measurement_registers[1],
            MeasurementValue::Unset
        );
    }

    #[test]
    fn measurement_drives_classical_control_flow() {
        let source = ".seed 9\n.shots 128\n.trace instructions\nQALLOC q0\nQH q0\nQMEASURE q0, m0\nMOV r0, m0\nCMP r0, 0\nJZ measured_zero\nLOADI r1, 1\nJMP done\nmeasured_zero: LOADI r1, 0\ndone: HALT\n";
        let executable = assemble(source).expect("assemble hybrid program");
        let result = run(&executable, ResourceLimits::default()).expect("run hybrid program");
        assert!(result.measurement_counts.contains_key("0"));
        assert!(result.measurement_counts.contains_key("1"));
        let final_measurement = result.final_state.measurement_registers[0];
        let expected = i64::from(final_measurement == MeasurementValue::One);
        assert_eq!(result.final_state.classical_registers[1], expected);
        assert!(
            result
                .trace
                .events
                .iter()
                .any(|event| event.branch_taken == Some(true))
        );
        assert!(
            result
                .trace
                .events
                .iter()
                .any(|event| event.branch_taken == Some(false))
        );
    }

    #[test]
    fn moving_an_unset_measurement_traps() {
        let executable = assemble("MOV r0, m0\nHALT\n").expect("assemble");
        assert!(matches!(
            run(&executable, ResourceLimits::default()),
            Err(RuntimeError::ArchitecturalTrap { .. })
        ));
    }

    #[test]
    fn rotation_program_executes_through_runtime() {
        let source = "QALLOC q0\nQRX q0, 0.25\nQRY q0, -0.75\nQRZ q0, 1.5\nQSDG q0\nQTDG q0\nQMEASURE q0, m0\nHALT\n";
        let executable = assemble(source).expect("assemble rotations");
        let result = run(&executable, ResourceLimits::default()).expect("run rotations");
        assert_eq!(result.measurement_counts.values().sum::<u64>(), 1);
        assert_eq!(result.final_state.status, ProcessorStatus::Halted);
    }

    #[test]
    fn reset_then_free_updates_real_engine_and_processor_state() {
        let source =
            ".seed 4\n.shots 8\n.trace instructions\nQALLOC q0\nQX q0\nQRESET q0\nQFREE q0\nHALT\n";
        let executable = assemble(source).expect("assemble lifecycle");
        let result = run(&executable, ResourceLimits::default()).expect("run lifecycle");
        assert!(result.final_state.virtual_qubits.is_empty());
        assert_eq!(
            result
                .trace
                .events
                .iter()
                .filter(|event| event.reset_outcome == Some(true))
                .count(),
            8
        );
        assert_eq!(
            result
                .trace
                .events
                .iter()
                .filter(|event| event.freed_qubit == Some(QubitId::new(0)))
                .count(),
            8
        );
    }

    #[test]
    fn multi_qubit_operations_execute_through_runtime() {
        let source = "QALLOC q0\nQALLOC q1\nQALLOC q2\nQX q0\nQX q1\nQCCX q0, q1, q2\nQCZ q1, q2\nQSWAP q0, q2\nQMEASURE q0, m0\nQMEASURE q1, m1\nQMEASURE q2, m2\nHALT\n";
        let executable = assemble(source).expect("assemble multi-qubit program");
        let result = run(&executable, ResourceLimits::default()).expect("run multi-qubit program");
        assert_eq!(result.measurement_counts.get("111"), Some(&1));
    }

    #[test]
    fn classical_arithmetic_logic_and_register_comparison_execute() {
        let source = ".trace off\nLOADI r0, 9\nMOV r1, r0\nADD r2, r0, 3\nSUB r3, r2, r1\nMUL r4, r3, 7\nDIV r5, r4, 3\nMOD r6, r5, 5\nAND r7, r0, 3\nOR r8, r7, 8\nXOR r9, r8, r0\nNOT r10, r9\nSHL r11, r7, 3\nSHR r12, r11, 2\nCMP r0, r1\nJNZ fail\nJMP done\nfail: LOADI r15, -1\ndone: HALT\n";
        let executable = assemble(source).expect("assemble classical program");
        let result = run(&executable, ResourceLimits::default()).expect("run classical program");
        let registers = result.final_state.classical_registers;
        assert_eq!(registers[2], 12);
        assert_eq!(registers[3], 3);
        assert_eq!(registers[4], 21);
        assert_eq!(registers[5], 7);
        assert_eq!(registers[6], 2);
        assert_eq!(registers[12], 2);
        assert_eq!(registers[15], 0);
    }

    #[test]
    fn checked_arithmetic_traps_on_invalid_operations() {
        for source in [
            ".trace off\nLOADI r0, 9223372036854775807\nADD r1, r0, 1\nHALT\n",
            ".trace off\nLOADI r0, -9223372036854775808\nDIV r1, r0, -1\nHALT\n",
            ".trace off\nLOADI r0, 7\nDIV r1, r0, 0\nHALT\n",
            ".trace off\nLOADI r0, 7\nMOD r1, r0, 0\nHALT\n",
            ".trace off\nLOADI r0, 1\nSHL r1, r0, 64\nHALT\n",
            ".trace off\nLOADI r0, 1\nSHR r1, r0, -1\nHALT\n",
        ] {
            let executable = assemble(source).expect("assemble trap case");
            assert!(matches!(
                run(&executable, ResourceLimits::default()),
                Err(RuntimeError::ArchitecturalTrap { .. })
            ));
        }
    }

    #[test]
    fn backward_loop_runs_under_instruction_budget() {
        let source =
            ".trace instructions\nLOADI r0, 0\nloop: ADD r0, r0, 1\nCMP r0, 5\nJLT loop\nHALT\n";
        let executable = assemble(source).expect("assemble loop");
        assert!(executable.has_backward_branches);
        let limits = ResourceLimits {
            max_instructions: 100,
            ..ResourceLimits::default()
        };
        let result = run(&executable, limits).expect("bounded loop completes");
        assert_eq!(result.final_state.classical_registers[0], 5);
        assert!(result.trace.events.iter().any(|event| {
            event.mnemonic == "JLT"
                && event.branch_taken == Some(true)
                && event.comparison == Some(TraceComparison::Less)
        }));
        assert!(result.trace.events.iter().any(|event| {
            event.mnemonic == "JLT"
                && event.branch_taken == Some(false)
                && event.comparison == Some(TraceComparison::Equal)
        }));
    }

    #[test]
    fn jgt_jnz_and_jz_follow_explicit_comparisons() {
        let source = ".trace off\nLOADI r0, 2\nCMP r0, 1\nJGT greater\nLOADI r15, -1\ngreater: CMP r0, 0\nJNZ nonzero\nLOADI r15, -2\nnonzero: CMP r0, 2\nJZ equal\nLOADI r15, -3\nequal: LOADI r14, 1\nHALT\n";
        let result = run(
            &assemble(source).expect("assemble branches"),
            ResourceLimits::default(),
        )
        .expect("run branches");
        assert_eq!(result.final_state.classical_registers[14], 1);
        assert_eq!(result.final_state.classical_registers[15], 0);
    }

    #[test]
    fn conditional_branch_without_cmp_traps() {
        let executable = assemble(".trace off\nJGT done\ndone: HALT\n").expect("assemble");
        let error = run(&executable, ResourceLimits::default()).expect_err("missing CMP traps");
        assert_eq!(error.code(), "T203");
    }

    #[test]
    fn infinite_loop_hits_instruction_budget() {
        let executable = assemble(".trace off\nloop: JMP loop\nHALT\n").expect("assemble loop");
        let limits = ResourceLimits {
            max_instructions: 25,
            ..ResourceLimits::default()
        };
        assert!(matches!(
            run(&executable, limits),
            Err(RuntimeError::InstructionBudget { limit: 25, .. })
        ));
        let error = run(&executable, limits).expect_err("loop must trap again");
        assert_eq!(error.code(), "T204");
    }

    #[test]
    fn zero_time_budget_traps_before_execution() {
        let executable = assemble(".trace off\nHALT\n").expect("assemble");
        let limits = ResourceLimits {
            max_execution_millis: 0,
            ..ResourceLimits::default()
        };
        assert!(matches!(
            run(&executable, limits),
            Err(RuntimeError::TimeBudget { .. })
        ));
    }

    #[test]
    fn admitted_state_trace_bound_covers_serialized_trace() {
        let source = ".shots 32\n.trace state-small\nQALLOC q0\nQALLOC q1\nQH q0\nQCX q0, q1\nQMEASURE q0, m0\nQMEASURE q1, m1\nHALT\n";
        let executable = assemble(source).expect("assemble");
        let result = run(&executable, ResourceLimits::default()).expect("run");
        let serialized = u64::try_from(serde_json::to_vec(&result.trace).expect("serialize").len())
            .expect("trace size");
        assert!(result.plan.estimated_trace_bytes >= serialized);
    }
}
