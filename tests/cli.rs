use std::fs;
use std::io::Write;

use assert_cmd::Command;
use nginxfmt::config::{BraceStyle, Config, IndentStyle};
use nginxfmt::format_str;
use predicates::prelude::*;
use tempfile::TempDir;

fn nginxfmt_cmd() -> Command {
    Command::cargo_bin("nginxfmt").unwrap()
}

#[test]
fn prints_to_stdout_by_default() {
    let input = "server { listen 80; }";
    nginxfmt_cmd()
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("listen 80;"));
}

#[test]
fn check_succeeds_for_formatted_input() {
    let formatted = format_str("server { listen 80; }", &Config::default()).unwrap();
    nginxfmt_cmd()
        .arg("-")
        .arg("--check")
        .write_stdin(formatted)
        .assert()
        .success();
}

#[test]
fn check_succeeds_for_crlf_input_when_content_is_formatted() {
    let formatted = format_str("server { listen 80; }", &Config::default()).unwrap();
    let crlf = formatted.replace('\n', "\r\n");
    nginxfmt_cmd()
        .arg("-")
        .arg("--check")
        .write_stdin(crlf)
        .assert()
        .success();
}

#[test]
fn check_fails_for_unformatted_input() {
    nginxfmt_cmd()
        .arg("-")
        .arg("--check")
        .write_stdin("server{listen 80;}")
        .assert()
        .failure();
}

#[test]
fn write_updates_file_in_place() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nginx.conf");
    fs::write(&path, "server{listen 80;}").unwrap();

    nginxfmt_cmd().arg("--write").arg(&path).assert().success();

    let updated = fs::read_to_string(&path).unwrap();
    assert!(updated.contains("server {\n"));
    assert!(updated.contains("listen 80;\n"));
}

#[test]
fn cli_indent_width_override() {
    let output = nginxfmt_cmd()
        .arg("--indent-width")
        .arg("2")
        .arg("-")
        .write_stdin("server { listen 80; }")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("  listen 80;\n"));
}

#[test]
fn cli_brace_style_override() {
    let output = nginxfmt_cmd()
        .arg("--brace-style")
        .arg("next_line")
        .arg("-")
        .write_stdin("server { listen 80; }")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("server\n"));
    assert!(stdout.contains("{\n"));
}

#[test]
fn option_matrix_brace_styles() {
    let input = "server { listen 80; }";

    let same_line = format_str(
        input,
        &Config {
            brace_style: BraceStyle::SameLine,
            ..Config::default()
        },
    )
    .unwrap();
    assert!(same_line.contains("server {\n"));

    let next_line = format_str(
        input,
        &Config {
            brace_style: BraceStyle::NextLine,
            ..Config::default()
        },
    )
    .unwrap();
    assert!(next_line.contains("server\n{\n"));
}

#[test]
fn option_matrix_indent_styles() {
    let input = "server { listen 80; }";

    let spaces = format_str(
        input,
        &Config {
            indent_style: IndentStyle::Spaces,
            indent_width: 2,
            ..Config::default()
        },
    )
    .unwrap();
    assert!(spaces.contains("  listen 80;\n"));

    let tabs = format_str(
        input,
        &Config {
            indent_style: IndentStyle::Tabs,
            ..Config::default()
        },
    )
    .unwrap();
    assert!(tabs.contains("\tlisten 80;\n"));
}

#[test]
fn loads_config_file_from_disk() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join(".nginxfmt.toml");
    let mut file = fs::File::create(&config_path).unwrap();
    writeln!(file, "indent_width = 2").unwrap();
    writeln!(file, "brace_style = \"next_line\"").unwrap();

    let conf_path = dir.path().join("nginx.conf");
    fs::write(&conf_path, "server { listen 80; }").unwrap();

    let output = nginxfmt_cmd()
        .current_dir(dir.path())
        .arg(&conf_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("server\n"));
    assert!(stdout.contains("  listen 80;\n"));
}
