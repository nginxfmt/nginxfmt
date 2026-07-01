use std::fs;
use std::path::Path;

use nginxfmt::{format_str, Config};

fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

fn fixtures_dir() -> &'static Path {
    Path::new("tests/fixtures")
}

#[test]
fn golden_fixtures_match_expected_output() {
    for entry in fs::read_dir(fixtures_dir()).expect("fixtures directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if let Some(stem) = name.strip_suffix(".in.conf") {
            let input = fs::read_to_string(&path).unwrap();
            let expected_path = fixtures_dir().join(format!("{stem}.out.conf"));
            let expected = fs::read_to_string(expected_path).unwrap();
            let actual = format_str(&input, &Config::default()).unwrap();
            assert_eq!(
                normalize(&actual),
                normalize(&expected),
                "fixture mismatch for {stem}"
            );
        }
    }
}

#[test]
fn golden_fixtures_are_idempotent() {
    for entry in fs::read_dir(fixtures_dir()).expect("fixtures directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if name.ends_with(".in.conf") {
            let input = fs::read_to_string(&path).unwrap();
            let once = format_str(&input, &Config::default()).unwrap();
            let twice = format_str(&once, &Config::default()).unwrap();
            assert_eq!(once, twice, "idempotency failed for {name}");
        }
    }
}
