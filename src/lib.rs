pub mod ast;
#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
pub mod error;
pub mod formatter;
pub mod lexer;
pub mod parser;

pub use config::Config;
pub use error::FormatError;
pub use error::Span;

use formatter::Formatter;
use parser::Parser;

pub fn normalize_line_endings(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn format_str(input: &str, config: &Config) -> Result<String, FormatError> {
    let input = normalize_line_endings(input);
    let nodes = Parser::parse(&input)?;
    Ok(Formatter::new(config).format(&nodes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BraceStyle, IndentStyle};

    #[test]
    fn round_trip_basic_server_block() {
        let input = "server{listen 80;server_name example.com;}";
        let formatted = format_str(input, &Config::default()).unwrap();
        assert!(formatted.contains("server {\n"));
        assert!(formatted.contains("listen 80;\n"));
        assert!(formatted.contains("server_name example.com;\n"));
    }

    #[test]
    fn idempotent_default_config() {
        let input = include_str!("../tests/fixtures/basic.in.conf");
        let once = format_str(input, &Config::default()).unwrap();
        let twice = format_str(&once, &Config::default()).unwrap();
        assert_eq!(once, twice);
    }

    #[test]
    fn respects_indent_width_option() {
        let input = "server { listen 80; }";
        let config = Config {
            indent_width: 2,
            ..Config::default()
        };
        let formatted = format_str(input, &config).unwrap();
        assert!(formatted.contains("  listen 80;\n"));
    }

    #[test]
    fn respects_brace_style_option() {
        let input = "server { listen 80; }";
        let config = Config {
            brace_style: BraceStyle::NextLine,
            ..Config::default()
        };
        let formatted = format_str(input, &config).unwrap();
        assert!(formatted.contains("server\n"));
        assert!(formatted.contains("{\n"));
    }

    #[test]
    fn respects_tabs_option() {
        let input = "server { listen 80; }";
        let config = Config {
            indent_style: IndentStyle::Tabs,
            ..Config::default()
        };
        let formatted = format_str(input, &config).unwrap();
        assert!(formatted.contains("\tlisten 80;\n"));
    }
}
