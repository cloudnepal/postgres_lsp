mod common;
mod data;
mod dml;

pub use common::source;

use pg_lexer::{lex, SyntaxKind, Token, WHITESPACE_TOKENS};
use text_size::{TextRange, TextSize};

use crate::syntax_error::SyntaxError;

/// Main parser that exposes the `cstree` api, and collects errors and statements
/// It is modelled after a Pratt Parser. For a gentle introduction to Pratt Parsing, see https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
pub struct Parser {
    /// The ranges of the statements
    ranges: Vec<(usize, usize)>,
    /// The syntax errors accumulated during parsing
    errors: Vec<SyntaxError>,
    /// The start of the current statement, if any
    current_stmt_start: Option<usize>,
    /// The tokens to parse
    pub tokens: Vec<Token>,

    eof_token: Token,

    next_pos: usize,
}

/// Result of Building
#[derive(Debug)]
pub struct Parse {
    /// The ranges of the errors
    pub ranges: Vec<TextRange>,
    /// The syntax errors accumulated during parsing
    pub errors: Vec<SyntaxError>,
}

impl Parser {
    pub fn new(sql: &str) -> Self {
        // we dont care about whitespace tokens, except for double newlines
        // to make everything simpler, we just filter them out
        // the token holds the text range, so we dont need to worry about that
        let tokens = lex(sql)
            .iter()
            .filter(|t| {
                return !WHITESPACE_TOKENS.contains(&t.kind)
                    || (t.kind == SyntaxKind::Newline && t.text.chars().count() > 1);
            })
            .cloned()
            .collect::<Vec<_>>();

        let eof_token = Token::eof(usize::from(
            tokens
                .last()
                .map(|t| t.span.start())
                .unwrap_or(TextSize::from(0)),
        ));

        // next_pos should be the initialised with the first valid token already
        let mut next_pos = 0;
        loop {
            let token = tokens.get(next_pos).unwrap_or(&eof_token);

            if is_irrelevant_token(token) {
                next_pos += 1;
            } else {
                break;
            }
        }

        Self {
            ranges: Vec::new(),
            eof_token,
            errors: Vec::new(),
            current_stmt_start: None,
            tokens,
            next_pos,
        }
    }

    pub fn finish(self) -> Parse {
        Parse {
            ranges: self
                .ranges
                .iter()
                .map(|(start, end)| {
                    println!("{} {}", start, end);
                    let from = self.tokens.get(*start);
                    let to = self.tokens.get(*end).unwrap_or(&self.eof_token);

                    TextRange::new(from.unwrap().span.start(), to.span.end())
                })
                .collect(),
            errors: self.errors,
        }
    }

    /// Start statement
    pub fn start_stmt(&mut self) {
        assert!(self.current_stmt_start.is_none());
        self.current_stmt_start = Some(self.next_pos);
    }

    /// Close statement
    pub fn close_stmt(&mut self) {
        assert!(self.next_pos > 0);

        self.ranges.push((
            self.current_stmt_start.expect("Expected active statement"),
            self.next_pos - 1,
        ));

        self.current_stmt_start = None;
    }

    fn advance(&mut self) -> &Token {
        let mut first_relevant_token = None;
        loop {
            let token = self.tokens.get(self.next_pos).unwrap_or(&self.eof_token);

            // we need to continue with next_pos until the next relevant token after we already
            // found the first one
            if !is_irrelevant_token(token) {
                if let Some(t) = first_relevant_token {
                    return t;
                }
                first_relevant_token = Some(token);
            }

            self.next_pos += 1;
        }
    }

    fn peek(&self) -> &Token {
        match self.tokens.get(self.next_pos) {
            Some(token) => token,
            None => &self.eof_token,
        }
    }

    /// checks if the current token is of `kind` and advances if true
    /// returns true if the current token is of `kind`
    pub fn eat(&mut self, kind: SyntaxKind) -> bool {
        if self.peek().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, kind: SyntaxKind) {
        if self.eat(kind) {
            return;
        }

        self.error_at(format!("Expected {:#?}", kind));
    }

    /// collects an SyntaxError with an `error` message at the current position
    fn error_at(&mut self, error: String) {
        todo!();
    }
}

fn is_irrelevant_token(t: &Token) -> bool {
    return WHITESPACE_TOKENS.contains(&t.kind)
        && (t.kind != SyntaxKind::Newline || t.text.chars().count() == 1);
}
