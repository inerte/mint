use super::legacy::CliError;
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub fn run_command(
    file: &Path,
    json_output: bool,
    trace: bool,
    trace_expr: bool,
    breakpoints: &[String],
    break_fns: &[String],
    break_spans: &[String],
    breakpoint_collect: bool,
    break_max_hits: usize,
    record: Option<&Path>,
    replay: Option<&Path>,
    env: Option<&str>,
    args: &[String],
) -> Result<(), CliError> {
    super::legacy::run_command(
        file,
        json_output,
        trace,
        trace_expr,
        breakpoints,
        break_fns,
        break_spans,
        breakpoint_collect,
        break_max_hits,
        record,
        replay,
        env,
        args,
    )
}
