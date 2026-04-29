// SPDX-License-Identifier: AGPL-3.0-or-later
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const REQUIRED_RTL_HEADER: &str = "SPDX-License-Identifier: CERN-OHL-S-2.0";
const REQUIRED_TOOLS_HEADER: &str = "SPDX-License-Identifier: AGPL-3.0-or-later";

fn file_header(path: &Path) -> Result<bool, String> {
    let txt = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(match path.extension().and_then(|e| e.to_str()) {
        Some("sv") | Some("v") => txt.contains(REQUIRED_RTL_HEADER),
        Some("rs") | Some("ts") => txt.contains(REQUIRED_TOOLS_HEADER),
        Some("md") => true,
        _ => true,
    })
}

pub fn check() -> Result<Vec<PathBuf>, String> {
    let mut missing = Vec::new();
    for entry in WalkDir::new(".").into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        let rel = path.to_string_lossy();
        if rel.starts_with("./.git")
            || rel.starts_with(".git")
            || rel.starts_with("./target/")
            || rel.starts_with("target/")
        {
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext {
                "sv" | "v" | "rs" | "ts" | "md" if !file_header(&path)? => missing.push(path),
                _ => {}
            }
        }
    }
    Ok(missing)
}
