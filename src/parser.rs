use crate::ast::{is_lua_block_name, Block, Directive, Node, OpaqueBlock};
use crate::error::FormatError;
use crate::lexer::{Lexer, Token, TokenKind};

pub struct Parser<'a> {
    input: &'a str,
    tokens: Vec<Token>,
    index: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(input: &'a str) -> Result<Vec<Node>, FormatError> {
        let tokens = Lexer::new(input).tokenize()?;
        let mut parser = Self {
            input,
            tokens,
            index: 0,
        };
        parser.parse_nodes()
    }

    fn parse_nodes(&mut self) -> Result<Vec<Node>, FormatError> {
        let mut nodes = Vec::new();
        let mut pending_comments = Vec::new();

        while !self.is_at_end() {
            match self.peek_kind() {
                Some(TokenKind::Comment) => {
                    pending_comments.push(self.advance().text);
                }
                Some(TokenKind::Newline) => {
                    self.advance();
                    if self.peek_kind() == Some(TokenKind::Newline) {
                        nodes.push(Node::BlankLine);
                    }
                }
                Some(TokenKind::Ident) => {
                    let node = self.parse_statement(&mut pending_comments)?;
                    nodes.push(node);
                }
                Some(TokenKind::RBrace) => {
                    self.flush_pending_comments(&mut nodes, &mut pending_comments);
                    break;
                }
                Some(_) => {
                    let token = self.advance();
                    return Err(FormatError::parse(
                        token.span.line,
                        token.span.column,
                        format!("unexpected token '{}'", token.text),
                    ));
                }
                None => break,
            }
        }

        self.flush_pending_comments(&mut nodes, &mut pending_comments);
        Ok(nodes)
    }

    fn flush_pending_comments(&self, nodes: &mut Vec<Node>, pending_comments: &mut Vec<String>) {
        for comment in pending_comments.drain(..) {
            nodes.push(Node::Comment(comment));
        }
    }

    fn parse_statement(&mut self, pending_comments: &mut Vec<String>) -> Result<Node, FormatError> {
        let mut parts = Vec::new();
        let first = self.expect_ident()?;
        parts.push(first.text);

        while matches!(self.peek_kind(), Some(TokenKind::Ident | TokenKind::String)) {
            parts.push(self.advance().text);
        }

        self.skip_newlines();

        let inline_comment = self.take_inline_comment();
        let leading_comments = std::mem::take(pending_comments);

        match self.peek_kind() {
            Some(TokenKind::Semicolon) => {
                self.advance();
                let trailing_comment = self.take_inline_comment();
                let inline_comment = inline_comment.or(trailing_comment);
                let name = parts.remove(0);
                Ok(Node::Directive(Directive {
                    leading_comments,
                    name,
                    args: parts,
                    inline_comment,
                }))
            }
            Some(TokenKind::LBrace) => {
                self.advance();
                if is_lua_block_name(&parts[0]) {
                    let body = self.read_opaque_body()?;
                    self.expect_rbrace()?;
                    let header = parts.join(" ");
                    Ok(Node::OpaqueBlock(OpaqueBlock {
                        leading_comments,
                        header,
                        body,
                        inline_comment,
                    }))
                } else {
                    let name = parts.remove(0);
                    let args = parts;
                    let children = self.parse_nodes()?;
                    self.expect_rbrace()?;
                    Ok(Node::Block(Block {
                        leading_comments,
                        name,
                        args,
                        children,
                        inline_comment,
                    }))
                }
            }
            _ => {
                let token = self.peek().cloned().unwrap_or_else(|| Token {
                    kind: TokenKind::Ident,
                    text: String::new(),
                    span: crate::Span::new(0, 0, 1, 1),
                });
                Err(FormatError::parse(
                    token.span.line,
                    token.span.column,
                    format!("expected ';' or '{{' after '{}'", parts.join(" ")),
                ))
            }
        }
    }

    fn read_opaque_body(&mut self) -> Result<String, FormatError> {
        let start = self
            .peek()
            .map(|token| token.span.start)
            .unwrap_or(self.input.len());
        let mut depth = 1usize;

        while !self.is_at_end() {
            match self.peek_kind() {
                Some(TokenKind::LBrace) => {
                    depth += 1;
                    self.advance();
                }
                Some(TokenKind::RBrace) => {
                    depth -= 1;
                    if depth == 0 {
                        let end = self.peek().map(|token| token.span.start).unwrap_or(start);
                        return Ok(self.input[start..end].to_string());
                    }
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }

        Err(FormatError::parse(1, 1, "unterminated opaque block"))
    }

    fn skip_newlines(&mut self) {
        while self.peek_kind() == Some(TokenKind::Newline) {
            self.advance();
        }
    }

    fn take_inline_comment(&mut self) -> Option<String> {
        if self.peek_kind() == Some(TokenKind::Comment) {
            Some(self.advance().text)
        } else {
            None
        }
    }

    fn expect_ident(&mut self) -> Result<Token, FormatError> {
        match self.peek_kind() {
            Some(TokenKind::Ident) => Ok(self.advance()),
            _ => {
                let token = self.peek().cloned().unwrap_or_else(|| Token {
                    kind: TokenKind::Ident,
                    text: String::new(),
                    span: crate::Span::new(0, 0, 1, 1),
                });
                Err(FormatError::parse(
                    token.span.line,
                    token.span.column,
                    "expected identifier",
                ))
            }
        }
    }

    fn expect_rbrace(&mut self) -> Result<(), FormatError> {
        match self.peek_kind() {
            Some(TokenKind::RBrace) => {
                self.advance();
                Ok(())
            }
            _ => {
                let token = self.peek().cloned().unwrap_or_else(|| Token {
                    kind: TokenKind::RBrace,
                    text: String::new(),
                    span: crate::Span::new(0, 0, 1, 1),
                });
                Err(FormatError::parse(
                    token.span.line,
                    token.span.column,
                    "expected '}'",
                ))
            }
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|token| token.kind.clone())
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.index].clone();
        self.index += 1;
        token
    }

    fn is_at_end(&self) -> bool {
        self.index >= self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_directive() {
        let nodes = Parser::parse("worker_processes 4;").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Directive(d) => {
                assert_eq!(d.name, "worker_processes");
                assert_eq!(d.args, vec!["4"]);
            }
            _ => panic!("expected directive"),
        }
    }

    #[test]
    fn parses_nested_blocks() {
        let nodes = Parser::parse(
            r#"
server {
    location / {
        proxy_pass http://backend;
    }
}
"#,
        )
        .unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Block(server) => {
                assert_eq!(server.name, "server");
                assert_eq!(server.children.len(), 1);
            }
            _ => panic!("expected block"),
        }
    }

    #[test]
    fn parses_lua_block_as_opaque() {
        let nodes = Parser::parse(
            r#"
content_by_lua_block {
    ngx.say("hello")
}
"#,
        )
        .unwrap();
        match &nodes[0] {
            Node::OpaqueBlock(block) => {
                assert!(block.header.contains("content_by_lua_block"));
                assert!(block.body.contains("ngx.say"));
            }
            _ => panic!("expected opaque block"),
        }
    }

    #[test]
    fn parses_trailing_comments() {
        let nodes = Parser::parse("server { listen 80; }\n# trailing\n").unwrap();
        assert_eq!(nodes.len(), 2);
        match &nodes[1] {
            Node::Comment(text) => assert_eq!(text, "# trailing"),
            _ => panic!("expected trailing comment"),
        }
    }

    #[test]
    fn parses_comment_before_closing_brace() {
        let nodes = Parser::parse("server { listen 80;\n# note\n}").unwrap();
        match &nodes[0] {
            Node::Block(block) => {
                assert_eq!(block.children.len(), 2);
                match &block.children[1] {
                    Node::Comment(text) => assert_eq!(text, "# note"),
                    _ => panic!("expected comment before brace"),
                }
            }
            _ => panic!("expected block"),
        }
    }

    #[test]
    fn parses_comment_only_file() {
        let nodes = Parser::parse("# just a comment\n").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Comment(text) => assert_eq!(text, "# just a comment"),
            _ => panic!("expected comment-only node"),
        }
    }
}
