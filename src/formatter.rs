use crate::ast::{Block, Directive, Node, OpaqueBlock};
use crate::config::{BraceStyle, Config};

pub struct Formatter<'a> {
    config: &'a Config,
    output: String,
    blank_run: u8,
}

impl<'a> Formatter<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            output: String::new(),
            blank_run: 0,
        }
    }

    pub fn format(mut self, nodes: &[Node]) -> String {
        self.write_nodes(nodes, 0);
        self.collapse_trailing_blank_lines();

        if self.config.trailing_newline && !self.output.ends_with('\n') {
            self.output.push('\n');
        } else if !self.config.trailing_newline {
            while self.output.ends_with('\n') {
                self.output.pop();
            }
        }

        self.output
    }

    fn write_nodes(&mut self, nodes: &[Node], depth: usize) {
        for node in nodes {
            self.write_node(node, depth);
        }
    }

    fn write_node(&mut self, node: &Node, depth: usize) {
        match node {
            Node::Directive(directive) => self.write_directive(directive, depth),
            Node::Block(block) => self.write_block(block, depth),
            Node::OpaqueBlock(block) => self.write_opaque_block(block, depth),
            Node::Comment(comment) => self.write_comment_line(comment, depth),
            Node::BlankLine => self.write_blank_line(),
        }
    }

    fn write_directive(&mut self, directive: &Directive, depth: usize) {
        self.blank_run = 0;
        for comment in &directive.leading_comments {
            self.write_comment_line(comment, depth);
        }

        self.output.push_str(&self.config.indent_at(depth));
        self.output.push_str(&directive.name);
        if !directive.args.is_empty() {
            self.output.push(' ');
            self.output.push_str(&directive.args.join(" "));
        }
        self.output.push(';');
        self.write_inline_comment(directive.inline_comment.as_deref());
        self.output.push('\n');
    }

    fn write_block(&mut self, block: &Block, depth: usize) {
        self.blank_run = 0;
        for comment in &block.leading_comments {
            self.write_comment_line(comment, depth);
        }

        let header = format_block_header(&block.name, &block.args);

        match self.config.brace_style {
            BraceStyle::SameLine => {
                self.output.push_str(&self.config.indent_at(depth));
                self.output.push_str(&header);
                self.output.push(' ');
                self.write_inline_comment(block.inline_comment.as_deref());
                self.output.push_str("{\n");
            }
            BraceStyle::NextLine => {
                self.output.push_str(&self.config.indent_at(depth));
                self.output.push_str(&header);
                self.write_inline_comment(block.inline_comment.as_deref());
                self.output.push('\n');
                self.output.push_str(&self.config.indent_at(depth));
                self.output.push_str("{\n");
            }
        }

        self.write_nodes(&block.children, depth + 1);
        self.output.push_str(&self.config.indent_at(depth));
        self.output.push_str("}\n");
    }

    fn write_opaque_block(&mut self, block: &OpaqueBlock, depth: usize) {
        self.blank_run = 0;
        for comment in &block.leading_comments {
            self.write_comment_line(comment, depth);
        }

        self.write_opaque_opening(block, depth);

        let body = normalize_line_endings(&block.body);
        let body = body.strip_prefix('\n').unwrap_or(&body);
        let body = body.trim_end();
        if !body.is_empty() {
            self.output.push_str(body);
        }
        self.output.push('\n');

        self.output.push_str(&self.config.indent_at(depth));
        self.output.push_str("}\n");
    }

    fn write_opaque_opening(&mut self, block: &OpaqueBlock, depth: usize) {
        match self.config.brace_style {
            BraceStyle::SameLine => {
                self.output.push_str(&self.config.indent_at(depth));
                self.output.push_str(&block.header);
                self.output.push(' ');
                self.write_inline_comment(block.inline_comment.as_deref());
                self.output.push_str("{\n");
            }
            BraceStyle::NextLine => {
                self.output.push_str(&self.config.indent_at(depth));
                self.output.push_str(&block.header);
                self.write_inline_comment(block.inline_comment.as_deref());
                self.output.push('\n');
                self.output.push_str(&self.config.indent_at(depth));
                self.output.push_str("{\n");
            }
        }
    }

    fn write_comment_line(&mut self, comment: &str, depth: usize) {
        self.blank_run = 0;
        self.output.push_str(&self.config.indent_at(depth));
        self.output.push_str(comment);
        if !comment.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn write_blank_line(&mut self) {
        if self.blank_run >= self.config.max_blank_lines {
            return;
        }
        self.blank_run += 1;
        self.output.push('\n');
    }

    fn write_inline_comment(&mut self, comment: Option<&str>) {
        if !self.config.preserve_inline_comments {
            return;
        }
        if let Some(comment) = comment {
            if !self.output.ends_with(' ') && !self.output.ends_with('\n') {
                self.output.push(' ');
            }
            self.output.push_str(comment);
        }
    }

    fn collapse_trailing_blank_lines(&mut self) {
        while self.output.ends_with("\n\n") && self.blank_run > self.config.max_blank_lines {
            self.output.pop();
            self.blank_run = self.blank_run.saturating_sub(1);
        }
    }
}

fn format_block_header(name: &str, args: &[String]) -> String {
    if args.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", name, args.join(" "))
    }
}

fn normalize_line_endings(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BraceStyle, IndentStyle};

    fn format_with(input: &str, config: Config) -> String {
        let nodes = crate::parser::Parser::parse(input).unwrap();
        Formatter::new(&config).format(&nodes)
    }

    #[test]
    fn formats_same_line_braces() {
        let output = format_with("server{listen 80;}", Config::default());
        assert!(output.contains("server {\n"));
        assert!(output.contains("    listen 80;\n"));
    }

    #[test]
    fn formats_next_line_braces() {
        let config = Config {
            brace_style: BraceStyle::NextLine,
            ..Config::default()
        };
        let output = format_with("server{listen 80;}", config);
        assert!(output.contains("server\n"));
        assert!(output.contains("{\n"));
    }

    #[test]
    fn formats_with_tabs() {
        let config = Config {
            indent_style: IndentStyle::Tabs,
            ..Config::default()
        };
        let output = format_with("server { listen 80; }", config);
        assert!(output.contains("\tlisten 80;\n"));
    }
}
