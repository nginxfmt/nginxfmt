use wasm_bindgen::prelude::*;

use crate::config::{BraceStyle, Config, IndentStyle};

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn format(
    input: &str,
    indent_style: &str,
    indent_width: u8,
    brace_style: &str,
    max_blank_lines: u8,
    trailing_newline: bool,
    preserve_inline_comments: bool,
) -> Result<String, JsError> {
    let config = Config {
        indent_style: if indent_style == "tabs" {
            IndentStyle::Tabs
        } else {
            IndentStyle::Spaces
        },
        indent_width,
        brace_style: if brace_style == "next_line" {
            BraceStyle::NextLine
        } else {
            BraceStyle::SameLine
        },
        max_blank_lines,
        trailing_newline,
        preserve_inline_comments,
    };
    crate::format_str(input, &config).map_err(|e| JsError::new(&e.to_string()))
}
