//! Sigil project configuration and layout
//!
//! Handles detection and loading of sigil.json project configuration.
//! `src/` and `tests/` are canonical project directories; `sigil.json`
//! currently exists to mark the project root and optionally override the
//! generated output directory.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Effective project layout used by the compiler.
///
/// `src/` and `tests/` are fixed by the compiler. Only `out` is configurable
/// through `sigil.json`.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectLayout {
    pub src: String,
    pub tests: String,
    pub out: String,
}

fn default_src() -> String {
    "src".to_string()
}

fn default_tests() -> String {
    "tests".to_string()
}

fn default_out() -> String {
    ".local".to_string()
}

impl ProjectLayout {
    fn canonical(out: String) -> Self {
        Self {
            src: default_src(),
            tests: default_tests(),
            out,
        }
    }
}

impl Default for ProjectLayout {
    fn default() -> Self {
        Self::canonical(default_out())
    }
}

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub root: PathBuf,
    pub layout: ProjectLayout,
}

#[derive(Debug, Deserialize)]
struct RawProjectLayout {
    #[serde(default = "default_out")]
    out: String,
}

#[derive(Debug, Deserialize)]
struct RawProjectConfig {
    #[serde(default)]
    layout: Option<RawProjectLayout>,
}

/// Find the Sigil project root by searching for sigil.json
pub fn find_project_root(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path.to_path_buf();

    if current.is_file() {
        current = current.parent()?.to_path_buf();
    }

    loop {
        let config_path = current.join("sigil.json");
        if config_path.exists() {
            return Some(current);
        }

        current = current.parent()?.to_path_buf();
    }
}

/// Get Sigil project configuration
pub fn get_project_config(start_path: &Path) -> Option<ProjectConfig> {
    let root = find_project_root(start_path)?;
    let config_path = root.join("sigil.json");

    let raw_config: RawProjectConfig = serde_json::from_str(&fs::read_to_string(config_path).ok()?).ok()?;
    let out = raw_config
        .layout
        .map(|layout| layout.out)
        .unwrap_or_else(default_out);

    Some(ProjectConfig {
        root,
        layout: ProjectLayout::canonical(out),
    })
}

#[cfg(test)]
mod tests {
    use super::{ProjectLayout, RawProjectConfig};

    #[test]
    fn empty_config_uses_canonical_layout_defaults() {
        let raw: RawProjectConfig = serde_json::from_str("{}").unwrap();
        let out = raw.layout.map(|layout| layout.out).unwrap_or_else(super::default_out);
        let layout = ProjectLayout::canonical(out);

        assert_eq!(layout.src, "src");
        assert_eq!(layout.tests, "tests");
        assert_eq!(layout.out, ".local");
    }

    #[test]
    fn legacy_src_and_tests_entries_are_ignored() {
        let raw: RawProjectConfig = serde_json::from_str(
            r#"{
  "name": "demo",
  "version": "0.1.0",
  "layout": {
    "src": "custom-src",
    "tests": "custom-tests",
    "out": "build"
  }
}"#,
        )
        .unwrap();

        let layout = ProjectLayout::canonical(raw.layout.unwrap().out);
        assert_eq!(layout.src, "src");
        assert_eq!(layout.tests, "tests");
        assert_eq!(layout.out, "build");
    }
}
