use nginxfmt::config::{BraceStyle, Config, IndentStyle};
use nginxfmt::format_str;
use proptest::prelude::*;

fn arb_config() -> impl Strategy<Value = Config> {
    (
        any::<bool>(),
        1u8..=8u8,
        any::<bool>(),
        1u8..=3u8,
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(tabs, width, next_line, max_blank, trailing, preserve)| Config {
                indent_style: if tabs {
                    IndentStyle::Tabs
                } else {
                    IndentStyle::Spaces
                },
                indent_width: width,
                brace_style: if next_line {
                    BraceStyle::NextLine
                } else {
                    BraceStyle::SameLine
                },
                max_blank_lines: max_blank,
                trailing_newline: trailing,
                preserve_inline_comments: preserve,
            },
        )
}

fn arb_snippet() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("worker_processes 1;".to_string()),
        Just("events { worker_connections 1024; }".to_string()),
        Just("server { listen 80; location / { root /var/www; } }".to_string()),
        Just("http { include mime.types; sendfile on; }".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn format_is_idempotent(input in arb_snippet(), config in arb_config()) {
        let once = format_str(&input, &config).expect("format once");
        let twice = format_str(&once, &config).expect("format twice");
        prop_assert_eq!(once, twice);
    }
}
