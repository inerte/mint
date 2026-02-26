//! Bidirectional Type Checking for Sigil
//!
//! Uses two complementary modes:
//! - Synthesis (⇒): Infer type from expression structure (bottom-up)
//! - Checking (⇐): Verify expression matches expected type (top-down)
//!
//! This is simpler than Hindley-Milner because Sigil requires mandatory
//! type annotations everywhere, making the inference burden much lighter.

use crate::environment::TypeEnvironment;
use crate::errors::TypeError;
use crate::types::InferenceType;
use crate::TypeCheckOptions;
use sigil_ast::Program;
use std::collections::HashMap;

/// Type check a Sigil program
///
/// Returns a map of function names to their inferred types
pub fn type_check(
    _program: &Program,
    _source_code: &str,
    _options: TypeCheckOptions,
) -> Result<HashMap<String, InferenceType>, TypeError> {
    // TODO: Implement bidirectional type checking
    //
    // This will involve:
    // 1. Creating initial environment
    // 2. Processing all declarations (types, imports, consts, functions)
    // 3. Type checking each function body
    // 4. Returning map of function names to types
    
    Ok(HashMap::new())
}
