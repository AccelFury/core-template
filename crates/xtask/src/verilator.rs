// SPDX-License-Identifier: AGPL-3.0-or-later
use std::process::Command;
use std::string::String;

#[derive(Debug)]
pub struct ExecResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run(cmd: &mut Command) -> Result<ExecResult, String> {
    let output = cmd.output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(ExecResult {
        status: output.status.code().unwrap_or(-1),
        stdout,
        stderr,
    })
}

pub fn check_status(result: ExecResult, label: &str) -> Result<(), String> {
    if result.status != 0 {
        Err(format!(
            "{label} failed (code {}):\n{}\n{}",
            result.status, result.stdout, result.stderr
        ))
    } else {
        Ok(())
    }
}
