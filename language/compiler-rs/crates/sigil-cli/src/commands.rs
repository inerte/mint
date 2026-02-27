//! Command implementations for CLI

use sigil_codegen::{CodegenOptions, TypeScriptGenerator};
use sigil_lexer::Lexer;
use sigil_parser::Parser;
use sigil_typechecker::{type_check, TypeError, TypeCheckOptions};
use sigil_validator::{validate_canonical_form, validate_surface_form, ValidationError};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Lexer error: {0}")]
    Lexer(String),

    #[error("Parser error: {0}")]
    Parser(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Type error: {0}")]
    Type(String),

    #[error("Codegen error: {0}")]
    Codegen(String),

    #[error("Runtime error: {0}")]
    Runtime(String),
}

/// Lex command: tokenize a Sigil file
pub fn lex_command(file: &Path, human: bool) -> Result<(), CliError> {
    let source = fs::read_to_string(file)?;
    let filename = file.to_string_lossy().to_string();

    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| CliError::Lexer(format!("{:?}", e)))?;

    if human {
        println!("sigilc lex OK phase=lexer");
        for token in &tokens {
            println!(
                "{}({}) at {}:{}",
                format!("{:?}", token.token_type),
                &token.value,
                token.location.start.line,
                token.location.start.column
            );
        }
    } else {
        // JSON output
        let output = serde_json::json!({
            "formatVersion": 1,
            "command": "sigilc lex",
            "ok": true,
            "phase": "lexer",
            "data": {
                "file": filename,
                "summary": {
                    "tokens": tokens.len()
                },
                "tokens": tokens.iter().map(|t| {
                    serde_json::json!({
                        "type": format!("{:?}", t.token_type),
                        "lexeme": &t.value,
                        "start": {
                            "line": t.location.start.line,
                            "column": t.location.start.column,
                            "offset": t.location.start.offset
                        }
                    })
                }).collect::<Vec<_>>()
            }
        });
        println!("{}", serde_json::to_string(&output).unwrap());
    }

    Ok(())
}

/// Parse command: parse a Sigil file to AST
pub fn parse_command(file: &Path, human: bool) -> Result<(), CliError> {
    let source = fs::read_to_string(file)?;
    let filename = file.to_string_lossy().to_string();

    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| CliError::Lexer(format!("{:?}", e)))?;

    // Parse
    let mut parser = Parser::new(tokens, &filename);
    let ast = parser.parse().map_err(|e| CliError::Parser(format!("{:?}", e)))?;

    // Validate surface form
    validate_surface_form(&ast).map_err(|e: Vec<ValidationError>| {
        CliError::Validation(format!("{} validation errors", e.len()))
    })?;

    if human {
        println!("sigilc parse OK phase=parser");
        println!("{:#?}", ast);
    } else {
        // JSON output
        let output = serde_json::json!({
            "formatVersion": 1,
            "command": "sigilc parse",
            "ok": true,
            "phase": "parser",
            "data": {
                "file": filename,
                "summary": {
                    "declarations": ast.declarations.len()
                },
                "ast": format!("{:#?}", ast) // Simplified for now
            }
        });
        println!("{}", serde_json::to_string(&output).unwrap());
    }

    Ok(())
}

/// Compile command: compile a Sigil file to TypeScript
pub fn compile_command(
    file: &Path,
    output: Option<&Path>,
    show_types: bool,
    human: bool,
) -> Result<(), CliError> {
    let source = fs::read_to_string(file)?;
    let filename = file.to_string_lossy().to_string();

    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| CliError::Lexer(format!("{:?}", e)))?;

    // Parse
    let mut parser = Parser::new(tokens, &filename);
    let ast = parser.parse().map_err(|e| CliError::Parser(format!("{:?}", e)))?;

    // Validate surface form
    validate_surface_form(&ast).map_err(|errors: Vec<ValidationError>| {
        CliError::Validation(format!("{} validation errors", errors.len()))
    })?;

    // Validate canonical form
    validate_canonical_form(&ast).map_err(|errors: Vec<ValidationError>| {
        CliError::Validation(format!("{} validation errors", errors.len()))
    })?;

    // Type check
    let _inferred_types = type_check(&ast, &source, None)
        .map_err(|error: TypeError| CliError::Type(format!("Type error: {:?}", error)))?;

    // Generate TypeScript
    let codegen_options = CodegenOptions {
        source_file: Some(filename.clone()),
        output_file: output.map(|p| p.to_string_lossy().to_string()),
        project_root: None,
    };
    let mut codegen = TypeScriptGenerator::new(codegen_options);
    let ts_code = codegen
        .generate(&ast)
        .map_err(|e| CliError::Codegen(format!("{:?}", e)))?;

    // Determine output file
    let output_file_owned: std::path::PathBuf;
    let output_file = if let Some(out) = output {
        out
    } else {
        let input_str = file.to_string_lossy();
        output_file_owned = if input_str.ends_with(".sigil") {
            std::path::PathBuf::from(format!(
                ".local/{}",
                input_str.replace(".sigil", ".ts")
            ))
        } else {
            std::path::PathBuf::from(format!("{}.ts", input_str))
        };
        &output_file_owned
    };

    // Create output directory if needed
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write output file
    fs::write(output_file, ts_code)?;

    if human {
        println!("sigilc compile OK phase=codegen");
        println!("Output: {}", output_file.display());
    } else {
        // JSON output
        let output_json = serde_json::json!({
            "formatVersion": 1,
            "command": "sigilc compile",
            "ok": true,
            "phase": "codegen",
            "data": {
                "input": filename,
                "outputs": {
                    "rootTs": output_file.to_string_lossy()
                },
                "typecheck": {
                    "ok": true,
                    "inferred": if show_types { vec![] as Vec<serde_json::Value> } else { vec![] }
                }
            }
        });
        println!("{}", serde_json::to_string(&output_json).unwrap());
    }

    Ok(())
}

/// Run command: compile and execute a Sigil file
pub fn run_command(file: &Path, human: bool) -> Result<(), CliError> {
    let source = fs::read_to_string(file)?;
    let filename = file.to_string_lossy().to_string();

    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| CliError::Lexer(format!("{:?}", e)))?;

    // Parse
    let mut parser = Parser::new(tokens, &filename);
    let ast = parser.parse().map_err(|e| CliError::Parser(format!("{:?}", e)))?;

    // Validate surface form
    validate_surface_form(&ast).map_err(|errors: Vec<ValidationError>| {
        CliError::Validation(format!("{} validation errors", errors.len()))
    })?;

    // Validate canonical form
    validate_canonical_form(&ast).map_err(|errors: Vec<ValidationError>| {
        CliError::Validation(format!("{} validation errors", errors.len()))
    })?;

    // Type check
    let _inferred_types = type_check(&ast, &source, None)
        .map_err(|error: TypeError| CliError::Type(format!("Type error: {:?}", error)))?;

    // Generate TypeScript
    let input_str = file.to_string_lossy();
    let output_file_path = if input_str.ends_with(".sigil") {
        PathBuf::from(format!(".local/{}", input_str.replace(".sigil", ".ts")))
    } else {
        PathBuf::from(format!("{}.ts", input_str))
    };

    let codegen_options = CodegenOptions {
        source_file: Some(filename.clone()),
        output_file: Some(output_file_path.to_string_lossy().to_string()),
        project_root: None,
    };
    let mut codegen = TypeScriptGenerator::new(codegen_options);
    let ts_code = codegen
        .generate(&ast)
        .map_err(|e| CliError::Codegen(format!("{:?}", e)))?;

    // Create output directory if needed
    if let Some(parent) = output_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write output file
    fs::write(&output_file_path, ts_code)?;

    // Create runner file
    let runner_path = output_file_path.with_extension("run.ts");
    let module_name = output_file_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let runner_code = format!(
        r#"import {{ main }} from './{module_name}';

if (typeof main !== 'function') {{
  console.error('Error: No main() function found in {filename}');
  console.error('Add a main() function to make this program runnable.');
  process.exit(1);
}}

// Call main and handle the result (all Sigil functions are async)
const result = await main();

// If main returns a value (not Unit/undefined), show it
if (result !== undefined) {{
  console.log(result);
}}
"#
    );

    fs::write(&runner_path, runner_code)?;

    // Execute the runner (use absolute path to avoid path resolution issues)
    let abs_runner_path = std::fs::canonicalize(&runner_path)?;
    let start_time = Instant::now();
    let output = Command::new("pnpm")
        .args(&["exec", "node", "--import", "tsx"])
        .arg(&abs_runner_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CliError::Runtime("pnpm not found. Please install pnpm to run Sigil programs.".to_string())
            } else {
                CliError::Runtime(format!("Failed to execute: {}", e))
            }
        })?;

    let duration_ms = start_time.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    if exit_code != 0 {
        if human {
            eprintln!("{}", stderr);
            eprintln!("sigilc run FAIL (exit code: {})", exit_code);
        } else {
            let output_json = serde_json::json!({
                "formatVersion": 1,
                "command": "sigilc run",
                "ok": false,
                "phase": "runtime",
                "error": {
                    "code": "SIGIL-RUNTIME-CHILD-EXIT",
                    "phase": "runtime",
                    "message": format!("child process exited with nonzero status: {}", exit_code),
                    "details": {
                        "exitCode": exit_code,
                        "stdout": stdout.to_string(),
                        "stderr": stderr.to_string()
                    }
                }
            });
            println!("{}", serde_json::to_string(&output_json).unwrap());
        }
        return Err(CliError::Runtime(format!("Process exited with code {}", exit_code)));
    }

    if human {
        print!("{}", stdout);
        eprint!("{}", stderr);
        println!("sigilc run OK phase=runtime");
    } else {
        let output_json = serde_json::json!({
            "formatVersion": 1,
            "command": "sigilc run",
            "ok": true,
            "phase": "runtime",
            "data": {
                "compile": {
                    "input": filename,
                    "output": output_file_path.to_string_lossy(),
                    "runnerFile": runner_path.to_string_lossy()
                },
                "runtime": {
                    "engine": "node+tsx",
                    "exitCode": exit_code,
                    "durationMs": duration_ms,
                    "stdout": stdout.to_string(),
                    "stderr": stderr.to_string()
                }
            }
        });
        println!("{}", serde_json::to_string(&output_json).unwrap());
    }

    Ok(())
}
