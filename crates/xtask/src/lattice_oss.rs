// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn synthesize(board_dir: &str) -> Result<String, String> {
    let board_file = format!("{board_dir}/README.md");
    if !std::path::Path::new(&board_file).exists() {
        return Err(format!(
            "unsupported board directory for lattice oss flow: {board_dir}"
        ));
    }
    let board = std::path::Path::new(board_dir)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("board");

    let dir = PathBuf::from("reports/resources");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let mut path = dir;
    path.push(format!("lattice_oss_{board}_{ts}.txt"));
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    writeln!(
        f,
        "lattice OSS synthesis placeholder for {board}\nRun yosys + nextpnr flow and attach timing/resource reports."
    )
    .map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}
