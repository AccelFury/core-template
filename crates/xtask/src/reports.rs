// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn write_report(tag: &str, payload: &str) -> Result<String, String> {
    fs::create_dir_all("reports").map_err(|e| e.to_string())?;
    fs::create_dir_all("reports/simulation").map_err(|e| e.to_string())?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    let path = format!("reports/simulation/{tag}_{:0>10}.log", ts.as_secs());
    fs::write(&path, payload).map_err(|e| e.to_string())?;
    Ok(path)
}
