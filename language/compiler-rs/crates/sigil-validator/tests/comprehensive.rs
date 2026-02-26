//! Comprehensive validator tests

use sigil_lexer::tokenize;
use sigil_parser::parse;
use sigil_validator::{validate_canonical_form, validate_surface_form, ValidationError};

// ============================================================================
// DUPLICATE DECLARATION TESTS
// ============================================================================

#[test]
fn test_duplicate_types() {
    let source = "t Foo=Bar\nt Foo=Baz";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    let result = validate_canonical_form(&program);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0], ValidationError::DuplicateDeclaration { .. }));
}

#[test]
fn test_duplicate_consts() {
    let source = "c pi:ℝ=3.14\nc pi:ℝ=3.15";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    let result = validate_canonical_form(&program);
    assert!(result.is_err());
}

#[test]
fn test_duplicate_imports() {
    let source = "i stdlib⋅list\ni stdlib⋅list";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    let result = validate_canonical_form(&program);
    assert!(result.is_err());
}

#[test]
fn test_no_duplicates_different_names() {
    let source = "λfoo()→ℤ=1\nλbar()→ℤ=2\nc baz:ℤ=3";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_canonical_form(&program).is_ok());
}

#[test]
fn test_different_declaration_types() {
    // Different declaration types don't conflict
    let source = "t Maybe=Some(ℤ)|None\nλfoo()→ℤ=1\nc bar:ℤ=2";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    // This should pass - different declaration types and names
    assert!(validate_canonical_form(&program).is_ok());
}

// ============================================================================
// RECURSION VALIDATION TESTS
// ============================================================================

#[test]
fn test_non_recursive_function() {
    let source = "λadd(x:ℤ,y:ℤ)→ℤ=x+y";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_canonical_form(&program).is_ok());
}

#[test]
fn test_recursive_single_param() {
    let source = "λcountdown(n:ℤ)→ℤ=countdown(n-1)";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    // Simple recursion is allowed
    assert!(validate_canonical_form(&program).is_ok());
}

#[test]
fn test_cps_rejected() {
    // TODO: This test requires function type syntax to work
    // For now, just verify basic validation works
    let source = "λfoo()→ℤ=1";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_canonical_form(&program).is_ok());
}

// ============================================================================
// SURFACE FORM VALIDATION TESTS
// ============================================================================

#[test]
fn test_surface_form_with_type_annotations() {
    let source = "λfoo(x:ℤ)→ℤ=x";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_surface_form(&program).is_ok());
}

#[test]
fn test_surface_form_const_with_type() {
    let source = "c answer:ℤ=42";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_surface_form(&program).is_ok());
}

#[test]
fn test_surface_form_multiple_functions() {
    let source = "λa()→ℤ=1\nλb()→ℤ=2";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_surface_form(&program).is_ok());
}

// ============================================================================
// COMBINED VALIDATION TESTS
// ============================================================================

#[test]
fn test_valid_program_both_validators() {
    let source = "λfib(n:ℤ)→ℤ=fib(n-1)+fib(n-2)";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_canonical_form(&program).is_ok());
    assert!(validate_surface_form(&program).is_ok());
}

#[test]
fn test_multiple_errors_collected() {
    let source = "λfoo()→ℤ=1\nλfoo()→ℤ=2\nλfoo()→ℤ=3";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    let result = validate_canonical_form(&program);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    // Should report 2 duplicates (second and third foo)
    assert_eq!(errors.len(), 2);
}

#[test]
fn test_exported_function_valid() {
    let source = "export λmain()→ℤ=42";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_canonical_form(&program).is_ok());
    assert!(validate_surface_form(&program).is_ok());
}

#[test]
fn test_mockable_function_valid() {
    let source = "mockable λfetch()→𝕊=\"data\"";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_canonical_form(&program).is_ok());
}

#[test]
fn test_type_declaration_valid() {
    let source = "t Result[T,E]=Ok(T)|Err(E)";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_canonical_form(&program).is_ok());
}

#[test]
fn test_import_valid() {
    let source = "i stdlib⋅list";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_canonical_form(&program).is_ok());
}

#[test]
fn test_const_lowercase_name() {
    let source = "c my_constant:ℤ=100";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_canonical_form(&program).is_ok());
}

#[test]
fn test_effect_annotations_valid() {
    let source = "λread()→!IO𝕊=\"\"";
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();

    assert!(validate_canonical_form(&program).is_ok());
}
