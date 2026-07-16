//! Conservative resource estimation and execution admission.

#![forbid(unsafe_code)]

use clio_isa::Executable;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const COMPLEX_BYTES: u64 = 16;
const FIXED_RUNTIME_OVERHEAD: u64 = 64 * 1024;
// Covers the largest currently serialized bounded state-small event observed by the
// frozen evidence protocol, with headroom for JSON field and numeric variation.
const TRACE_EVENT_ESTIMATE: u64 = 640;

/// Runtime safety limits supplied by the host application.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum active virtual qubits.
    pub max_qubits: u16,
    /// Maximum estimated total memory.
    pub max_total_memory_bytes: u64,
    /// Maximum shots.
    pub max_shots: u64,
    /// Maximum total executed instructions across shots.
    pub max_instructions: u64,
    /// Maximum estimated trace bytes.
    pub max_trace_bytes: u64,
    /// Maximum wall-clock execution time in milliseconds.
    pub max_execution_millis: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_qubits: 25,
            max_total_memory_bytes: 512 * 1024 * 1024,
            max_shots: 1_000_000,
            max_instructions: 10_000_000,
            max_trace_bytes: 256 * 1024 * 1024,
            max_execution_millis: 30_000,
        }
    }
}

/// A checked pre-allocation execution plan.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Required active qubits.
    pub qubits: u16,
    /// Number of complex amplitudes.
    pub amplitude_count: u64,
    /// Exact raw double-complex state bytes.
    pub raw_state_bytes: u64,
    /// Conservative state plus runtime overhead.
    pub estimated_runtime_bytes: u64,
    /// Conservative trace bytes.
    pub estimated_trace_bytes: u64,
    /// Conservative total bytes.
    pub estimated_total_bytes: u64,
    /// Maximum instruction executions for the straight-line program.
    pub estimated_instructions: u64,
    /// Shots.
    pub shots: u64,
}

/// Deterministic resource admission failure.
#[derive(Clone, Debug, Eq, Error, PartialEq, Serialize, Deserialize)]
pub enum ResourceError {
    /// Checked arithmetic could not represent a requested scale.
    #[error("resource calculation overflowed")]
    ArithmeticOverflow,
    /// The qubit count exceeds the configured limit.
    #[error("required qubits {required} exceed limit {limit}")]
    QubitLimit {
        /// Required qubits.
        required: u16,
        /// Configured maximum.
        limit: u16,
    },
    /// The shot count exceeds the configured limit.
    #[error("requested shots {requested} exceed limit {limit}")]
    ShotLimit {
        /// Requested shots.
        requested: u64,
        /// Configured maximum.
        limit: u64,
    },
    /// The instruction execution bound exceeds the configured limit.
    #[error("estimated instructions {estimated} exceed limit {limit}")]
    InstructionLimit {
        /// Estimated executions.
        estimated: u64,
        /// Configured maximum.
        limit: u64,
    },
    /// The trace estimate exceeds the configured limit.
    #[error("estimated trace bytes {estimated} exceed limit {limit}")]
    TraceLimit {
        /// Estimated trace bytes.
        estimated: u64,
        /// Configured maximum.
        limit: u64,
    },
    /// The total memory estimate exceeds the effective budget.
    #[error("estimated memory {estimated} bytes exceeds budget {budget} bytes")]
    MemoryLimit {
        /// Estimated total bytes.
        estimated: u64,
        /// Effective budget.
        budget: u64,
    },
}

/// Builds and admits a plan before any state-vector allocation.
pub fn plan(
    executable: &Executable,
    limits: ResourceLimits,
) -> Result<ExecutionPlan, ResourceError> {
    if executable.required_qubits > limits.max_qubits {
        return Err(ResourceError::QubitLimit {
            required: executable.required_qubits,
            limit: limits.max_qubits,
        });
    }
    if executable.metadata.shots > limits.max_shots {
        return Err(ResourceError::ShotLimit {
            requested: executable.metadata.shots,
            limit: limits.max_shots,
        });
    }

    let amplitude_count = 1_u64
        .checked_shl(u32::from(executable.required_qubits))
        .ok_or(ResourceError::ArithmeticOverflow)?;
    let raw_state_bytes = amplitude_count
        .checked_mul(COMPLEX_BYTES)
        .ok_or(ResourceError::ArithmeticOverflow)?;
    let engine_overhead = raw_state_bytes / 8;
    let estimated_runtime_bytes = raw_state_bytes
        .checked_add(engine_overhead)
        .and_then(|bytes| bytes.checked_add(FIXED_RUNTIME_OVERHEAD))
        .ok_or(ResourceError::ArithmeticOverflow)?;
    let instruction_count = u64::try_from(executable.instructions.len())
        .map_err(|_| ResourceError::ArithmeticOverflow)?;
    let estimated_instructions = if executable.has_backward_branches {
        limits.max_instructions
    } else {
        instruction_count
            .checked_mul(executable.metadata.shots)
            .ok_or(ResourceError::ArithmeticOverflow)?
    };
    if estimated_instructions > limits.max_instructions {
        return Err(ResourceError::InstructionLimit {
            estimated: estimated_instructions,
            limit: limits.max_instructions,
        });
    }
    let estimated_trace_bytes =
        if matches!(executable.metadata.trace_level, clio_core::TraceLevel::Off) {
            0
        } else {
            estimated_instructions
                .checked_mul(TRACE_EVENT_ESTIMATE)
                .ok_or(ResourceError::ArithmeticOverflow)?
        };
    if estimated_trace_bytes > limits.max_trace_bytes {
        return Err(ResourceError::TraceLimit {
            estimated: estimated_trace_bytes,
            limit: limits.max_trace_bytes,
        });
    }
    let estimated_total_bytes = estimated_runtime_bytes
        .checked_add(estimated_trace_bytes)
        .ok_or(ResourceError::ArithmeticOverflow)?;
    let effective_budget = executable
        .metadata
        .memory_budget_bytes
        .map_or(limits.max_total_memory_bytes, |declared| {
            declared.min(limits.max_total_memory_bytes)
        });
    if estimated_total_bytes > effective_budget {
        return Err(ResourceError::MemoryLimit {
            estimated: estimated_total_bytes,
            budget: effective_budget,
        });
    }
    Ok(ExecutionPlan {
        qubits: executable.required_qubits,
        amplitude_count,
        raw_state_bytes,
        estimated_runtime_bytes,
        estimated_trace_bytes,
        estimated_total_bytes,
        estimated_instructions,
        shots: executable.metadata.shots,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clio_assembler::assemble;

    #[test]
    fn exact_raw_state_formula_is_used() {
        let executable =
            assemble(".shots 1\n.trace off\nQALLOC q0\nQALLOC q1\nHALT\n").expect("valid program");
        let plan = plan(&executable, ResourceLimits::default()).expect("admitted");
        assert_eq!(plan.amplitude_count, 4);
        assert_eq!(plan.raw_state_bytes, 64);
    }

    #[test]
    fn rejects_before_exceeding_declared_memory() {
        let executable = assemble(".shots 1\n.trace off\n.budget memory=65536B\nQALLOC q0\nHALT\n")
            .expect("valid program");
        assert!(matches!(
            plan(&executable, ResourceLimits::default()),
            Err(ResourceError::MemoryLimit { .. })
        ));
    }
}
