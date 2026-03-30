use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap()
        .to_path_buf()
}

fn sigil_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sigil"))
}

fn temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = repo_root().join("target").join(format!(
        "sigil-cli-compile-{label}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_program(dir: &Path, name: &str, source: &str) -> PathBuf {
    let file = dir.join(name);
    fs::write(&file, source).unwrap();
    file
}

fn parse_json(text: &[u8]) -> Value {
    serde_json::from_slice(text).unwrap()
}

#[test]
fn compile_emits_root_span_map_for_single_file() {
    let dir = temp_dir("single");
    let file = write_program(&dir, "main.sigil", "λmain()=>Int=1+2\n");

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("compile")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let json = parse_json(&output.stdout);
    let span_map_path = PathBuf::from(
        json["data"]["outputs"]["rootSpanMap"]
            .as_str()
            .expect("rootSpanMap path"),
    );
    assert!(span_map_path.exists());
    assert!(json["data"]["outputs"]["allModules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|module| module["spanMapFile"] == span_map_path.to_string_lossy().to_string()));

    let span_map: Value = serde_json::from_str(&fs::read_to_string(&span_map_path).unwrap()).unwrap();
    assert_eq!(span_map["formatVersion"], 1);
    assert_eq!(span_map["sourceFile"], file.to_string_lossy().to_string());
    assert_eq!(span_map["outputFile"], json["data"]["outputs"]["rootTs"]);
    assert!(span_map["spans"].as_array().unwrap().len() >= 2);
}

#[test]
fn compile_directory_reports_root_span_map_per_entry() {
    let dir = temp_dir("directory");
    write_program(&dir, "main.sigil", "λmain()=>Int=1\n");

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("compile")
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let json = parse_json(&output.stdout);
    assert_eq!(json["data"]["files"].as_array().unwrap().len(), 1);
    let span_map_path = PathBuf::from(
        json["data"]["files"][0]["rootSpanMap"]
            .as_str()
            .expect("rootSpanMap path"),
    );
    assert!(span_map_path.exists());
}
