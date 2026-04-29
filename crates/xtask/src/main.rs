// SPDX-License-Identifier: AGPL-3.0-or-later
mod gowin;
mod lattice_oss;
mod quartus;
mod reports;
mod spdx;
mod tasks_new;
mod verilator;
mod vivado;

use clap::Parser;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum RtlLanguage {
    #[value(name = "verilog-2001")]
    Verilog2001,
    #[value(name = "verilog-2005")]
    Verilog2005,
    #[value(name = "systemverilog")]
    SystemVerilog,
}

#[derive(Parser)]
#[command(name = "xtask", version)]
enum Command {
    Vectors,
    #[command(name = "lint-rtl")]
    LintRtl {
        #[arg(long, value_enum, default_value = "systemverilog")]
        language: RtlLanguage,
    },
    #[command(name = "sim-verilator")]
    SimVerilator {
        #[arg(long, value_enum, default_value = "systemverilog")]
        language: RtlLanguage,
    },
    #[command(name = "check-spdx")]
    CheckSpdx,
    #[command(name = "check-docs")]
    CheckDocs,
    #[command(name = "synth-gowin")]
    SynthGowin {
        board: String,
    },
    SynthVivado {
        board: String,
    },
    #[command(name = "synth-quartus")]
    SynthQuartus {
        board: String,
    },
    #[command(name = "synth-lattice-oss")]
    SynthLatticeOss {
        board: String,
    },
    #[command(name = "check-boards")]
    CheckBoards,
    #[command(name = "check-no-python")]
    CheckNoPython,
    #[command(name = "collect-reports")]
    CollectReports,
    #[command(name = "board-matrix")]
    BoardMatrix,
    #[command(name = "new-ip")]
    NewIp {
        name: String,
        category: String,
        #[arg(long, value_enum, default_value = "systemverilog")]
        language: RtlLanguage,
    },
    #[command(name = "new-board")]
    NewBoard {
        #[arg(long)]
        board_id: String,
        #[arg(long)]
        vendor: String,
        #[arg(long)]
        family: String,
        #[arg(long, value_name = "format")]
        constraint_format: String,
    },
}

fn main() -> Result<(), String> {
    let cmd = Command::parse();
    match cmd {
        Command::Vectors => tasks_new::run_vectors(),
        Command::LintRtl { language } => tasks_new::lint_rtl(&language),
        Command::SimVerilator { language } => tasks_new::sim_verilator(&language),
        Command::CheckSpdx => tasks_new::check_spdx(),
        Command::CheckDocs => tasks_new::check_docs(),
        Command::SynthGowin { board } => tasks_new::synth_gowin(&board),
        Command::SynthVivado { board } => tasks_new::synth_vivado(&board),
        Command::SynthQuartus { board } => tasks_new::synth_quartus(&board),
        Command::SynthLatticeOss { board } => tasks_new::synth_lattice_oss(&board),
        Command::CheckBoards => tasks_new::check_boards(),
        Command::CheckNoPython => tasks_new::check_no_python(),
        Command::CollectReports => tasks_new::collect_reports(),
        Command::BoardMatrix => tasks_new::board_matrix(),
        Command::NewIp {
            name,
            category,
            language,
        } => tasks_new::new_ip(&name, &category, language),
        Command::NewBoard {
            board_id,
            vendor,
            family,
            constraint_format,
        } => tasks_new::new_board(&board_id, &vendor, &family, &constraint_format),
    }
}
