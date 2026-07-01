use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::FormatError;

pub const CONFIG_FILE_NAME: &str = ".nginxfmt.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IndentStyle {
    #[default]
    Spaces,
    Tabs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BraceStyle {
    #[default]
    SameLine,
    NextLine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub indent_style: IndentStyle,
    #[serde(default = "default_indent_width")]
    pub indent_width: u8,
    #[serde(default)]
    pub brace_style: BraceStyle,
    #[serde(default = "default_max_blank_lines")]
    pub max_blank_lines: u8,
    #[serde(default = "default_true")]
    pub trailing_newline: bool,
    #[serde(default = "default_true")]
    pub preserve_inline_comments: bool,
}

fn default_indent_width() -> u8 {
    4
}

fn default_max_blank_lines() -> u8 {
    1
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Spaces,
            indent_width: 4,
            brace_style: BraceStyle::SameLine,
            max_blank_lines: 1,
            trailing_newline: true,
            preserve_inline_comments: true,
        }
    }
}

impl Config {
    pub fn from_toml_str(content: &str) -> Result<Self, FormatError> {
        toml::from_str(content)
            .map_err(|err| FormatError::Config(format!("invalid config file: {err}")))
    }

    pub fn from_file(path: &Path) -> Result<Self, FormatError> {
        let content = fs::read_to_string(path)?;
        Self::from_toml_str(&content)
    }

    pub fn discover(start: Option<&Path>) -> Result<Option<(PathBuf, Self)>, FormatError> {
        let mut dir = match start {
            Some(path) if path.is_file() => path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from(".")),
            Some(path) => path.to_path_buf(),
            None => std::env::current_dir()?,
        };

        loop {
            let candidate = dir.join(CONFIG_FILE_NAME);
            if candidate.is_file() {
                return Ok(Some((candidate.clone(), Self::from_file(&candidate)?)));
            }

            if !dir.pop() {
                break;
            }
        }

        Ok(None)
    }

    pub fn merge_cli_overrides(&mut self, overrides: &CliOverrides) {
        if let Some(indent_style) = overrides.indent_style {
            self.indent_style = indent_style;
        }
        if let Some(indent_width) = overrides.indent_width {
            self.indent_width = indent_width;
        }
        if let Some(brace_style) = overrides.brace_style {
            self.brace_style = brace_style;
        }
        if let Some(max_blank_lines) = overrides.max_blank_lines {
            self.max_blank_lines = max_blank_lines;
        }
        if let Some(trailing_newline) = overrides.trailing_newline {
            self.trailing_newline = trailing_newline;
        }
        if let Some(preserve_inline_comments) = overrides.preserve_inline_comments {
            self.preserve_inline_comments = preserve_inline_comments;
        }
    }

    pub fn indent_unit(&self) -> String {
        match self.indent_style {
            IndentStyle::Spaces => " ".repeat(self.indent_width as usize),
            IndentStyle::Tabs => "\t".to_string(),
        }
    }

    pub fn indent_at(&self, level: usize) -> String {
        match self.indent_style {
            IndentStyle::Spaces => " ".repeat(self.indent_width as usize * level),
            IndentStyle::Tabs => "\t".repeat(level),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CliOverrides {
    pub indent_style: Option<IndentStyle>,
    pub indent_width: Option<u8>,
    pub brace_style: Option<BraceStyle>,
    pub max_blank_lines: Option<u8>,
    pub trailing_newline: Option<bool>,
    pub preserve_inline_comments: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parses_toml_config() {
        let cfg = Config::from_toml_str(
            r#"
indent_style = "tabs"
indent_width = 2
brace_style = "next_line"
max_blank_lines = 2
trailing_newline = false
preserve_inline_comments = false
"#,
        )
        .unwrap();

        assert_eq!(cfg.indent_style, IndentStyle::Tabs);
        assert_eq!(cfg.indent_width, 2);
        assert_eq!(cfg.brace_style, BraceStyle::NextLine);
        assert_eq!(cfg.max_blank_lines, 2);
        assert!(!cfg.trailing_newline);
        assert!(!cfg.preserve_inline_comments);
    }

    #[test]
    fn discovers_config_walking_up() {
        let root = TempDir::new().unwrap();
        let nested = root.path().join("conf").join("sites");
        std::fs::create_dir_all(&nested).unwrap();
        let config_path = root.path().join(CONFIG_FILE_NAME);
        std::fs::write(
            &config_path,
            "indent_width = 8\nbrace_style = \"next_line\"\n",
        )
        .unwrap();

        let (found, cfg) = Config::discover(Some(&nested.join("default.conf")))
            .unwrap()
            .expect("config should be discovered");

        assert_eq!(found, config_path);
        assert_eq!(cfg.indent_width, 8);
        assert_eq!(cfg.brace_style, BraceStyle::NextLine);
    }

    #[test]
    fn merge_cli_overrides() {
        let mut cfg = Config::default();
        let overrides = CliOverrides {
            indent_style: Some(IndentStyle::Tabs),
            indent_width: Some(2),
            ..CliOverrides::default()
        };
        cfg.merge_cli_overrides(&overrides);
        assert_eq!(cfg.indent_style, IndentStyle::Tabs);
        assert_eq!(cfg.indent_width, 2);
    }
}
