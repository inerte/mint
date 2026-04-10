//! Sigil project configuration and layout
//!
//! Handles detection and loading of sigil.json project configuration.
//! `src/` and `tests/` are canonical project directories; `sigil.json`
//! marks the project root and declares required project metadata.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sigil_diagnostics::codes;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Effective project layout used by the compiler.
///
/// `src/`, `tests/`, and `.local/` are fixed by the compiler.
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

#[derive(Debug, Error)]
pub enum ProjectConfigError {
    #[error("failed to read sigil.json at {}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid sigil.json at {}: {message}", path.display())]
    Invalid { path: PathBuf, message: String },

    #[error(
        "{}: project has executable source under src/ but is missing src/main.sigil",
        codes::cli::PROJECT_MAIN_REQUIRED
    )]
    MissingProjectMain {
        root: PathBuf,
        main_path: PathBuf,
        executable_sources: Vec<PathBuf>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProjectConfig {
    name: String,
    version: String,
}

fn invalid_config(path: PathBuf, message: impl Into<String>) -> ProjectConfigError {
    ProjectConfigError::Invalid {
        path,
        message: message.into(),
    }
}

impl ProjectConfigError {
    pub fn code(&self) -> &'static str {
        match self {
            ProjectConfigError::MissingProjectMain { .. } => codes::cli::PROJECT_MAIN_REQUIRED,
            _ => codes::cli::UNEXPECTED,
        }
    }

    pub fn details(&self) -> Value {
        match self {
            ProjectConfigError::Io { path, .. } => {
                json!({
                    "path": path.to_string_lossy()
                })
            }
            ProjectConfigError::Invalid { path, message } => {
                json!({
                    "path": path.to_string_lossy(),
                    "message": message
                })
            }
            ProjectConfigError::MissingProjectMain {
                root,
                main_path,
                executable_sources,
            } => {
                json!({
                    "projectRoot": root.to_string_lossy(),
                    "missingPath": main_path.to_string_lossy(),
                    "executableSources": executable_sources
                        .iter()
                        .map(|path| path.to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                })
            }
        }
    }
}

fn is_project_executable_source(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".sigil") && !name.ends_with(".lib.sigil"))
}

fn collect_project_executable_sources(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), ProjectConfigError> {
    if !dir.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(dir)
        .map_err(|source| ProjectConfigError::Io {
            path: dir.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, std::io::Error>>()
        .map_err(|source| ProjectConfigError::Io {
            path: dir.to_path_buf(),
            source,
        })?
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();

    for path in entries {
        if path.is_dir() {
            collect_project_executable_sources(&path, files)?;
        } else if is_project_executable_source(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn parse_project_config(
    config_path: PathBuf,
    root: PathBuf,
    source: &str,
) -> Result<ProjectConfig, ProjectConfigError> {
    let raw: RawProjectConfig = serde_json::from_str(source)
        .map_err(|err| invalid_config(config_path.clone(), err.to_string()))?;
    let name = raw.name.trim();
    let version = raw.version.trim();

    if name.is_empty() {
        return Err(invalid_config(
            config_path,
            "field `name` must be a non-empty string",
        ));
    }

    if version.is_empty() {
        return Err(invalid_config(
            config_path,
            "field `version` must be a non-empty string",
        ));
    }

    Ok(ProjectConfig {
        root,
        layout: ProjectLayout::default(),
    })
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
pub fn get_project_config(start_path: &Path) -> Result<Option<ProjectConfig>, ProjectConfigError> {
    let Some(root) = find_project_root(start_path) else {
        return Ok(None);
    };
    let config_path = root.join("sigil.json");
    let source = fs::read_to_string(&config_path).map_err(|source| ProjectConfigError::Io {
        path: config_path.clone(),
        source,
    })?;

    parse_project_config(config_path, root, &source).map(Some)
}

pub fn validate_project_default_entrypoint(
    project: &ProjectConfig,
) -> Result<(), ProjectConfigError> {
    let src_dir = project.root.join(&project.layout.src);
    let main_path = src_dir.join("main.sigil");
    let mut executable_sources = Vec::new();
    collect_project_executable_sources(&src_dir, &mut executable_sources)?;

    if !executable_sources.is_empty() && !main_path.is_file() {
        return Err(ProjectConfigError::MissingProjectMain {
            root: project.root.clone(),
            main_path,
            executable_sources,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_project_config, validate_project_default_entrypoint, ProjectConfigError};
    use sigil_diagnostics::codes;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn parse(source: &str) -> Result<super::ProjectConfig, ProjectConfigError> {
        parse_project_config(
            PathBuf::from("/tmp/demo/sigil.json"),
            PathBuf::from("/tmp/demo"),
            source,
        )
    }

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "sigil-project-config-{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(dir: &Path, relative: &str, source: &str) {
        let file = dir.join(relative);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(file, source).unwrap();
    }

    fn parsed_project(root: &Path) -> super::ProjectConfig {
        parse_project_config(
            root.join("sigil.json"),
            root.to_path_buf(),
            r#"{"name":"demo","version":"0.1.0"}"#,
        )
        .unwrap()
    }

    #[test]
    fn valid_config_requires_name_and_version_and_uses_canonical_layout() {
        let config = parse(r#"{"name":"demo","version":"0.1.0"}"#).unwrap();

        assert_eq!(config.layout.src, "src");
        assert_eq!(config.layout.tests, "tests");
        assert_eq!(config.layout.out, ".local");
    }

    #[test]
    fn config_rejects_unknown_fields() {
        let err = parse(
            r#"{
  "name":"demo",
  "version":"0.1.0",
  "extra":true
}"#,
        )
        .unwrap_err();

        assert!(err.to_string().contains("unknown field `extra`"));
    }

    #[test]
    fn config_rejects_missing_name() {
        let err = parse(r#"{"version":"0.1.0"}"#).unwrap_err();

        assert!(err.to_string().contains("missing field `name`"));
    }

    #[test]
    fn config_rejects_empty_version() {
        let err = parse(r#"{"name":"demo","version":"   "}"#).unwrap_err();

        assert!(err
            .to_string()
            .contains("field `version` must be a non-empty string"));
    }

    #[test]
    fn library_only_projects_can_omit_src_main() {
        let dir = temp_dir("library-only");
        write_file(&dir, "sigil.json", r#"{"name":"demo","version":"0.1.0"}"#);
        write_file(&dir, "src/helper.lib.sigil", "λdouble(n:Int)=>Int=n+n\n");

        let project = parsed_project(&dir);

        validate_project_default_entrypoint(&project).unwrap();
    }

    #[test]
    fn executable_projects_require_src_main() {
        let dir = temp_dir("missing-main");
        write_file(&dir, "sigil.json", r#"{"name":"demo","version":"0.1.0"}"#);
        write_file(&dir, "src/demo.sigil", "λmain()=>Int=1\n");

        let project = parsed_project(&dir);
        let err = validate_project_default_entrypoint(&project).unwrap_err();

        assert_eq!(err.code(), codes::cli::PROJECT_MAIN_REQUIRED);
        assert!(err.to_string().contains("missing src/main.sigil"));
    }

    #[test]
    fn executable_projects_with_src_main_are_valid() {
        let dir = temp_dir("has-main");
        write_file(&dir, "sigil.json", r#"{"name":"demo","version":"0.1.0"}"#);
        write_file(&dir, "src/main.sigil", "λmain()=>String=\"demo\"\n");
        write_file(&dir, "src/demo.sigil", "λmain()=>Int=1\n");

        let project = parsed_project(&dir);

        validate_project_default_entrypoint(&project).unwrap();
    }
}
