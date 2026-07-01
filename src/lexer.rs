use crate::error::FormatError;
use crate::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    String,
    LBrace,
    RBrace,
    Semicolon,
    Comment,
    Newline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: Span,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, FormatError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_trivia(&mut tokens)?;

            if self.is_at_end() {
                break;
            }

            let start = self.pos;
            let line = self.line;
            let column = self.column;
            let ch = self.peek_char();

            let (kind, text) = match ch {
                '{' => {
                    self.advance();
                    (TokenKind::LBrace, "{".to_string())
                }
                '}' => {
                    self.advance();
                    (TokenKind::RBrace, "}".to_string())
                }
                ';' => {
                    self.advance();
                    (TokenKind::Semicolon, ";".to_string())
                }
                '\'' | '"' => self.read_quoted_string(ch)?,
                '#' => self.read_comment()?,
                _ if is_atom_char(ch) => self.read_atom(),
                _ => {
                    return Err(FormatError::parse(
                        line,
                        column,
                        format!("unexpected character '{ch}'"),
                    ));
                }
            };

            let end = self.pos;
            tokens.push(Token {
                kind,
                text,
                span: Span::new(start, end, line, column),
            });
        }

        Ok(tokens)
    }

    fn skip_trivia(&mut self, tokens: &mut Vec<Token>) -> Result<(), FormatError> {
        loop {
            if self.is_at_end() {
                return Ok(());
            }

            let ch = self.peek_char();
            if ch == '\n' {
                let start = self.pos;
                let line = self.line;
                let column = self.column;
                self.advance();
                tokens.push(Token {
                    kind: TokenKind::Newline,
                    text: "\n".to_string(),
                    span: Span::new(start, self.pos, line, column),
                });
                continue;
            }

            if ch.is_whitespace() && ch != '\n' {
                self.advance();
                continue;
            }

            return Ok(());
        }
    }

    fn read_atom(&mut self) -> (TokenKind, String) {
        let start = self.pos;
        while !self.is_at_end() && is_atom_char(self.peek_char()) {
            self.advance();
        }
        (TokenKind::Ident, self.input[start..self.pos].to_string())
    }

    fn read_quoted_string(&mut self, quote: char) -> Result<(TokenKind, String), FormatError> {
        let start = self.pos;
        self.advance(); // opening quote

        while !self.is_at_end() {
            let ch = self.peek_char();
            if ch == '\\' {
                self.advance();
                if self.is_at_end() {
                    return Err(FormatError::parse(
                        self.line,
                        self.column,
                        "unterminated escape in string",
                    ));
                }
                self.advance();
                continue;
            }
            if ch == quote {
                self.advance();
                return Ok((TokenKind::String, self.input[start..self.pos].to_string()));
            }
            if ch == '\n' {
                return Err(FormatError::parse(
                    self.line,
                    self.column,
                    "unterminated string literal",
                ));
            }
            self.advance();
        }

        Err(FormatError::parse(
            self.line,
            self.column,
            "unterminated string literal",
        ))
    }

    fn read_comment(&mut self) -> Result<(TokenKind, String), FormatError> {
        let start = self.pos;
        while !self.is_at_end() && self.peek_char() != '\n' {
            self.advance();
        }
        Ok((TokenKind::Comment, self.input[start..self.pos].to_string()))
    }

    fn peek_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn advance(&mut self) {
        if self.is_at_end() {
            return;
        }

        let ch = self.peek_char();
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }
}

fn is_atom_char(ch: char) -> bool {
    !(ch.is_whitespace() || ch == ';' || ch == '{' || ch == '}' || ch == '#')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(input: &str) -> Vec<TokenKind> {
        Lexer::new(input)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn tokenizes_directive() {
        assert_eq!(
            kinds("worker_processes 4;"),
            vec![TokenKind::Ident, TokenKind::Ident, TokenKind::Semicolon]
        );
    }

    #[test]
    fn tokenizes_block() {
        assert_eq!(
            kinds("server {\n    listen 80;\n}"),
            vec![
                TokenKind::Ident,
                TokenKind::LBrace,
                TokenKind::Newline,
                TokenKind::Ident,
                TokenKind::Ident,
                TokenKind::Semicolon,
                TokenKind::Newline,
                TokenKind::RBrace
            ]
        );
    }

    #[test]
    fn tokenizes_comments_and_strings() {
        assert_eq!(
            kinds(r#"root "/var/www"; # inline"#),
            vec![
                TokenKind::Ident,
                TokenKind::String,
                TokenKind::Semicolon,
                TokenKind::Comment
            ]
        );
    }

    #[test]
    fn tokenizes_regex_location() {
        let tokens = Lexer::new("location ~ ^/api/.* { }").tokenize().unwrap();
        let texts: Vec<_> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(texts.contains(&"location"));
        assert!(texts.contains(&"~"));
        assert!(texts.contains(&"^/api/.*"));
    }
}
