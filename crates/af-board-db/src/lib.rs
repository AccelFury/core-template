// SPDX-License-Identifier: AGPL-3.0-or-later
//! Board registry loading helpers used by xtask and tooling.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RegistryBoardsFile {
    boards: Boards,
}

#[derive(Debug, Deserialize)]
struct RegistryFamiliesFile {
    families: Families,
}

#[derive(Debug, Deserialize)]
struct RegistryToolchainsFile {
    toolchains: Toolchains,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolchainEntry {
    pub id: String,
    pub display_name: String,
    pub command: String,
    pub constraint_formats: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FamilyEntry {
    pub vendor: String,
    pub family: String,
    pub resources: FamilyResources,
    pub supported_constraint_formats: Vec<String>,
    pub suggested_toolchains: Vec<String>,
    pub portability_notes: PortabilityNotes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FamilyResources {
    pub lut_kind: String,
    pub has_carry_chain: bool,
    pub has_dsp: bool,
    pub dsp_widths: Vec<u32>,
    pub bram_kbits: u32,
    pub distributed_ram: bool,
    pub has_hard_cpu: bool,
    pub has_serdes: bool,
    pub has_pcie: bool,
    pub has_pll: bool,
    pub has_external_memory_on_common_boards: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortabilityNotes {
    pub sv_support_level: String,
    pub inferred_ram_behavior: String,
    pub inferred_dsp_behavior: String,
    pub reset_preference: String,
    pub initial_block_synthesis: String,
    pub bram_read_during_write: String,
    pub pll_wrapper_requirements: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardEntry {
    pub board_id: String,
    pub display_name: String,
    pub vendor: String,
    pub fpga_family: String,
    pub fpga_part_if_known_or_template: String,
    pub logic_size_class: String,
    pub dsp_class: String,
    pub memory_class: String,
    pub high_speed_io_class: String,
    pub default_toolchain: String,
    pub alternative_toolchains: Vec<String>,
    pub constraint_format: String,
    pub board_dir: String,
    pub exact_pinout_status: String,
    pub safe_for_beginner: bool,
    pub suggested_ip_classes: Vec<String>,
    pub excluded_ip_classes: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpCategories {
    pub categories: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CategoriesFile {
    categories: Vec<String>,
}

pub type Toolchains = Vec<ToolchainEntry>;
pub type Families = Vec<FamilyEntry>;
pub type Boards = Vec<BoardEntry>;

#[derive(Debug)]
pub enum RegistryFile {
    Board(Boards),
    Family(Families),
    Toolchain(Toolchains),
}

pub fn load_registry<T>(path: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let txt = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    serde_json::from_str(&txt).map_err(|e| format!("parse {path}: {e}"))
}

pub fn assert_file_exists(path: &str) -> Result<(), String> {
    if Path::new(path).exists() {
        Ok(())
    } else {
        Err(format!("missing required file: {path}"))
    }
}

pub fn find_board<'a>(boards: &'a Boards, board_id: &str) -> Option<&'a BoardEntry> {
    boards.iter().find(|b| b.board_id == board_id)
}

pub fn load_boards(path: &str) -> Result<Boards, String> {
    let txt = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let manifest: RegistryBoardsFile =
        serde_json::from_str(&txt).map_err(|e| format!("parse {path}: {e}"))?;
    Ok(manifest.boards)
}

pub fn load_families(path: &str) -> Result<Families, String> {
    let txt = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let manifest: RegistryFamiliesFile =
        serde_json::from_str(&txt).map_err(|e| format!("parse {path}: {e}"))?;
    Ok(manifest.families)
}

pub fn load_toolchains(path: &str) -> Result<Toolchains, String> {
    let txt = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let manifest: RegistryToolchainsFile =
        serde_json::from_str(&txt).map_err(|e| format!("parse {path}: {e}"))?;
    Ok(manifest.toolchains)
}

pub fn valid_constraint_format(formats: &[String], value: &str) -> bool {
    formats.iter().any(|item| item == value)
}

pub fn categories() -> Vec<String> {
    let txt = fs::read_to_string("registries/ip_categories.json").unwrap_or_else(|_| {
        "{\"categories\":[\"field_arithmetic\",\"stream_infra\",\"dsp\"]}".to_string()
    });
    let parsed: CategoriesFile = serde_json::from_str(&txt).unwrap_or_else(|_| CategoriesFile {
        categories: vec![
            "field_arithmetic".to_string(),
            "stream_infra".to_string(),
            "dsp".to_string(),
        ],
    });
    parsed.categories
}

pub fn assert_category(category: &str) -> Result<(), String> {
    if categories().iter().any(|item| item == category) {
        Ok(())
    } else {
        Err(format!("unsupported category: {category}"))
    }
}
