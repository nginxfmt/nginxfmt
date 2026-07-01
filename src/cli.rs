use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory, Parser as ClapParser, ValueEnum};
use clap_complete::{generate, shells};

use crate::config::{BraceStyle, CliOverrides, Config, IndentStyle};
use crate::error::FormatError;

#[derive(Debug, ClapParser)]
#[command(
    name = "nginxfmt",
    version,
    about = "Format nginx configuration files",
    after_help = "By default, formatted output is written to stdout. Use --write to edit files in place."
)]
pub struct Cli {
    /// Path to nginx config file. Reads from stdin when omitted or '-'.
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Write formatted output back to the file.
    #[arg(short, long)]
    pub write: bool,

    /// Check whether the file is formatted; exit with status 1 if not.
    #[arg(long, conflicts_with = "write")]
    pub check: bool,

    /// Path to a nginxfmt config file.
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Use tabs for indentation.
    #[arg(long, conflicts_with = "spaces")]
    pub tabs: bool,

    /// Use spaces for indentation.
    #[arg(long, conflicts_with = "tabs")]
    pub spaces: bool,

    /// Number of spaces per indentation level.
    #[arg(long, value_name = "N")]
    pub indent_width: Option<u8>,

    /// Brace placement style.
    #[arg(long, value_enum)]
    pub brace_style: Option<BraceStyleArg>,

    /// Maximum consecutive blank lines to preserve.
    #[arg(long, value_name = "N")]
    pub max_blank_lines: Option<u8>,

    /// Ensure output ends with a trailing newline.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub trailing_newline: bool,

    /// Omit the trailing newline at end of output.
    #[arg(long, action = clap::ArgAction::SetTrue, conflicts_with = "trailing_newline")]
    pub no_trailing_newline: bool,

    /// Preserve inline comments on the same line.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub preserve_inline_comments: bool,

    /// Strip inline comments instead of preserving them.
    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        conflicts_with = "preserve_inline_comments"
    )]
    pub no_preserve_inline_comments: bool,

    /// Generate shell completions for the given shell.
    #[arg(long, value_enum, hide = true)]
    pub generate_completions: Option<ShellArg>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum BraceStyleArg {
    SameLine,
    NextLine,
}

impl From<BraceStyleArg> for BraceStyle {
    fn from(value: BraceStyleArg) -> Self {
        match value {
            BraceStyleArg::SameLine => BraceStyle::SameLine,
            BraceStyleArg::NextLine => BraceStyle::NextLine,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ShellArg {
    Bash,
    Fish,
    Zsh,
}

impl Cli {
    pub fn run(self) -> ExitCode {
        if let Some(shell) = self.generate_completions {
            return generate_shell_completions(shell);
        }

        match self.execute() {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("nginxfmt: {err}");
                ExitCode::FAILURE
            }
        }
    }

    fn execute(self) -> Result<(), FormatError> {
        let mut config = load_config(&self)?;
        config.merge_cli_overrides(&self.cli_overrides());

        let input = read_input(self.file.as_deref())?;
        let formatted = crate::format_str(&input, &config)?;

        if self.check {
            let normalized_input = crate::normalize_line_endings(&input);
            if formatted == normalized_input {
                return Ok(());
            }
            return Err(FormatError::Config(
                "file is not formatted according to current options".to_string(),
            ));
        }

        if self.write {
            let path = self
                .file
                .as_ref()
                .ok_or_else(|| FormatError::Config("--write requires a file path".to_string()))?;
            if path.as_os_str() == "-" {
                return Err(FormatError::Config(
                    "cannot write to stdin; omit --write to print to stdout".to_string(),
                ));
            }
            std::fs::write(path, formatted)?;
            return Ok(());
        }

        let mut stdout = io::stdout().lock();
        stdout.write_all(formatted.as_bytes())?;
        Ok(())
    }

    fn cli_overrides(&self) -> CliOverrides {
        let indent_style = if self.tabs {
            Some(IndentStyle::Tabs)
        } else if self.spaces {
            Some(IndentStyle::Spaces)
        } else {
            None
        };

        CliOverrides {
            indent_style,
            indent_width: self.indent_width,
            brace_style: self.brace_style.map(Into::into),
            max_blank_lines: self.max_blank_lines,
            trailing_newline: match (self.trailing_newline, self.no_trailing_newline) {
                (true, false) => Some(true),
                (false, true) => Some(false),
                _ => None,
            },
            preserve_inline_comments: match (
                self.preserve_inline_comments,
                self.no_preserve_inline_comments,
            ) {
                (true, false) => Some(true),
                (false, true) => Some(false),
                _ => None,
            },
        }
    }
}

fn load_config(cli: &Cli) -> Result<Config, FormatError> {
    if let Some(path) = &cli.config {
        return Config::from_file(path);
    }

    if let Some((_, config)) = Config::discover(cli.file.as_deref())? {
        return Ok(config);
    }

    Ok(Config::default())
}

fn read_input(file: Option<&Path>) -> Result<String, FormatError> {
    match file {
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
        Some(path) if path.as_os_str() == "-" => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
        Some(path) => Ok(std::fs::read_to_string(path)?),
    }
}

fn generate_shell_completions(shell: ShellArg) -> ExitCode {
    let mut cmd = Cli::command();
    let name = "nginxfmt";

    match shell {
        ShellArg::Bash => generate(shells::Bash, &mut cmd, name, &mut io::stdout()),
        ShellArg::Fish => generate(shells::Fish, &mut cmd, name, &mut io::stdout()),
        ShellArg::Zsh => generate(shells::Zsh, &mut cmd, name, &mut io::stdout()),
    }

    ExitCode::SUCCESS
}

pub fn write_packaged_completions() -> std::io::Result<()> {
    use std::fs::File;

    let mut cmd = Cli::command();
    let out_dir = Path::new("packaging/completions");
    std::fs::create_dir_all(out_dir)?;

    generate(
        shells::Bash,
        &mut cmd,
        "nginxfmt",
        &mut File::create(out_dir.join("nginxfmt.bash"))?,
    );
    generate(
        shells::Fish,
        &mut cmd,
        "nginxfmt",
        &mut File::create(out_dir.join("nginxfmt.fish"))?,
    );
    generate(
        shells::Zsh,
        &mut cmd,
        "nginxfmt",
        &mut File::create(out_dir.join("_nginxfmt"))?,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_overrides_tabs() {
        let cli = Cli::try_parse_from(["nginxfmt", "--tabs", "nginx.conf"]).unwrap();
        let overrides = cli.cli_overrides();
        assert_eq!(overrides.indent_style, Some(IndentStyle::Tabs));
    }
}
