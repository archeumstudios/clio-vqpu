//! Clio CLI process entry point for the executable Bell-state workflow.

#![forbid(unsafe_code)]

use clio_sdk::{
    ExecutionLimits, SdkError, build, disassemble, estimate, execute, inspect, observe, validate,
};
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match dispatch(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("{message}");
            ExitCode::from(code)
        }
    }
}

fn dispatch(arguments: &[String]) -> Result<(), (u8, String)> {
    let Some(command) = arguments.first().map(String::as_str) else {
        print_help();
        return Ok(());
    };
    if matches!(command, "help" | "--help" | "-h") {
        print_help();
        return Ok(());
    }
    if matches!(command, "version" | "--version" | "-V") {
        println!("clio foundation-build {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let path = arguments
        .get(1)
        .ok_or_else(|| (2, format!("clio {command}: missing source path")))?;
    let json = arguments.iter().any(|argument| argument == "--json");
    let source =
        fs::read_to_string(path).map_err(|error| (3, format!("failed to read {path}: {error}")))?;
    let limits = parse_limits(arguments)?;

    match command {
        "check" => match build(&source) {
            Ok(executable) => {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "valid": true,
                            "program": executable.metadata.name,
                            "instructions": executable.instructions.len(),
                            "source_sha256": executable.source_sha256,
                        })
                    );
                } else {
                    println!(
                        "Valid Clio program: {} ({} instructions)",
                        executable.metadata.name,
                        executable.instructions.len()
                    );
                }
                Ok(())
            }
            Err(error) => Err(sdk_error(4, error, json)),
        },
        "build" => match build(&source) {
            Ok(executable) => {
                let output = serde_json::to_string_pretty(&executable)
                    .map_err(|error| (5, format!("serialization failed: {error}")))?;
                println!("{output}");
                Ok(())
            }
            Err(error) => Err(sdk_error(4, error, json)),
        },
        "disasm" => match disassemble(&source) {
            Ok(listing) => {
                print!("{listing}");
                Ok(())
            }
            Err(error) => Err(sdk_error(4, error, json)),
        },
        "estimate" => match estimate(&source, limits) {
            Ok(plan) => {
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&plan)
                            .map_err(|error| (5, error.to_string()))?
                    );
                } else {
                    println!("Clio execution plan");
                    println!("Virtual qubits: {}", plan.qubits);
                    println!("Shots: {}", plan.shots);
                    println!("Raw state memory: {} bytes", plan.raw_state_bytes);
                    println!(
                        "Estimated total memory: {} bytes",
                        plan.estimated_total_bytes
                    );
                    println!("Status: ADMITTED");
                }
                Ok(())
            }
            Err(error) => Err(sdk_error(6, error, json)),
        },
        "run" | "trace" => match execute(&source, limits) {
            Ok(result) => {
                if json || command == "trace" {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&result)
                            .map_err(|error| (5, error.to_string()))?
                    );
                } else {
                    println!("Program: {}", result.source_sha256);
                    println!("Shots: {}", result.shots);
                    println!("Measurement counts:");
                    for (outcome, count) in result.measurement_counts {
                        println!("  {outcome}: {count}");
                    }
                    println!("Status: {:?}", result.final_state.status);
                    println!("Trace events: {}", result.trace.events.len());
                }
                Ok(())
            }
            Err(error) => Err(sdk_error(7, error, json)),
        },
        "inspect" => match inspect(&source, limits) {
            Ok(result) => {
                let snapshots = result
                    .trace
                    .events
                    .iter()
                    .filter_map(|event| {
                        event.state_snapshot.as_ref().map(|snapshot| {
                            (event.program_counter, event.mnemonic.as_str(), snapshot)
                        })
                    })
                    .collect::<Vec<_>>();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&snapshots)
                        .map_err(|error| (5, error.to_string()))?
                );
                Ok(())
            }
            Err(error) => Err(sdk_error(7, error, json)),
        },
        "observe" => match observe(&source, limits) {
            Ok(report) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| (5, error.to_string()))?
                );
                Ok(())
            }
            Err(error) => Err(sdk_error(7, error, json)),
        },
        "validate" => match validate(&source, limits) {
            Ok(report) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| (5, error.to_string()))?
                );
                Ok(())
            }
            Err(error) => Err(sdk_error(8, error, json)),
        },
        _ => Err((
            2,
            format!(
                "unknown command `{command}`; executable commands are check, build, disasm, estimate, run, trace, inspect, observe, and validate"
            ),
        )),
    }
}

fn parse_limits(arguments: &[String]) -> Result<ExecutionLimits, (u8, String)> {
    let mut limits = ExecutionLimits::default();
    let mut index = 2;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--json" => index += 1,
            "--instruction-limit" => {
                let value = arguments.get(index + 1).ok_or_else(|| {
                    (
                        2,
                        "--instruction-limit requires a positive integer".to_owned(),
                    )
                })?;
                limits.max_instructions = value
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| (2, "invalid --instruction-limit".to_owned()))?;
                index += 2;
            }
            "--time-limit-ms" => {
                let value = arguments.get(index + 1).ok_or_else(|| {
                    (
                        2,
                        "--time-limit-ms requires a nonnegative integer".to_owned(),
                    )
                })?;
                limits.max_execution_millis = value
                    .parse::<u64>()
                    .map_err(|_| (2, "invalid --time-limit-ms".to_owned()))?;
                index += 2;
            }
            option if option.starts_with('-') => {
                return Err((2, format!("unknown option `{option}`")));
            }
            _ => index += 1,
        }
    }
    Ok(limits)
}

fn sdk_error(code: u8, error: SdkError, json: bool) -> (u8, String) {
    if json {
        let details = match &error {
            SdkError::Validation(diagnostics) => {
                serde_json::to_value(diagnostics).unwrap_or_else(|_| serde_json::json!([]))
            }
            _ => serde_json::json!([]),
        };
        (
            code,
            serde_json::json!({
                "ok": false,
                "code": error.code(),
                "error": error.to_string(),
                "diagnostics": details,
            })
            .to_string(),
        )
    } else if let SdkError::Validation(diagnostics) = error {
        let rendered = diagnostics
            .into_iter()
            .map(|diagnostic| {
                let location = diagnostic.span.map_or_else(String::new, |span| {
                    format!(" at {}:{}", span.line, span.column)
                });
                let help = diagnostic
                    .help
                    .map_or_else(String::new, |help| format!("\n  help: {help}"));
                format!(
                    "error[{}]{location}: {}{help}",
                    diagnostic.code, diagnostic.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        (code, rendered)
    } else {
        (code, error.to_string())
    }
}

fn print_help() {
    println!("Clio VQPU foundation CLI");
    println!(
        "Usage: clio <check|build|disasm|estimate|run|trace|inspect|observe|validate> <program.clio> [options]"
    );
    println!("Options: --json --instruction-limit N --time-limit-ms N");
    println!("       clio <help|version>");
    println!("This internal build executes the verified ISA subset documented in Clio Spec.");
}
