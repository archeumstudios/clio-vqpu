//! Shared bounded architecture types and diagnostics for Clio VQPU.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;

/// Maximum number of classical or measurement registers.
pub const REGISTER_COUNT: usize = 16;

/// A byte-oriented source span with one-based display coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceSpan {
    /// Inclusive byte offset.
    pub start: usize,
    /// Exclusive byte offset.
    pub end: usize,
    /// One-based line.
    pub line: usize,
    /// One-based column.
    pub column: usize,
}

impl SourceSpan {
    /// Creates a source span.
    #[must_use]
    pub const fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

/// A structured, actionable source diagnostic.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable diagnostic code.
    pub code: String,
    /// Concise failure message.
    pub message: String,
    /// Optional source span.
    pub span: Option<SourceSpan>,
    /// Optional correction guidance.
    pub help: Option<String>,
}

impl Diagnostic {
    /// Constructs an error diagnostic.
    #[must_use]
    pub fn error(code: impl Into<String>, message: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            span: Some(span),
            help: None,
        }
    }

    /// Adds correction guidance.
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// A classical register index in `r0..r15`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ClassicalRegister(u8);

/// A measurement register index in `m0..m15`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MeasurementRegister(u8);

/// A typed logical virtual-qubit identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct QubitId(u16);

/// Error returned for an invalid bounded identifier.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum IdentifierError {
    /// The register index is outside the architectural register file.
    #[error("register index {0} is outside 0..15")]
    RegisterOutOfRange(u8),
    /// The source token has the wrong prefix or numeric shape.
    #[error("invalid {kind} identifier `{value}`")]
    InvalidSyntax {
        /// Expected identifier kind.
        kind: &'static str,
        /// Rejected value.
        value: String,
    },
    /// The qubit identifier exceeds its encoded range.
    #[error("qubit identifier is outside 0..65535")]
    QubitOutOfRange,
}

macro_rules! register_type {
    ($name:ident, $prefix:literal, $kind:literal) => {
        impl $name {
            /// Creates a checked register index.
            ///
            /// # Errors
            ///
            /// Returns [`IdentifierError::RegisterOutOfRange`] for indices above 15.
            pub fn new(index: u8) -> Result<Self, IdentifierError> {
                if usize::from(index) < REGISTER_COUNT {
                    Ok(Self(index))
                } else {
                    Err(IdentifierError::RegisterOutOfRange(index))
                }
            }

            /// Returns the numeric index.
            #[must_use]
            pub const fn index(self) -> usize {
                self.0 as usize
            }
        }

        impl FromStr for $name {
            type Err = IdentifierError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let Some(index) = value
                    .strip_prefix($prefix)
                    .or_else(|| value.strip_prefix($prefix.to_ascii_uppercase().as_str()))
                else {
                    return Err(IdentifierError::InvalidSyntax {
                        kind: $kind,
                        value: value.to_owned(),
                    });
                };
                let index = index
                    .parse::<u8>()
                    .map_err(|_| IdentifierError::InvalidSyntax {
                        kind: $kind,
                        value: value.to_owned(),
                    })?;
                Self::new(index)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}{}", $prefix, self.0)
            }
        }
    };
}

register_type!(ClassicalRegister, "r", "classical register");
register_type!(MeasurementRegister, "m", "measurement register");

impl QubitId {
    /// Creates a logical qubit identifier.
    #[must_use]
    pub const fn new(index: u16) -> Self {
        Self(index)
    }

    /// Returns its numeric identity.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl FromStr for QubitId {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some(index) = value.strip_prefix('q').or_else(|| value.strip_prefix('Q')) else {
            return Err(IdentifierError::InvalidSyntax {
                kind: "qubit",
                value: value.to_owned(),
            });
        };
        let index = index
            .parse::<u16>()
            .map_err(|_| IdentifierError::QubitOutOfRange)?;
        Ok(Self(index))
    }
}

impl fmt::Display for QubitId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "q{}", self.0)
    }
}

/// Configured execution trace detail.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TraceLevel {
    /// No trace events.
    Off,
    /// Lifecycle and aggregate events.
    Summary,
    /// Every executed instruction.
    #[default]
    Instructions,
    /// Instruction events with safe small-state summaries.
    StateSmall,
    /// Maximum bounded debug detail.
    FullDebug,
}

/// Program execution metadata derived from directives.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProgramMetadata {
    /// Declared program name.
    pub name: String,
    /// User seed.
    pub seed: u64,
    /// Number of independent shots.
    pub shots: u64,
    /// Requested trace level.
    pub trace_level: TraceLevel,
    /// Optional total-memory budget in bytes.
    pub memory_budget_bytes: Option<u64>,
}

impl Default for ProgramMetadata {
    fn default() -> Self {
        Self {
            name: "anonymous".to_owned(),
            seed: 0,
            shots: 1,
            trace_level: TraceLevel::Instructions,
            memory_budget_bytes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_registers_reject_out_of_range_values() {
        assert_eq!(ClassicalRegister::new(15).expect("r15").index(), 15);
        assert_eq!(
            ClassicalRegister::new(16),
            Err(IdentifierError::RegisterOutOfRange(16))
        );
    }

    #[test]
    fn typed_identifiers_parse_case_insensitively() {
        assert_eq!("R3".parse::<ClassicalRegister>().expect("R3").index(), 3);
        assert_eq!("m8".parse::<MeasurementRegister>().expect("m8").index(), 8);
        assert_eq!("Q21".parse::<QubitId>().expect("Q21").index(), 21);
    }
}
