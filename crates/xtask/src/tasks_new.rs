// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use af_board_db::{assert_category, find_board, load_boards, BoardEntry};
use serde::Deserialize;

use crate::RtlLanguage;
use crate::{gowin, lattice_oss, quartus, reports, spdx, verilator, vivado};

const DOC_FILES: [&str; 14] = [
    "docs/00_overview.md",
    "docs/01_architecture.md",
    "docs/02_interface.md",
    "docs/03_microarchitecture.md",
    "docs/04_verification_plan.md",
    "docs/05_timing_and_constraints.md",
    "docs/06_resource_report.md",
    "docs/07_board_targets.md",
    "docs/08_release_checklist.md",
    "docs/09_portability_rules.md",
    "docs/10_commercial_licensing.md",
    "docs/11_fpga_family_notes.md",
    "docs/12_toolchain_notes.md",
    "docs/13_security_and_side_channels.md",
];

fn language_path_suffix(language: &RtlLanguage) -> &'static str {
    match language {
        RtlLanguage::Verilog2001 | RtlLanguage::Verilog2005 => "v",
        RtlLanguage::SystemVerilog => "sv",
    }
}

fn language_label(language: &RtlLanguage) -> &'static str {
    match language {
        RtlLanguage::Verilog2001 => "verilog-2001",
        RtlLanguage::Verilog2005 => "verilog-2005",
        RtlLanguage::SystemVerilog => "systemverilog",
    }
}

fn filelist_for_language(language: &RtlLanguage) -> &'static str {
    match language {
        RtlLanguage::Verilog2001 => "tb/verilator/filelist_verilog2001.f",
        RtlLanguage::Verilog2005 => "tb/verilator/filelist_verilog2005.f",
        RtlLanguage::SystemVerilog => "tb/verilator/filelist.f",
    }
}

fn new_ip_core_sv(name: &str) -> String {
    format!(
        r#"// SPDX-License-Identifier: CERN-OHL-S-2.0
module {name} #(
  parameter int FIELD_BITS = 64,
  parameter logic [FIELD_BITS-1:0] MODULUS = 64'hFFFF_FFFF_0000_0001
) (
  input  logic                  i_clk,
  input  logic                  i_rst_n,
  input  logic                  i_valid,
  output logic                  o_ready,
  input  logic [FIELD_BITS-1:0] i_a,
  input  logic [FIELD_BITS-1:0] i_b,
  output logic                  o_valid,
  input  logic                  i_ready,
  output logic [FIELD_BITS-1:0] o_result
);
  logic [FIELD_BITS:0] sum_full;
  logic [FIELD_BITS-1:0] sum_mod;
  logic full;

  assign o_ready = i_ready || !full;
  assign sum_full = {{1'b0, i_a}} + {{1'b0, i_b}};
  assign sum_mod = (sum_full >= {{1'b0, MODULUS}}) ? sum_full[FIELD_BITS-1:0] - MODULUS : sum_full[FIELD_BITS-1:0];
  assign o_valid = full;

  always_ff @(posedge i_clk or negedge i_rst_n) begin
    if (!i_rst_n) begin
      full <= 1'b0;
      o_result <= '0;
    end else begin
      if (i_valid && o_ready) begin
        full <= 1'b1;
        o_result <= sum_mod;
      end else if (i_ready) begin
        full <= 1'b0;
      end
    end
  end
endmodule
"#,
        name = name
    )
}

fn new_ip_top_sv(name: &str) -> String {
    format!(
        r#"// SPDX-License-Identifier: CERN-OHL-S-2.0
module {name}_top #(
  parameter int FIELD_BITS = 64,
  parameter logic [FIELD_BITS-1:0] MODULUS = 64'hFFFF_FFFF_0000_0001
) (
  input  logic                  i_clk,
  input  logic                  i_rst_n,
  input  logic                  i_valid,
  output logic                  o_ready,
  input  logic [FIELD_BITS-1:0] i_a,
  input  logic [FIELD_BITS-1:0] i_b,
  output logic                  o_valid,
  input  logic                  i_ready,
  output logic [FIELD_BITS-1:0] o_result
);
  {name} #(
    .FIELD_BITS(FIELD_BITS),
    .MODULUS(MODULUS)
  ) u_core (
    .i_clk(i_clk),
    .i_rst_n(i_rst_n),
    .i_valid(i_valid),
    .o_ready(o_ready),
    .i_a(i_a),
    .i_b(i_b),
    .o_valid(o_valid),
    .i_ready(i_ready),
    .o_result(o_result)
  );
endmodule
"#,
        name = name
    )
}

fn new_ip_core_v(name: &str) -> String {
    format!(
        "// SPDX-License-Identifier: CERN-OHL-S-2.0\n\
`timescale 1ns/1ps\n\
module {name} #(\n\
  parameter integer FIELD_BITS = 64,\n\
  parameter [FIELD_BITS-1:0] MODULUS = 64'hFFFF_FFFF_0000_0001\n\
) (\n\
  input  wire                  i_clk,\n\
  input  wire                  i_rst_n,\n\
  input  wire                  i_valid,\n\
  output wire                  o_ready,\n\
  input  wire [FIELD_BITS-1:0] i_a,\n\
  input  wire [FIELD_BITS-1:0] i_b,\n\
  output wire                  o_valid,\n\
  input  wire                  i_ready,\n\
  output reg  [FIELD_BITS-1:0] o_result\n\
);\n\
  wire [FIELD_BITS:0] sum_full;\n\
  reg  [FIELD_BITS-1:0] sum_mod;\n\
  wire accept;\n\
  reg  full;\n\
\n\
  assign o_ready = i_ready || !full;\n\
  assign accept = i_valid && o_ready;\n\
  assign o_valid = full;\n\
  assign sum_full = {{1'b0, i_a}} + {{1'b0, i_b}};\n\
\n\
  always @* begin\n\
    if (sum_full >= {{1'b0, MODULUS}}) begin\n\
      sum_mod = sum_full[FIELD_BITS-1:0] - MODULUS;\n\
    end else begin\n\
      sum_mod = sum_full[FIELD_BITS-1:0];\n\
    end\n\
  end\n\
\n\
  always @(posedge i_clk or negedge i_rst_n) begin\n\
    if (!i_rst_n) begin\n\
      full <= 1'b0;\n\
      o_result <= {{FIELD_BITS{{1'b0}}}};\n\
    end else begin\n\
      if (accept) begin\n\
        full <= 1'b1;\n\
        o_result <= sum_mod;\n\
      end else if (i_ready) begin\n\
        full <= 1'b0;\n\
      end\n\
    end\n\
  end\n\
endmodule\n",
        name = name
    )
}

fn new_ip_top_v(name: &str) -> String {
    format!(
        "// SPDX-License-Identifier: CERN-OHL-S-2.0\n\
`timescale 1ns/1ps\n\
module {name}_top #(\n\
  parameter integer FIELD_BITS = 64,\n\
  parameter [FIELD_BITS-1:0] MODULUS = 64'hFFFF_FFFF_0000_0001\n\
) (\n\
  input  wire                  i_clk,\n\
  input  wire                  i_rst_n,\n\
  input  wire                  i_valid,\n\
  output wire                  o_ready,\n\
  input  wire [FIELD_BITS-1:0] i_a,\n\
  input  wire [FIELD_BITS-1:0] i_b,\n\
  output wire                  o_valid,\n\
  input  wire                  i_ready,\n\
  output wire [FIELD_BITS-1:0] o_result\n\
);\n\
  {name} #(\n\
    .FIELD_BITS(FIELD_BITS),\n\
    .MODULUS(MODULUS)\n\
  ) u_core (\n\
    .i_clk(i_clk),\n\
    .i_rst_n(i_rst_n),\n\
    .i_valid(i_valid),\n\
    .o_ready(o_ready),\n\
    .i_a(i_a),\n\
    .i_b(i_b),\n\
    .o_valid(o_valid),\n\
    .i_ready(i_ready),\n\
    .o_result(o_result)\n\
  );\n\
endmodule\n",
        name = name
    )
}

#[derive(Debug, Deserialize)]
struct Manifest {
    boards: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct ReportManifest {
    label: String,
    path: PathBuf,
    generated_at: u64,
}

fn read_text(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))
}

fn parse_json<T>(path: &str) -> Result<T, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let txt = read_text(path)?;
    serde_json::from_str(&txt).map_err(|e| format!("parse {path}: {e}"))
}

fn manifest_boards() -> Result<Vec<String>, String> {
    let manifest: Manifest = parse_json("ip.manifest.json")?;
    if manifest.boards.is_empty() {
        return Err("ip.manifest.json: boards is empty".to_string());
    }
    let mut seen = HashSet::new();
    for board in &manifest.boards {
        if board.trim().is_empty() {
            return Err("manifest contains empty board id".to_string());
        }
        if !seen.insert(board.clone()) {
            return Err(format!("manifest contains duplicate board id: {board}"));
        }
    }
    Ok(manifest.boards)
}

fn resolve_board(id: &str) -> Result<BoardEntry, String> {
    let boards = load_boards("registries/boards.registry.json")?;
    find_board(&boards, id)
        .cloned()
        .ok_or_else(|| format!("board '{id}' not found in registries/boards.registry.json"))
}

fn constraint_file_name(format: &str) -> &'static str {
    match format {
        "cst" => "pins.cst",
        "pcf" => "pins.pcf",
        "lpf" => "constraints.lpf",
        "qsf" => "project.qsf",
        "xdc" => "constraints.xdc",
        "sdc" => "timing.sdc",
        "pdc" => "constraints.pdc",
        _ => "constraints.txt",
    }
}

fn ensure_exists(path: &str) -> Result<(), String> {
    if Path::new(path).exists() {
        Ok(())
    } else {
        Err(format!("missing required path: {path}"))
    }
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn walk_no_python(root: &Path, forbidden_markers: &[&str], depth: usize) -> Result<(), String> {
    if depth > 50000 {
        return Ok(());
    }
    if !root.exists() {
        return Ok(());
    }
    if root
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name == "node_modules")
    {
        return Err(format!(
            "forbidden dependency directory: {}",
            root.to_string_lossy()
        ));
    }

    let skip = [".git", "target", ".cargo", "obj_dir"];
    if root
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| skip.iter().any(|entry| entry == &name))
    {
        return Ok(());
    }

    if root.is_file() {
        let original = root.to_string_lossy().to_string();
        let normalized = original.replace('\\', "/");
        let lowered = original.to_lowercase();
        let forbidden_extensions = [
            ".py", ".pyc", ".pyo", ".pyd", ".c", ".cc", ".cpp", ".cxx", ".h", ".hh", ".hpp",
            ".hxx", ".sh", ".bash", ".zsh", ".fish",
        ];
        if forbidden_extensions
            .iter()
            .any(|extension| lowered.ends_with(extension))
        {
            return Err(format!("forbidden source/script artifact: {original}"));
        }
        if normalized.contains("/crates/xtask/src/") {
            return Ok(());
        }
        for marker in forbidden_markers {
            if lowered.contains(marker) {
                return Err(format!("forbidden marker in path: {original}"));
            }
        }
        if lowered.contains("/pipfile") || lowered.contains("/poetry.lock") {
            return Err(format!("forbidden python artifact: {original}"));
        }
        return Ok(());
    }

    for entry in
        fs::read_dir(root).map_err(|e| format!("read_dir {}: {e}", root.to_string_lossy()))?
    {
        let entry = entry.map_err(|e| format!("readdir entry: {e}"))?;
        walk_no_python(&entry.path(), forbidden_markers, depth + 1)?;
    }
    Ok(())
}

pub fn run_vectors() -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "af-vectors"]);
    let out = verilator::run(&mut cmd)?;
    verilator::check_status(out, "cargo run -p af-vectors")
}

pub fn lint_rtl(language: &RtlLanguage) -> Result<(), String> {
    let mut cmd = Command::new("verilator");
    let filelist = filelist_for_language(language);
    cmd.args([
        "--lint-only",
        "-Wall",
        "-Wno-DECLFILENAME",
        "-Wno-MULTITOP",
        "-Wno-TIMESCALEMOD",
        "-Wno-INITIALDLY",
        "-Wno-MULTIDRIVEN",
        "--timing",
        "-f",
        filelist,
    ]);
    let out = verilator::run(&mut cmd)?;
    verilator::check_status(
        out,
        &format!("verilator --lint-only ({})", language_label(language)),
    )
}

pub fn sim_verilator(language: &RtlLanguage) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    let filelist = filelist_for_language(language);
    cmd.args([
        "run",
        "-p",
        "af-verilator-runner",
        "--",
        "--top-module",
        "tb_af_mod_add",
        "--filelist",
        filelist,
    ]);
    let out = verilator::run(&mut cmd)?;
    verilator::check_status(
        out,
        &format!(
            "cargo run -p af-verilator-runner ({})",
            language_label(language)
        ),
    )
}

pub fn check_spdx() -> Result<(), String> {
    let missing = spdx::check()?;
    if missing.is_empty() {
        return Ok(());
    }
    let list = missing
        .iter()
        .map(|p| p.to_string_lossy())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!("missing SPDX header in {list}"))
}

pub fn check_docs() -> Result<(), String> {
    for path in DOC_FILES {
        let text = read_text(path)?;
        if text.trim().is_empty() {
            return Err(format!("{path} is empty"));
        }
    }
    Ok(())
}

pub fn synth_gowin(board_id: &str) -> Result<(), String> {
    let entry = resolve_board(board_id)?;
    let report = gowin::synthesize(&entry.board_dir)?;
    println!("gowin synthesis report: {report}");
    Ok(())
}

pub fn synth_vivado(board_id: &str) -> Result<(), String> {
    let entry = resolve_board(board_id)?;
    let report = vivado::synthesize(&entry.board_dir)?;
    println!("vivado synthesis report: {report}");
    Ok(())
}

pub fn synth_quartus(board_id: &str) -> Result<(), String> {
    let entry = resolve_board(board_id)?;
    let report = quartus::synthesize(&entry.board_dir)?;
    println!("quartus synthesis report: {report}");
    Ok(())
}

pub fn synth_lattice_oss(board_id: &str) -> Result<(), String> {
    let entry = resolve_board(board_id)?;
    let report = lattice_oss::synthesize(&entry.board_dir)?;
    println!("lattice oss synthesis report: {report}");
    Ok(())
}

fn collect_status(path: &str) -> Result<serde_json::Value, String> {
    parse_json::<serde_json::Value>(path)
}

fn ensure_board_status_allowed(
    board_dir: &str,
    key: &str,
    value: &str,
    allowed: &[&str],
) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "status.{key} has invalid value '{value}' in {board_dir}/board.status.json"
        ))
    }
}

fn collect_report_paths(root: &str) -> Result<Vec<ReportManifest>, String> {
    let dir = Path::new(root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("read_dir {root}: {e}"))? {
        let e = entry.map_err(|e| e.to_string())?;
        if e.file_type().map_err(|e| e.to_string())?.is_file() {
            let p = e.path();
            let label = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            out.push(ReportManifest {
                label,
                path: p,
                generated_at: timestamp(),
            });
        }
    }
    Ok(out)
}

pub fn collect_reports() -> Result<(), String> {
    let payload = serde_json::json!({
        "generated_at": timestamp(),
        "resources": collect_report_paths("reports/resources")?,
        "timing": collect_report_paths("reports/timing")?,
        "simulation": collect_report_paths("reports/simulation")?,
        "board_bringup": collect_report_paths("reports/board_bringup")?,
    });
    let payload = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    let path = reports::write_report("collect", &payload)?;
    println!("report written: {path}");
    Ok(())
}

fn check_board_entry(entry: &BoardEntry) -> Result<(), String> {
    ensure_exists(&format!("{}/README.md", entry.board_dir))?;
    ensure_exists(&format!("{}/bringup.md", entry.board_dir))?;
    ensure_exists(&format!("{}/constraints/README.md", entry.board_dir))?;
    ensure_exists(&format!(
        "{}/constraints/{}",
        entry.board_dir,
        constraint_file_name(&entry.constraint_format),
    ))?;
    let build_path = match entry.vendor.as_str() {
        "gowin" => format!("{}/gowin/build.tcl", entry.board_dir),
        "lattice" => format!("{}/lattice/build.tcl", entry.board_dir),
        "intel" => format!("{}/quartus/build.tcl", entry.board_dir),
        "xilinx" => format!("{}/vivado/build.tcl", entry.board_dir),
        "efinix" => format!("{}/efinity/build.tcl", entry.board_dir),
        "microchip" => format!("{}/libero/build.tcl", entry.board_dir),
        _ => format!("{}/build.tcl", entry.board_dir),
    };
    ensure_exists(&build_path)?;
    let top_sv = format!("{}/top/af_board_top.sv", entry.board_dir);
    let top_v = format!("{}/top/af_board_top.v", entry.board_dir);
    if !Path::new(&top_sv).exists() && !Path::new(&top_v).exists() {
        return Err(format!("missing top wrapper in {}", entry.board_dir));
    }
    ensure_exists(&format!("{}/board.status.json", entry.board_dir))?;
    let status: serde_json::Value =
        collect_status(&format!("{}/board.status.json", entry.board_dir))?;
    let status_obj = status
        .get("status")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            format!(
                "board.status missing status object in {}/board.status.json",
                entry.board_dir
            )
        })?;
    let status_value = |key: &str| -> Result<&str, String> {
        match status_obj.get(key).and_then(|v| v.as_str()) {
            Some(v) if !v.is_empty() => {}
            _ => {
                return Err(format!(
                    "status.{key} missing or invalid in {}",
                    entry.board_dir
                ))
            }
        }
        Ok(status_obj.get(key).and_then(|v| v.as_str()).unwrap_or(""))
    };

    let sim = status_value("sim")?;
    let synthesis = status_value("synthesis")?;
    let pnr = status_value("pnr")?;
    let hardware_bringup = status_value("hardware_bringup")?;

    ensure_board_status_allowed(
        &entry.board_dir,
        "sim",
        sim,
        &[
            "not_applicable",
            "not_started",
            "template_only",
            "sim_passed",
        ],
    )?;
    ensure_board_status_allowed(
        &entry.board_dir,
        "synthesis",
        synthesis,
        &[
            "not_measured",
            "not_started",
            "template_only",
            "synth_passed",
        ],
    )?;
    ensure_board_status_allowed(
        &entry.board_dir,
        "pnr",
        pnr,
        &["not_measured", "not_started", "template_only", "pnr_passed"],
    )?;
    ensure_board_status_allowed(
        &entry.board_dir,
        "hardware_bringup",
        hardware_bringup,
        &["not_tested", "not_started", "template_only", "board_tested"],
    )?;

    if status
        .get("status")
        .and_then(|v| v.get("template"))
        .and_then(|v| v.as_bool())
        == Some(true)
    {
        if sim != "not_applicable"
            || synthesis != "not_measured"
            || pnr != "not_measured"
            || hardware_bringup != "not_tested"
        {
            return Err(format!(
                "template board must not claim pass status in {}/board.status.json",
                entry.board_dir
            ));
        }
        if status
            .get("warnings")
            .and_then(|v| v.as_array())
            .is_none_or(|warnings| warnings.is_empty())
        {
            return Err(format!(
                "template board must carry draft warnings in {}/board.status.json",
                entry.board_dir
            ));
        }
    }

    if status.get("board").and_then(|v| v.as_str()) != Some(entry.board_id.as_str()) {
        return Err(format!(
            "board.status board mismatch in {}/board.status.json",
            entry.board_dir
        ));
    }
    Ok(())
}

pub fn check_boards() -> Result<(), String> {
    let manifest = manifest_boards()?;
    let registry = load_boards("registries/boards.registry.json")?;
    let set: HashSet<String> = manifest.iter().cloned().collect();
    let mut missing_in_manifest = Vec::new();
    let mut missing_in_registry = Vec::new();
    let mut seen = HashSet::new();

    for entry in &registry {
        if !seen.insert(entry.board_id.clone()) {
            return Err(format!(
                "duplicate board_id in registry: {}",
                entry.board_id
            ));
        }

        if !set.contains(&entry.board_id) {
            missing_in_manifest.push(entry.board_id.clone());
        }
        check_board_entry(entry)?;
    }

    for board_id in manifest {
        if !registry.iter().any(|entry| entry.board_id == board_id) {
            missing_in_registry.push(board_id);
        }
    }

    if !missing_in_manifest.is_empty() {
        return Err(format!(
            "boards.registry contains boards not listed in manifest: {missing_in_manifest:?}",
        ));
    }

    if !missing_in_registry.is_empty() {
        return Err(format!(
            "manifest contains boards missing in boards.registry: {missing_in_registry:?}",
        ));
    }

    if seen.len() < set.len() {
        return Err("registry has fewer unique board entries than manifest".to_string());
    }
    Ok(())
}

pub fn check_no_python() -> Result<(), String> {
    let forbidden = [
        "requirements.txt",
        "pyproject.toml",
        "setup.py",
        "setup.cfg",
        "tox.ini",
        ".python-version",
        "poetry.lock",
        "pipfile",
        "pipfile.lock",
        "package.json",
        "node_modules",
    ];
    walk_no_python(Path::new("."), &forbidden, 0)
}

pub fn board_matrix() -> Result<(), String> {
    let manifest = manifest_boards()?;
    let manifest_set: HashSet<String> = manifest.iter().cloned().collect();
    let registry = load_boards("registries/boards.registry.json")?;

    let mut rows = vec![vec![
        "board_id".to_string(),
        "vendor".to_string(),
        "family".to_string(),
        "constraint_format".to_string(),
        "in_manifest".to_string(),
        "synthesis".to_string(),
        "pnr".to_string(),
        "hardware_bringup".to_string(),
    ]];

    for entry in registry {
        let status_file = format!("{}/board.status.json", entry.board_dir);
        let status = parse_json::<serde_json::Value>(&status_file)
            .unwrap_or_else(|_| serde_json::json!({ "status": {} }));
        let status = status
            .get("status")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let in_manifest = if manifest_set.contains(&entry.board_id) {
            "yes"
        } else {
            "no"
        };
        let synthesis = status
            .get("synthesis")
            .and_then(|v| v.as_str())
            .unwrap_or("not_measured");
        let pnr = status
            .get("pnr")
            .and_then(|v| v.as_str())
            .unwrap_or("not_measured");
        let hardware_bringup = status
            .get("hardware_bringup")
            .and_then(|v| v.as_str())
            .unwrap_or("not_tested");

        rows.push(vec![
            entry.board_id,
            entry.vendor,
            entry.fpga_family,
            entry.constraint_format,
            in_manifest.to_string(),
            synthesis.to_string(),
            pnr.to_string(),
            hardware_bringup.to_string(),
        ]);
    }

    let out = format_markdown_table(&rows)?;
    fs::write("docs/board_matrix.md", out).map_err(|e| e.to_string())?;
    Ok(())
}

fn format_markdown_table(rows: &[Vec<String>]) -> Result<String, String> {
    if rows.is_empty() {
        return Ok(String::new());
    }
    let columns = rows[0].len();
    let mut widths = vec![0usize; columns];
    for row in rows {
        if row.len() != columns {
            return Err("board matrix row has inconsistent column count".to_string());
        }
        for (column, cell) in row.iter().enumerate() {
            widths[column] = widths[column].max(cell.len());
        }
    }

    let mut out = String::new();
    write_markdown_row(&mut out, &rows[0], &widths)?;
    let separator = widths
        .iter()
        .map(|width| "-".repeat(*width))
        .collect::<Vec<_>>();
    write_markdown_row(&mut out, &separator, &widths)?;
    for row in &rows[1..] {
        write_markdown_row(&mut out, row, &widths)?;
    }
    Ok(out)
}

fn write_markdown_row(out: &mut String, row: &[String], widths: &[usize]) -> Result<(), String> {
    out.push('|');
    for (column, cell) in row.iter().enumerate() {
        write!(out, " {:width$} |", cell, width = widths[column]).map_err(|e| e.to_string())?;
    }
    out.push('\n');
    Ok(())
}

pub fn new_ip(name: &str, category: &str, language: RtlLanguage) -> Result<(), String> {
    if !name.starts_with("af_") {
        return Err("ip name must start with af_".to_string());
    }
    assert_category(category)?;
    if !category.is_empty() {
        let allowed = af_board_db::categories();
        if !allowed.iter().any(|value| value == category) {
            return Err(format!("unsupported category: {category}"));
        }
    }

    let ext = language_path_suffix(&language);
    let core_path = format!("rtl/core/{name}.{ext}");
    let top_path = format!("rtl/core/{name}_top.{ext}");
    if Path::new(&core_path).exists() || Path::new(&top_path).exists() {
        return Err("target RTL files already exist".to_string());
    }

    let (core, top) = match language {
        RtlLanguage::Verilog2001 | RtlLanguage::Verilog2005 => {
            (new_ip_core_v(name), new_ip_top_v(name))
        }
        RtlLanguage::SystemVerilog => (new_ip_core_sv(name), new_ip_top_sv(name)),
    };
    fs::write(&core_path, core).map_err(|e| format!("write {core_path}: {e}"))?;
    fs::write(&top_path, top).map_err(|e| format!("write {top_path}: {e}"))?;

    println!("created {core_path} ({})", language_label(&language));
    println!("created {top_path} ({})", language_label(&language));
    Ok(())
}

pub fn new_board(
    board_id: &str,
    vendor: &str,
    family: &str,
    constraint_format: &str,
) -> Result<(), String> {
    if board_id.trim().is_empty() || vendor.trim().is_empty() || family.trim().is_empty() {
        return Err("board_id, vendor, family are required".to_string());
    }

    let allowed_formats = ["cst", "pcf", "lpf", "qsf", "sdc", "xdc", "pdc"];
    if !allowed_formats.contains(&constraint_format) {
        return Err(format!("unsupported constraint_format {constraint_format}"));
    }

    let dir = format!("boards/{vendor}/{board_id}");
    if Path::new(&dir).exists() && !Path::new(&dir).is_dir() {
        return Err(format!(
            "board directory exists and is not a directory: {dir}"
        ));
    }
    fs::create_dir_all(format!("{dir}/top")).map_err(|e| e.to_string())?;
    fs::create_dir_all(format!("{dir}/constraints")).map_err(|e| e.to_string())?;

    let build_dir = match vendor {
        "gowin" => format!("{dir}/gowin"),
        "lattice" => format!("{dir}/lattice"),
        "intel" => format!("{dir}/quartus"),
        "xilinx" => format!("{dir}/vivado"),
        "efinix" => format!("{dir}/efinity"),
        "microchip" => format!("{dir}/libero"),
        _ => format!("{dir}/build"),
    };
    fs::create_dir_all(&build_dir).map_err(|e| e.to_string())?;

    let toolchain = match vendor {
        "gowin" => "gowin_eda",
        "lattice" => "lattice_oss_yosys_nextpnr_ice40",
        "intel" => "intel_quartus",
        "xilinx" => "xilinx_vivado",
        "efinix" => "efinix_efinity",
        "microchip" => "microchip_libero",
        _ => "unknown",
    };

    let status = serde_json::json!({
        "board": board_id,
        "vendor": vendor,
        "family": family,
        "toolchains": [toolchain],
        "constraint_formats": [constraint_format],
        "status": {
            "template": true,
            "sim": "not_applicable",
            "synthesis": "not_measured",
            "pnr": "not_measured",
            "hardware_bringup": "not_tested"
        },
        "warnings": [
            "Draft only. Verify every pin against the official schematic and board revision before programming hardware."
        ]
    });
    fs::write(
        format!("{dir}/board.status.json"),
        serde_json::to_string_pretty(&status).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    if !Path::new(&format!("{dir}/README.md")).exists() {
        fs::write(
            format!("{dir}/README.md"),
            format!(
                "# {board_id}\n\nTemplate board target.\n\nDraft only. Verify every pin against the official schematic and board revision before programming hardware.\n"
            ),
        )
        .map_err(|e| e.to_string())?;
    }

    if !Path::new(&format!("{dir}/bringup.md")).exists() {
        fs::write(
            format!("{dir}/bringup.md"),
            "# Board bringup\n\n- power rails verified\n- pin map confirmed against schematic\n- UART-free smoke test executed\n- LED heartbeat indicates board activity\n- do not enable external high-speed interfaces by default\n",
        )
        .map_err(|e| e.to_string())?;
    }

    if !Path::new(&format!("{dir}/constraints/README.md")).exists() {
        fs::write(
            format!("{dir}/constraints/README.md"),
            "# Constraint placeholders\n\nDraft only. Verify every pin against the official schematic and board revision before programming hardware.\n",
        )
        .map_err(|e| e.to_string())?;
    }

    if !Path::new(&format!(
        "{dir}/constraints/{}",
        constraint_file_name(constraint_format)
    ))
    .exists()
    {
        fs::write(
            format!("{dir}/constraints/{}", constraint_file_name(constraint_format)),
            format!(
                "// Draft constraints placeholder for {board_id}\n// Draft only. Verify every pin against the official schematic and board revision before programming hardware.\n"
            ),
            )
        .map_err(|e| e.to_string())?;
    }

    let top = r#"// SPDX-License-Identifier: CERN-OHL-S-2.0
  module af_board_top (
  input  wire        i_clk,
  input  wire        i_rst_n,
  output reg [1:0]   led
);
  wire        o_ready;
  localparam int HEARTBEAT_DIV = 22;
  reg  [HEARTBEAT_DIV:0] heartbeat_cnt;
  reg  [63:0] i_a;
  reg  [63:0] i_b;
  reg         i_valid;
  reg         self_test_done;
  wire        o_valid;
  wire [63:0] o_result;

  af_mod_add_top #(
    .FIELD_BITS(64),
    .MODULUS(64'hFFFF_FFFF_0000_0001)
  ) u_core (
    .i_clk(i_clk),
    .i_rst_n(i_rst_n),
    .i_valid(i_valid),
    .o_ready(o_ready),
    .i_a(i_a),
    .i_b(i_b),
    .o_valid(o_valid),
    .i_ready(1'b1),
    .o_result(o_result)
  );

  always @(posedge i_clk or negedge i_rst_n) begin
    if (!i_rst_n) begin
      heartbeat_cnt <= '0;
      i_a <= 64'h0;
      i_b <= 64'h0;
      i_valid <= 1'b0;
      self_test_done <= 1'b0;
      led <= 2'b00;
    end else begin
      heartbeat_cnt <= heartbeat_cnt + 1'b1;
      if (!self_test_done) begin
        i_a <= 64'h1;
        i_b <= 64'h1;
        i_valid <= 1'b1;
        if (o_valid) begin
          self_test_done <= 1'b1;
          i_valid <= 1'b0;
          led <= (o_result == 64'h2) ? 2'b11 : 2'b01;
        end
      end else begin
        i_valid <= 1'b0;
        led <= {heartbeat_cnt[HEARTBEAT_DIV-1], heartbeat_cnt[HEARTBEAT_DIV-2]};
      end
    end
  end
endmodule
"#;
    if !Path::new(&format!("{dir}/top/af_board_top.sv")).exists()
        && !Path::new(&format!("{dir}/top/af_board_top.v")).exists()
    {
        fs::write(format!("{dir}/top/af_board_top.sv"), top).map_err(|e| e.to_string())?;
    }

    if !Path::new(&format!("{build_dir}/build.tcl")).exists() {
        fs::write(
            format!("{build_dir}/build.tcl"),
            "# Draft vendor build script.\n# Replace with a board- and toolchain-specific flow.\nset_global_assignment -name TOP_LEVEL_ENTITY af_board_top\n",
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
