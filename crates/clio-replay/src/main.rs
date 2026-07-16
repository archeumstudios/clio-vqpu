//! Replay package command-line interface.

#![forbid(unsafe_code)]

use clio_replay::{ReplayPackage, create_package, verify_package};
use clio_resource::ResourceLimits;
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.first().map(String::as_str) {
        Some("pack") => {
            let source_path = arguments.get(1).ok_or("missing source path")?;
            let output_path = arguments.get(2).ok_or("missing package output path")?;
            let source = fs::read_to_string(source_path)?;
            let package = create_package(&source, ResourceLimits::default())?;
            fs::write(output_path, serde_json::to_vec_pretty(&package)?)?;
            println!("wrote replay package {output_path}");
        }
        Some("verify") => {
            let package_path = arguments.get(1).ok_or("missing package path")?;
            let package: ReplayPackage = serde_json::from_slice(&fs::read(package_path)?)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&verify_package(
                    &package,
                    ResourceLimits::default()
                )?)?
            );
        }
        _ => return Err("usage: clio-replay <pack SOURCE OUTPUT|verify PACKAGE>".into()),
    }
    Ok(())
}
