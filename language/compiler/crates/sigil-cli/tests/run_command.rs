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
    let dir = repo_root().join(".local").join(format!(
        "sigil-cli-run-{label}-{}-{unique}",
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
fn run_streams_raw_stdout_by_default() {
    let dir = temp_dir("raw-success");
    let file = write_program(
        &dir,
        "main.sigil",
        "e console:{log:λ(String)=>!Log Unit}\n\nλmain()=>!Log Unit=console.log(\"raw ok\")\n",
    );

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "raw ok\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn run_json_preserves_success_envelope() {
    let dir = temp_dir("json-success");
    let file = write_program(
        &dir,
        "main.sigil",
        "e console:{log:λ(String)=>!Log Unit}\n\nλmain()=>!Log Unit=console.log(\"json ok\")\n",
    );

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg("--json")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let json = parse_json(&output.stdout);
    assert_eq!(json["command"], "sigilc run");
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["runtime"]["stdout"], "json ok\n");
    assert_eq!(json["data"]["runtime"]["stderr"], "");
}

#[test]
fn run_emits_json_error_on_compile_failure() {
    let dir = temp_dir("compile-failure");
    let file = write_program(&dir, "broken.sigil", "λmain()=>Unit={\n");

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("\nError: "));
    let json = parse_json(output.stderr.trim_ascii());
    assert_eq!(json["command"], "sigilc run");
    assert_eq!(json["ok"], false);
    assert_eq!(json["phase"], "parser");
}

#[test]
fn run_keeps_streamed_output_and_appends_json_on_child_failure() {
    let dir = temp_dir("runtime-failure");
    let file = write_program(
        &dir,
        "main.sigil",
        "e console:{log:λ(String)=>!Log Unit}\n\
\n\
e process:{exit:λ(Int)=>Unit}\n\
\n\
λmain()=>!Log Unit={\n  l _=(console.log(\"before exit\"):Unit);\n  process.exit(1)\n}\n",
    );

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "before exit\n");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("\nError: Process exited with code"));
    let json = parse_json(output.stderr.trim_ascii());
    assert_eq!(json["command"], "sigilc run");
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "SIGIL-RUNTIME-CHILD-EXIT");
    assert_eq!(json["error"]["details"]["stdout"], "before exit\n");
}

#[test]
fn run_json_reports_runtime_failures_without_extra_text() {
    let dir = temp_dir("json-runtime-failure");
    let file = write_program(
        &dir,
        "main.sigil",
        "e console:{log:λ(String)=>!Log Unit}\n\
\n\
e process:{exit:λ(Int)=>Unit}\n\
\n\
λmain()=>!Log Unit={\n  l _=(console.log(\"json before exit\"):Unit);\n  process.exit(1)\n}\n",
    );

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg("--json")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());

    let json = parse_json(&output.stdout);
    assert_eq!(json["command"], "sigilc run");
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "SIGIL-RUNTIME-CHILD-EXIT");
    assert_eq!(json["error"]["details"]["stdout"], "json before exit\n");
}
