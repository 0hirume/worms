mod classification;
mod formatting;
mod syntax;

use larvae::fmt::FmtConfig;
use larvae_worm::native::{Doc, Format, Handler, Settings};

pub use formatting::format_luau;

/// Native worm state carrying the project's resolved formatter settings.
#[derive(Default)]
pub struct VerticalSpacing {
    fmt: FmtConfig,
}

impl Handler for VerticalSpacing {
    fn init(&mut self, _config: &str, _rules: &str, settings: &Settings) -> Result<(), String> {
        if settings.fmt.is_empty() {
            return Ok(());
        }

        self.fmt = serde_json::from_str(&settings.fmt)
            .map_err(|error| format!("cannot read the resolved format settings: {error}"))?;

        Ok(())
    }

    fn transform(&mut self, source: &str) -> Result<String, String> {
        Ok(source.to_owned())
    }

    fn format(&mut self, source: &str) -> Result<Format, String> {
        let comments = larvae::syntax::lexer::lex(source)
            .map_err(|error| format!("cannot lex byte {}: {}", error.offset, error.message))?
            .comments;
        let spaced = format_luau(source, &self.fmt)?;
        let document = Doc::lit(spaced.trim_end_matches(['\r', '\n']));

        Ok(Format::document(document).with_comments(comments))
    }
}
