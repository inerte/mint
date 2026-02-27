//! Sigil to TypeScript Code Generator
//!
//! Compiles Sigil AST to runnable TypeScript (ES2022-compatible output).
//!
//! Key transformations:
//! - All functions become `async function`
//! - All function calls use `await`
//! - Pattern matching compiles to if/else chains with __match variables
//! - Sum type constructors compile to objects with __tag and __fields
//! - Mock runtime helpers emitted at top of file

use sigil_ast::*;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("Codegen error: {0}")]
    General(String),
}

pub struct CodegenOptions {
    pub source_file: Option<String>,
    pub output_file: Option<String>,
    pub project_root: Option<String>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            source_file: None,
            output_file: None,
            project_root: None,
        }
    }
}

pub struct TypeScriptGenerator {
    indent: usize,
    output: Vec<String>,
    source_file: Option<String>,
    output_file: Option<String>,
    project_root: Option<String>,
    test_meta_entries: Vec<String>,
    mockable_functions: HashSet<String>,
}

impl TypeScriptGenerator {
    pub fn new(options: CodegenOptions) -> Self {
        Self {
            indent: 0,
            output: Vec::new(),
            source_file: options.source_file,
            output_file: options.output_file,
            project_root: options.project_root,
            test_meta_entries: Vec::new(),
            mockable_functions: HashSet::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, CodegenError> {
        self.output.clear();
        self.indent = 0;
        self.test_meta_entries.clear();
        self.mockable_functions.clear();

        // Collect mockable functions
        for decl in &program.declarations {
            if let Declaration::Function(func) = decl {
                if func.is_mockable {
                    self.mockable_functions.insert(func.name.clone());
                }
            }
        }

        // Emit mock runtime helpers first
        self.emit_mock_runtime_helpers();

        // Generate code for all declarations
        for decl in &program.declarations {
            self.generate_declaration(decl)?;
            self.output.push("\n".to_string());
        }

        // Emit test metadata if any tests were found
        if !self.test_meta_entries.is_empty() {
            self.emit("export const __sigil_tests = [");
            self.indent += 1;
            let entries = self.test_meta_entries.clone();
            for entry in &entries {
                self.emit(&format!("{},", entry));
            }
            self.indent -= 1;
            self.emit("];");
            self.output.push("\n".to_string());
        }

        Ok(self.output.join(""))
    }

    fn emit(&mut self, line: &str) {
        let indentation = "  ".repeat(self.indent);
        self.output.push(format!("{}{}\n", indentation, line));
    }

    fn emit_mock_runtime_helpers(&mut self) {
        self.emit("// Sigil Mock Runtime");
        self.emit("const __sigil_mocks = new Map();");
        self.emit("");
        self.emit("function __sigil_call(name, impl, ...args) {");
        self.indent += 1;
        self.emit("if (__sigil_mocks.has(name)) {");
        self.indent += 1;
        self.emit("return __sigil_mocks.get(name)(...args);");
        self.indent -= 1;
        self.emit("}");
        self.emit("return impl(...args);");
        self.indent -= 1;
        self.emit("}");
        self.emit("");
        self.emit("function __sigil_with_mock(name, replacement, body) {");
        self.indent += 1;
        self.emit("__sigil_mocks.set(name, replacement);");
        self.emit("try {");
        self.indent += 1;
        self.emit("return body();");
        self.indent -= 1;
        self.emit("} finally {");
        self.indent += 1;
        self.emit("__sigil_mocks.delete(name);");
        self.indent -= 1;
        self.emit("}");
        self.indent -= 1;
        self.emit("}");
        self.emit("");
    }

    fn generate_declaration(&mut self, decl: &Declaration) -> Result<(), CodegenError> {
        match decl {
            Declaration::Function(func) => self.generate_function(func),
            Declaration::Type(type_decl) => self.generate_type_decl(type_decl),
            Declaration::Const(const_decl) => self.generate_const(const_decl),
            Declaration::Import(import) => self.generate_import(import),
            Declaration::Extern(extern_decl) => self.generate_extern(extern_decl),
            Declaration::Test(test) => self.generate_test(test),
        }
    }

    fn generate_function(&mut self, func: &FunctionDecl) -> Result<(), CodegenError> {
        let params: Vec<String> = func.params.iter().map(|p| p.name.clone()).collect();
        let params_str = params.join(", ");

        let impl_name = if func.is_mockable {
            format!("__sigil_impl_{}", func.name)
        } else {
            func.name.clone()
        };

        let should_export = func.is_exported || func.name == "main";
        let fn_keyword = if should_export {
            "export async function"
        } else {
            "async function"
        };

        self.emit(&format!("{} {}({}) {{", fn_keyword, impl_name, params_str));
        self.indent += 1;

        let body_code = self.generate_expression(&func.body)?;
        self.emit(&format!("return {};", body_code));

        self.indent -= 1;
        self.emit("}");

        // If mockable, emit wrapper
        if func.is_mockable {
            self.emit("");
            let export_keyword = if should_export { "export " } else { "" };
            self.emit(&format!("{}async function {}({}) {{", export_keyword, func.name, params_str));
            self.indent += 1;
            let args = if params.is_empty() {
                String::new()
            } else {
                format!(", {}", params_str)
            };
            self.emit(&format!("return await __sigil_call('{}', {}{});", func.name, impl_name, args));
            self.indent -= 1;
            self.emit("}");
        }

        Ok(())
    }

    fn generate_type_decl(&mut self, _type_decl: &TypeDecl) -> Result<(), CodegenError> {
        // Type declarations don't generate runtime code
        // Constructors are handled as functions
        Ok(())
    }

    fn generate_const(&mut self, const_decl: &ConstDecl) -> Result<(), CodegenError> {
        let value = self.generate_expression(&const_decl.value)?;
        let export_keyword = if const_decl.is_exported { "export " } else { "" };
        self.emit(&format!("{}const {} = {};", export_keyword, const_decl.name, value));
        Ok(())
    }

    fn generate_import(&mut self, import: &ImportDecl) -> Result<(), CodegenError> {
        // Convert Sigil import to ES module import (namespace style)
        let module_path = import.module_path.join("/");
        let namespace = import.module_path.last().unwrap();
        self.emit(&format!("import * as {} from './{}.js';", namespace, module_path));
        Ok(())
    }

    fn generate_extern(&mut self, extern_decl: &ExternDecl) -> Result<(), CodegenError> {
        // Extern declarations become ES module imports
        let module_path = extern_decl.module_path.join("/");

        if let Some(ref members) = extern_decl.members {
            let member_names: Vec<String> = members.iter().map(|m| m.name.clone()).collect();
            self.emit(&format!("import {{ {} }} from '{}';", member_names.join(", "), module_path));
        } else {
            // Import entire namespace
            let namespace = extern_decl.module_path.last().unwrap();
            self.emit(&format!("import * as {} from '{}';", namespace, module_path));
        }

        Ok(())
    }

    fn generate_test(&mut self, test: &TestDecl) -> Result<(), CodegenError> {
        // Generate a unique test name from the description
        let test_name = test.description
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .to_lowercase();
        let test_name = if test_name.is_empty() {
            format!("test_{}", self.test_meta_entries.len())
        } else {
            test_name
        };

        // Generate test function
        self.emit(&format!("async function __test_{}() {{", test_name));
        self.indent += 1;
        let body = self.generate_expression(&test.body)?;
        self.emit(&format!("return {};", body));
        self.indent -= 1;
        self.emit("}");

        // Add to test metadata
        let description = test.description.replace('\"', "\\\"");
        self.test_meta_entries.push(format!(
            "{{ name: '{}', description: '{}', fn: __test_{} }}",
            test_name, description, test_name
        ));

        Ok(())
    }

    fn generate_expression(&mut self, _expr: &Expr) -> Result<String, CodegenError> {
        // Placeholder - will implement in next iteration
        Ok("null".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_program() {
        let program = Program {
            declarations: vec![],
            location: sigil_lexer::SourceLocation {
                start: sigil_lexer::Position { line: 1, column: 1, offset: 0 },
                end: sigil_lexer::Position { line: 1, column: 1, offset: 0 },
            },
        };

        let mut gen = TypeScriptGenerator::new(CodegenOptions::default());
        let result = gen.generate(&program);
        assert!(result.is_ok());
    }
}
