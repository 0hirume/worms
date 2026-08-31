use larvae::fmt::FmtConfig;
use larvae::fmt::config::LineEndings;

use crate::classification::Gap;
use crate::syntax::{Boundary, collect_boundaries};

/// Protect deterministic boundaries, format Luau, then enforce them again.
///
/// # Errors
///
/// Returns an error when larvae cannot format or parse the source, or when an
/// AST boundary does not lie on the source.
pub fn format_luau(source: &str, fmt: &FmtConfig) -> Result<String, String> {
    let prepared = enforce_boundaries(source, fmt)?;
    let formatted = larvae::fmt::format(&prepared, fmt)
        .map_err(|error| format!("cannot format Luau: {error:#}"))?;

    enforce_boundaries(&formatted, fmt)
}

fn enforce_boundaries(source: &str, fmt: &FmtConfig) -> Result<String, String> {
    let parsed = larvae::syntax::parse_one(source)
        .map_err(|error| format!("cannot parse byte {}: {}", error.offset, error.message))?;
    let mut boundaries = Vec::new();

    collect_boundaries(
        &parsed.chunk.block,
        source,
        &parsed.lexed,
        true,
        &mut boundaries,
    );

    apply_boundaries(source, fmt, boundaries)
}

fn apply_boundaries(
    source: &str,
    fmt: &FmtConfig,
    mut boundaries: Vec<Boundary>,
) -> Result<String, String> {
    boundaries.sort_unstable_by_key(|boundary| (boundary.start, boundary.end));
    boundaries.dedup_by_key(|boundary| (boundary.start, boundary.end));

    let newline = match fmt.line_endings {
        LineEndings::Unix => "\n",
        LineEndings::Windows => "\r\n",
    };
    let mut output = source.to_owned();

    for boundary in boundaries.into_iter().rev() {
        let gap = source
            .get(boundary.start as usize..boundary.end as usize)
            .ok_or_else(|| {
                format!(
                    "invalid statement boundary {}..{}",
                    boundary.start, boundary.end
                )
            })?;

        if !gap.bytes().all(|byte| byte.is_ascii_whitespace()) {
            continue;
        }

        let indentation = gap
            .rsplit_once('\n')
            .map_or(gap, |(_, indentation)| indentation)
            .trim_start_matches('\r');

        if !indentation.bytes().all(|byte| matches!(byte, b' ' | b'\t')) {
            continue;
        }

        let replacement = match boundary.gap {
            Gap::Tight => format!("{newline}{indentation}"),
            Gap::Blank => format!("{newline}{newline}{indentation}"),
        };

        output.replace_range(boundary.start as usize..boundary.end as usize, &replacement);
    }

    Ok(output)
}
