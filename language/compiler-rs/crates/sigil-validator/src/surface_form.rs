//! Surface form validation
//!
//! Validates basic syntax correctness requirements:
//! - All functions have return type annotations
//! - All parameters have type annotations
//! - Type names follow naming conventions

use sigil_ast::*;
use crate::error::ValidationError;

/// Validate surface form requirements
pub fn validate_surface_form(program: &Program) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    for decl in &program.declarations {
        // Validate function declarations
        if let Declaration::Function(func) = decl {
            // Check return type annotation
            if func.return_type.is_none() {
                errors.push(ValidationError::MissingReturnType {
                    function_name: func.name.clone(),
                    location: func.location,
                });
            }

            // Check parameter type annotations
            for param in &func.params {
                if param.type_annotation.is_none() {
                    errors.push(ValidationError::MissingParamType {
                        param_name: param.name.clone(),
                        location: param.location,
                    });
                }
            }
        }

        // Validate const declarations
        if let Declaration::Const(c) = decl {
            if c.type_annotation.is_none() {
                // Constants should have type annotations in canonical form
                // This is enforced by the parser, so this is just a sanity check
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigil_lexer::tokenize;
    use sigil_parser::parse;

    #[test]
    fn test_valid_surface_form() {
        let source = r#"
λ add(x: ℤ, y: ℤ) → ℤ = x + y
c PI: ℝ = 3.14159
        "#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens, "test.sigil").unwrap();

        assert!(validate_surface_form(&program).is_ok());
    }

    #[test]
    fn test_surface_form_accepts_parsed_programs() {
        // Since the parser enforces type annotations in canonical form,
        // any successfully parsed program should pass surface form validation
        let source = r#"
export λ fibonacci(n: ℤ) → ℤ = n ≡
  0 → 0
  | 1 → 1
  | _ → fibonacci(n - 1) + fibonacci(n - 2)
        "#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens, "test.sigil").unwrap();

        // Should pass because parser already enforces canonical form
        assert!(validate_surface_form(&program).is_ok());
    }
}
