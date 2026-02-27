//! Command implementations for CLI

use sigil_codegen::{CodegenOptions, TypeScriptGenerator};
use sigil_lexer::Lexer;
use sigil_parser::Parser;
use sigil_typechecker::{type_check, TypeError, TypeCheckOptions};
use sigil_validator::{validate_canonical_form, validate_surface_form, ValidationError};
use std::fs;
use std::path::Path;
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
