// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ffi::OsString;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("command `{0}` failed: {1}")]
    CommandFailed(String, String),
}

pub struct CommandResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_command(mut cmd: Command, label: &str) -> Result<CommandResult, RunnerError> {
    let output = cmd
        .output()
        .map_err(|e| RunnerError::CommandFailed(label.to_string(), e.to_string()))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let status = output.status.code().unwrap_or(-1);
    if output.status.success() {
        Ok(CommandResult {
            status,
            stdout,
            stderr,
        })
    } else {
        Err(RunnerError::CommandFailed(
            label.to_string(),
            format!("status={status}, stdout={stdout}, stderr={stderr}"),
        ))
    }
}

pub fn verilator_compile_command(top_module: &str, filelist: &str) -> Command {
    let mut cmd = Command::new("verilator");
    cmd.arg("--binary")
        .arg("-j")
        .arg("0")
        .arg("-Wall")
        .arg("--assert")
        .arg("--trace")
        .arg("-CFLAGS")
        .arg("-std=c++20")
        .arg("-MAKEFLAGS")
        .arg("VM_PARALLEL_BUILDS=0")
        .arg("-Wno-TIMESCALEMOD")
        .arg("-Wno-INITIALDLY")
        .arg("--top-module")
        .arg(top_module)
        .arg("-f")
        .arg(filelist);
    cmd
}

pub fn verilator_sim_command(binary: &str, args: &[OsString]) -> Command {
    let mut cmd = Command::new(binary);
    cmd.args(args);
    cmd
}
