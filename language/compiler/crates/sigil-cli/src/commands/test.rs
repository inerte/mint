use super::legacy::CliError;
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub fn test_command(
    path: &Path,
    env: Option<&str>,
    filter: Option<&str>,
    trace: bool,
    trace_expr: bool,
    breakpoints: &[String],
    break_fns: &[String],
    break_spans: &[String],
    breakpoint_collect: bool,
    break_max_hits: usize,
    record: Option<&Path>,
    replay: Option<&Path>,
) -> Result<(), CliError> {
    super::legacy::test_command(
        path,
        env,
        filter,
        trace,
        trace_expr,
        breakpoints,
        break_fns,
        break_spans,
        breakpoint_collect,
        break_max_hits,
        record,
        replay,
    )
}
