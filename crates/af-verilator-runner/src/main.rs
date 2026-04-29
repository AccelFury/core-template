// SPDX-License-Identifier: AGPL-3.0-or-later
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;

mod commands;

#[derive(clap::Parser)]
#[command(name = "af-verilator-runner", version)]
struct Args {
    #[arg(long, default_value = "tb_af_mod_add")]
    top_module: String,
    #[arg(long, default_value = "tb/verilator/filelist.f")]
    filelist: String,
    #[arg(long, default_value = "reports/simulation")]
    logs_dir: String,
}

fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let compile_cmd = commands::verilator_compile_command(&args.top_module, &args.filelist);
    let compiled =
        commands::run_command(compile_cmd, "verilator --binary").map_err(|e| e.to_string())?;

    std::fs::create_dir_all(&args.logs_dir).map_err(|e| e.to_string())?;
    std::fs::write(
        Path::new(&args.logs_dir).join("verilator_compile.log"),
        format!("{}\n{}", compiled.stdout, compiled.stderr),
    )
    .map_err(|e| e.to_string())?;

    let binary_path = format!("obj_dir/V{}", args.top_module);
    let fallback_binary_path = format!("./V{}", args.top_module);
    let binary_path = if Path::new(&binary_path).exists() {
        binary_path
    } else if Path::new(&fallback_binary_path).exists() {
        fallback_binary_path
    } else {
        return Err("verilator binary not produced".to_string());
    };

    let sim = commands::verilator_sim_command(&binary_path, &[]);
    let sim_output = commands::run_command(sim, "simulator").map_err(|e| e.to_string())?;
    let log_file = Path::new(&args.logs_dir).join(format!("sim_{}.log", timestamp()));
    let log = format!(
        "status={}\nstdout=\n{}\nstderr=\n{}\n",
        sim_output.status, sim_output.stdout, sim_output.stderr
    );
    std::fs::write(log_file, log).map_err(|e| e.to_string())?;
    Ok(())
}
