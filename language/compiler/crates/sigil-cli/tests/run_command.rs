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
    assert!(PathBuf::from(
        json["data"]["compile"]["spanMapFile"]
            .as_str()
            .expect("spanMapFile path")
    )
    .exists());
}

#[test]
fn run_trace_requires_json() {
    let dir = temp_dir("trace-requires-json");
    let file = write_program(&dir, "main.sigil", "λmain()=>Int=1\n");

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg("--trace")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let json = parse_json(output.stderr.trim_ascii());
    assert_eq!(json["command"], "sigilc run");
    assert_eq!(json["ok"], false);
    assert_eq!(json["phase"], "cli");
    assert_eq!(json["error"]["code"], "SIGIL-CLI-USAGE");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--json"));
}

#[test]
fn run_json_trace_success_includes_call_branch_and_effect_events() {
    let dir = temp_dir("trace-success");
    let file = write_program(
        &dir,
        "main.sigil",
        "λhelper(flag:Bool)=>!Random Int match flag{\n  true=>§random.intBetween(1,1)|\n  false=>0\n}\n\nλmain()=>!Random Int=helper(true)\n",
    );

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg("--json")
        .arg("--trace")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let json = parse_json(&output.stdout);
    assert_eq!(json["command"], "sigilc run");
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["trace"]["enabled"], true);
    let events = json["data"]["trace"]["events"]
        .as_array()
        .expect("trace events array");
    assert!(events.iter().any(|event| event["kind"] == "call"));
    assert!(events.iter().any(|event| event["kind"] == "branch_match"));
    assert!(events.iter().any(|event| event["kind"] == "effect_call"));
    assert!(events.iter().any(|event| event["kind"] == "effect_result"));
    assert!(events.iter().any(|event| {
        event["kind"] == "effect_call"
            && event["effectFamily"] == "random"
            && event["operation"] == "intBetween"
    }));
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
    assert_eq!(
        json["error"]["details"]["runtime"]["stdout"],
        "before exit\n"
    );
    assert_eq!(
        json["error"]["details"]["compile"]["input"],
        file.to_string_lossy().to_string()
    );
    assert!(json["error"]["details"]["exception"].is_null());
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
    assert_eq!(
        json["error"]["details"]["runtime"]["stdout"],
        "json before exit\n"
    );
    assert_eq!(
        json["error"]["details"]["compile"]["input"],
        file.to_string_lossy().to_string()
    );
    assert!(json["error"]["details"]["exception"].is_null());
}

#[test]
fn run_json_trace_preserves_child_exit_failures_with_trace_details() {
    let dir = temp_dir("json-trace-child-exit");
    let file = write_program(
        &dir,
        "main.sigil",
        "λmain()=>!Process Unit=§process.exit(1)\n",
    );

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg("--json")
        .arg("--trace")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());

    let json = parse_json(&output.stdout);
    assert_eq!(json["error"]["code"], "SIGIL-RUNTIME-CHILD-EXIT");
    assert_eq!(json["error"]["details"]["trace"]["enabled"], true);
    assert!(json["error"]["details"]["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["kind"] == "effect_call"));
    assert!(json["error"]["details"]["exception"].is_null());
}

#[test]
fn run_json_enriches_uncaught_runtime_exceptions() {
    let dir = temp_dir("json-runtime-exception");
    let file = write_program(
        &dir,
        "main.sigil",
        "e boom:{explode:λ()=>Unit}\n\nλmain()=>Unit=boom.explode()\n",
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
    assert_eq!(json["phase"], "runtime");
    assert_eq!(json["error"]["code"], "SIGIL-RUNTIME-UNCAUGHT-EXCEPTION");
    assert_eq!(
        json["error"]["location"]["file"],
        file.to_string_lossy().to_string()
    );
    assert_eq!(
        json["error"]["details"]["compile"]["input"],
        file.to_string_lossy().to_string()
    );
    assert!(PathBuf::from(
        json["error"]["details"]["compile"]["spanMapFile"]
            .as_str()
            .expect("spanMapFile path")
    )
    .exists());
    assert!(json["error"]["details"]["runtime"]["stderr"]
        .as_str()
        .unwrap()
        .contains("ReferenceError"));
    assert_eq!(
        json["error"]["details"]["exception"]["name"],
        "ReferenceError"
    );
    assert_eq!(
        json["error"]["details"]["exception"]["sigilFrame"]["label"],
        "main"
    );
    assert_eq!(
        json["error"]["details"]["exception"]["sigilFrame"]["kind"],
        "function_decl"
    );
    assert_eq!(
        json["error"]["details"]["exception"]["sigilFrame"]["file"],
        file.to_string_lossy().to_string()
    );
    assert!(
        json["error"]["details"]["exception"]["generatedFrame"]["file"]
            .as_str()
            .unwrap()
            .ends_with(".ts")
    );
    assert!(
        json["error"]["details"]["exception"]["sigilFrame"]["excerpt"]["text"]
            .as_str()
            .unwrap()
            .contains("λmain()=>Unit=boom.explode()")
    );
}

#[test]
fn run_json_trace_failure_includes_trace_details() {
    let dir = temp_dir("json-trace-runtime-exception");
    let file = write_program(
        &dir,
        "main.sigil",
        "e boom:{explode:λ()=>Int}\n\nλmain()=>Int=boom.explode()\n",
    );

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg("--json")
        .arg("--trace")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());

    let json = parse_json(&output.stdout);
    assert_eq!(json["error"]["code"], "SIGIL-RUNTIME-UNCAUGHT-EXCEPTION");
    assert_eq!(json["error"]["details"]["trace"]["enabled"], true);
    assert!(json["error"]["details"]["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["kind"] == "call"));
}

#[test]
fn run_json_enriches_import_time_runtime_exceptions() {
    let dir = temp_dir("json-import-runtime-exception");
    let file = write_program(
        &dir,
        "main.sigil",
        "e boom:{explode:λ()=>Int}\n\nc bad=(boom.explode():Int)\n\nλmain()=>Int=bad\n",
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
    assert_eq!(json["error"]["code"], "SIGIL-RUNTIME-UNCAUGHT-EXCEPTION");
    assert_eq!(
        json["error"]["location"]["file"],
        file.to_string_lossy().to_string()
    );
    assert_eq!(
        json["error"]["details"]["exception"]["sigilFrame"]["label"],
        "bad"
    );
    assert_eq!(
        json["error"]["details"]["exception"]["sigilFrame"]["kind"],
        "const_decl"
    );
    assert!(
        json["error"]["details"]["exception"]["sigilFrame"]["excerpt"]["text"]
            .as_str()
            .unwrap()
            .contains("c bad=(boom.explode():Int)")
    );
}

#[test]
fn run_json_trace_truncates_large_event_streams() {
    let dir = temp_dir("trace-truncation");
    let file = write_program(
        &dir,
        "main.sigil",
        "λloop(n:Int)=>Int match n=0{\n  true=>0|\n  false=>loop(n-1)\n}\n\nλmain()=>Int=loop(400)\n",
    );

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg("--json")
        .arg("--trace")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json = parse_json(&output.stdout);
    assert_eq!(json["data"]["trace"]["enabled"], true);
    assert_eq!(json["data"]["trace"]["truncated"], true);
    assert_eq!(json["data"]["trace"]["returnedEvents"], 256);
    assert!(
        json["data"]["trace"]["totalEvents"].as_u64().unwrap()
            > json["data"]["trace"]["returnedEvents"].as_u64().unwrap()
    );
    assert!(json["data"]["trace"]["droppedEvents"].as_u64().unwrap() > 0);
}

#[test]
fn run_json_preserves_topology_codes_for_bootstrap_failures() {
    let dir = temp_dir("json-topology-runtime-failure");
    let src_dir = dir.join("src");
    let config_dir = dir.join("config");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        dir.join("sigil.json"),
        "{\n  \"name\": \"topology-runtime-failure\",\n  \"version\": \"0.1.0\"\n}\n",
    )
    .unwrap();
    let file = write_program(&src_dir, "main.sigil", "λmain()=>Int=1\n");
    write_program(
        &src_dir,
        "topology.lib.sigil",
        "c local=(§topology.environment(\"local\"):§topology.Environment)\n",
    );
    fs::write(
        config_dir.join("staging.lib.sigil"),
        "e process\n\nc world=(†runtime.world(†clock.systemClock(),†fs.real(),[],†log.capture(),†process.real(),†random.seeded(1337),[],†timer.virtual()):†runtime.World)\n",
    )
    .unwrap();

    let output = Command::new(sigil_bin())
        .current_dir(repo_root())
        .arg("run")
        .arg("--json")
        .arg("--env")
        .arg("staging")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());

    let json = parse_json(&output.stdout);
    assert_eq!(json["command"], "sigilc run");
    assert_eq!(json["ok"], false);
    assert_eq!(json["phase"], "topology");
    assert_eq!(json["error"]["code"], "SIGIL-TOPO-ENV-NOT-FOUND");
    assert_eq!(
        json["error"]["details"]["compile"]["input"],
        file.to_string_lossy().to_string()
    );
    assert!(json["error"]["details"]["runtime"]["stderr"]
        .as_str()
        .unwrap()
        .contains("SIGIL-TOPO-ENV-NOT-FOUND"));
    assert_eq!(json["error"]["details"]["exception"]["name"], "Error");
    assert!(
        json["error"]["details"]["exception"]["generatedFrame"]["file"]
            .as_str()
            .unwrap()
            .ends_with(".run.ts")
    );
    assert!(json["error"]["location"].is_null());
    assert!(json["error"]["details"]["exception"]["sigilFrame"].is_null());
}
