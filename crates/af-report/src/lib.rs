// SPDX-License-Identifier: AGPL-3.0-or-later
//! Lightweight report helper for CI artifacts.

use serde::Serialize;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
pub struct Artifact {
    pub timestamp: String,
    pub label: String,
    pub content: String,
}

pub fn collect_report(label: &str, content: &str) -> Result<Artifact, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    Ok(Artifact {
        timestamp: now.to_string(),
        label: label.to_string(),
        content: content.to_string(),
    })
}

pub fn save_report(artifact: &Artifact, dir: &str) -> Result<String, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = format!("{}/{}_{}.json", dir, artifact.label, artifact.timestamp);
    let payload = serde_json::to_string_pretty(artifact).map_err(|e| e.to_string())?;
    fs::write(&path, payload).map_err(|e| e.to_string())?;
    Ok(path)
}
