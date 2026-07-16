//! Reproducible benchmark evidence generator.

#![forbid(unsafe_code)]

use clio_backend::{QuantumBackend, SingleQubitGate};
use clio_core::QubitId;
use clio_engine::StateVectorEngine;
use clio_resource::ResourceLimits;
use clio_sdk::execute;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{env, fmt::Write as _, fs, path::Path, process::Command, time::Instant};

#[derive(Serialize)]
struct Environment {
    schema: &'static str,
    os: String,
    architecture: String,
    rustc: String,
    profile: &'static str,
    repetitions: usize,
    smoke: bool,
    git_commit: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let smoke = arguments.iter().any(|value| value == "--smoke");
    let output = arguments
        .windows(2)
        .find(|pair| pair[0] == "--output-dir")
        .map_or("research/benchmarks", |pair| pair[1].as_str());
    let repetitions = if smoke { 2 } else { 10 };
    let output = Path::new(output);
    fs::create_dir_all(output.join("raw"))?;
    fs::create_dir_all(output.join("environment"))?;

    let mut csv = String::from(
        "schema,family,parameter,repetition,elapsed_ns,shots,qubits,instructions,trace_events,estimated_bytes,serialized_result_bytes,actual_trace_bytes\n",
    );
    let max_qubits = if smoke { 10 } else { 18 };
    for qubits in 1..=max_qubits {
        benchmark(
            &mut csv,
            "qubits",
            qubits,
            repetitions,
            &state_workload(qubits, 32, "off"),
        )?;
    }
    for depth in [1, 8, 32, 128, 512] {
        benchmark(
            &mut csv,
            "depth",
            depth,
            repetitions,
            &state_workload(8, depth, "off"),
        )?;
    }
    for shots in [1, 16, 64, 256, 1024] {
        benchmark(
            &mut csv,
            "shots",
            shots,
            repetitions,
            &bell_workload(shots, "off"),
        )?;
    }
    for (parameter, trace) in [(0, "off"), (1, "instructions"), (2, "state-small")] {
        benchmark(
            &mut csv,
            "trace",
            parameter,
            repetitions,
            &bell_workload(128, trace),
        )?;
    }
    benchmark_direct_engine(&mut csv, repetitions, 8, 128)?;
    benchmark(
        &mut csv,
        "abstraction",
        1,
        repetitions,
        &state_workload(8, 128, "off"),
    )?;

    let raw_path = output.join("raw/benchmark-results.csv");
    fs::write(&raw_path, &csv)?;
    let environment = Environment {
        schema: "clio-benchmark-environment-1",
        os: env::consts::OS.to_owned(),
        architecture: env::consts::ARCH.to_owned(),
        rustc: command_output("rustc", &["--version"]),
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        repetitions,
        smoke,
        git_commit: command_output("git", &["rev-parse", "HEAD"]),
    };
    fs::write(
        output.join("environment/environment.json"),
        serde_json::to_string_pretty(&environment)?,
    )?;
    let checksum = format!("{:x}", Sha256::digest(csv.as_bytes()));
    fs::write(
        output.join("raw/benchmark-results.sha256"),
        format!("{checksum}  benchmark-results.csv\n"),
    )?;
    println!(
        "wrote {} records; sha256={checksum}",
        csv.lines().count().saturating_sub(1)
    );
    Ok(())
}

fn benchmark_direct_engine(
    csv: &mut String,
    repetitions: usize,
    qubits: usize,
    depth: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for repetition in 0..=repetitions {
        let start = Instant::now();
        let mut engine = StateVectorEngine::new();
        for qubit in 0..qubits {
            engine.allocate(QubitId::new(u16::try_from(qubit)?))?;
        }
        for layer in 0..depth {
            engine.apply_single(
                QubitId::new(u16::try_from(layer % qubits)?),
                SingleQubitGate::Ry(0.01),
            )?;
        }
        let elapsed = start.elapsed().as_nanos();
        if repetition > 0 {
            let recorded_repetition = repetition - 1;
            writeln!(
                csv,
                "clio-benchmark-record-1,abstraction,0,{recorded_repetition},{elapsed},1,{qubits},{},0,{},0,0",
                qubits + depth,
                engine.amplitudes().len() * 16
            )?;
        }
    }
    Ok(())
}

fn benchmark(
    csv: &mut String,
    family: &str,
    parameter: usize,
    repetitions: usize,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    execute(source, ResourceLimits::default())?;
    for repetition in 0..repetitions {
        let start = Instant::now();
        let result = execute(source, ResourceLimits::default())?;
        let elapsed = start.elapsed().as_nanos();
        let serialized_result_bytes = serde_json::to_vec(&result)?.len();
        let actual_trace_bytes = serde_json::to_vec(&result.trace)?.len();
        writeln!(
            csv,
            "clio-benchmark-record-1,{family},{parameter},{repetition},{elapsed},{},{},{},{},{},{serialized_result_bytes},{actual_trace_bytes}",
            result.shots,
            result.plan.qubits,
            result.final_state.instruction_counter,
            result.trace.events.len(),
            result.plan.estimated_total_bytes
        )?;
    }
    Ok(())
}

fn state_workload(qubits: usize, depth: usize, trace: &str) -> String {
    let mut source = format!(".program bench_state\n.shots 1\n.trace {trace}\n");
    for qubit in 0..qubits {
        writeln!(source, "QALLOC q{qubit}").expect("String write");
    }
    for layer in 0..depth {
        writeln!(source, "QRY q{}, 0.01", layer % qubits).expect("String write");
    }
    source.push_str("HALT\n");
    source
}

fn bell_workload(shots: usize, trace: &str) -> String {
    format!(
        ".program bench_bell\n.seed 42\n.shots {shots}\n.trace {trace}\nQALLOC q0\nQALLOC q1\nQH q0\nQCX q0, q1\nQMEASURE q0, m0\nQMEASURE q1, m1\nHALT\n"
    )
}

fn command_output(program: &str, arguments: &[&str]) -> String {
    Command::new(program).args(arguments).output().map_or_else(
        |_| "unavailable".to_owned(),
        |output| {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_owned()
            } else {
                "unavailable-uncommitted".to_owned()
            }
        },
    )
}
