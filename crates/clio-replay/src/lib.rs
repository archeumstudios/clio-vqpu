//! Portable, checksummed Clio replay packages.

#![forbid(unsafe_code)]

use clio_isa::Executable;
use clio_resource::ResourceLimits;
use clio_runtime::{ENGINE_IDENTITY, ExecutionResult, RNG_IDENTITY};
use clio_sdk::{SdkError, build, execute};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Stable replay envelope schema.
pub const REPLAY_FORMAT: &str = "clio-replay-package-1";

/// Self-contained replay envelope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayPackage {
    /// Package schema.
    pub format: String,
    /// Original Clio source.
    pub source: String,
    /// Typed assembled executable.
    pub executable: Executable,
    /// Original deterministic result.
    pub result: ExecutionResult,
    /// Required engine semantic identity.
    pub engine: String,
    /// Required RNG semantic identity.
    pub rng: String,
    /// Producer OS.
    pub producer_os: String,
    /// Producer architecture.
    pub producer_architecture: String,
    /// SHA-256 over all preceding semantic payload fields.
    pub payload_sha256: String,
}

/// Successful replay verification details.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReplayVerification {
    /// Package integrity and compatibility passed.
    pub passed: bool,
    /// Source hash recovered from reassembly.
    pub source_sha256: String,
    /// Whether reproduced result equals the retained result exactly.
    pub exact_result_match: bool,
}

/// Replay creation or verification failure.
#[derive(Debug, Error)]
pub enum ReplayError {
    /// Build or execution failed.
    #[error(transparent)]
    Sdk(#[from] SdkError),
    /// Package bytes or compatibility identity do not match.
    #[error("replay package is incompatible or corrupted: {0}")]
    Incompatible(String),
    /// JSON encoding failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Builds, executes, and packages a source program.
pub fn create_package(source: &str, limits: ResourceLimits) -> Result<ReplayPackage, ReplayError> {
    let executable = build(source)?;
    let result = execute(source, limits)?;
    let mut package = ReplayPackage {
        format: REPLAY_FORMAT.to_owned(),
        source: source.to_owned(),
        executable,
        result,
        engine: ENGINE_IDENTITY.to_owned(),
        rng: RNG_IDENTITY.to_owned(),
        producer_os: std::env::consts::OS.to_owned(),
        producer_architecture: std::env::consts::ARCH.to_owned(),
        payload_sha256: String::new(),
    };
    package.payload_sha256 = payload_hash(&package)?;
    Ok(package)
}

/// Checks integrity and semantic compatibility, then re-executes exactly.
pub fn verify_package(
    package: &ReplayPackage,
    limits: ResourceLimits,
) -> Result<ReplayVerification, ReplayError> {
    if package.format != REPLAY_FORMAT {
        return Err(ReplayError::Incompatible(
            "unknown package schema".to_owned(),
        ));
    }
    if package.engine != ENGINE_IDENTITY {
        return Err(ReplayError::Incompatible(
            "engine identity mismatch".to_owned(),
        ));
    }
    if package.rng != RNG_IDENTITY {
        return Err(ReplayError::Incompatible(
            "RNG identity mismatch".to_owned(),
        ));
    }
    if payload_hash(package)? != package.payload_sha256 {
        return Err(ReplayError::Incompatible(
            "payload checksum mismatch".to_owned(),
        ));
    }
    let rebuilt = build(&package.source)?;
    if rebuilt != package.executable {
        return Err(ReplayError::Incompatible(
            "assembled executable mismatch".to_owned(),
        ));
    }
    let reproduced = execute(&package.source, limits)?;
    let exact_result_match = reproduced == package.result;
    if !exact_result_match {
        return Err(ReplayError::Incompatible(
            "deterministic result mismatch".to_owned(),
        ));
    }
    Ok(ReplayVerification {
        passed: true,
        source_sha256: rebuilt.source_sha256,
        exact_result_match,
    })
}

fn payload_hash(package: &ReplayPackage) -> Result<String, serde_json::Error> {
    let mut copy = package.clone();
    copy.payload_sha256.clear();
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(&copy)?)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BELL: &str = ".seed 42\n.shots 32\n.trace instructions\nQALLOC q0\nQALLOC q1\nQH q0\nQCX q0, q1\nQMEASURE q0, m0\nQMEASURE q1, m1\nHALT\n";

    #[test]
    fn package_roundtrip_replays_exactly() {
        let package = create_package(BELL, ResourceLimits::default()).expect("package");
        let encoded = serde_json::to_vec(&package).expect("encode");
        let decoded: ReplayPackage = serde_json::from_slice(&encoded).expect("decode");
        assert!(
            verify_package(&decoded, ResourceLimits::default())
                .expect("verify")
                .passed
        );
    }

    #[test]
    fn mutation_and_identity_drift_are_rejected() {
        let mut package = create_package(BELL, ResourceLimits::default()).expect("package");
        package.source.push_str("# mutation\n");
        assert!(matches!(
            verify_package(&package, ResourceLimits::default()),
            Err(ReplayError::Incompatible(_))
        ));
        let mut package = create_package(BELL, ResourceLimits::default()).expect("package");
        package.engine = "future-engine".to_owned();
        assert!(matches!(
            verify_package(&package, ResourceLimits::default()),
            Err(ReplayError::Incompatible(_))
        ));
    }
}
