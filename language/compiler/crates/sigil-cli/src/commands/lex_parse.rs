use super::legacy::CliError;
use std::path::Path;

pub fn lex_command(file: &Path) -> Result<(), CliError> {
    super::legacy::lex_command(file)
}

pub fn parse_command(file: &Path) -> Result<(), CliError> {
    super::legacy::parse_command(file)
}
