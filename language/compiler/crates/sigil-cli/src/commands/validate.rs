use super::legacy::CliError;
use std::path::Path;

pub fn validate_command(path: &Path, env: &str) -> Result<(), CliError> {
    super::compile_support::validate_command(path, env)
}
