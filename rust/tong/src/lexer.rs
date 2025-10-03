use anyhow::{anyhow, Result};
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum TokenKind {
    #[regex(r"[ \t\r\n]+", logos::skip)]
    Whitespace,
    #[regex(r"//[^\n]*", logos::skip)]
    LineComment,

    #[token("let")]
    Let,
    #[token("var")]
    Var,
    #[token("fn")]
    Fn,
    #[token("def")]
    Def,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("if")]
    If,
    #[token("return")]
    Return,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("parallel")]
    Parallel,
    #[token("data")]
    Data,
    #[token("match")]
    Match,
    #[token("in")]
    In,
    // Strings: naive double-quoted without escapes handling for MVP
    #[regex(r#""[^"\n]*""#)]
    String,
    #[regex(r"[0-9]+\.[0-9]+")]
    Float,
    #[regex(r"[0-9]+")]
    Int,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("||")]
    OrOr,
    #[token("|")]
    Pipe,
    #[token("&")]
    Ampersand,
    #[token("->")]
    Arrow,
    #[token("\\")]
    Backslash,
    #[token("==")]
    EqualEqual,
    #[token("!=")]
    BangEqual,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("=")]
    Equal,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    #[error]
    Error,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    // Span fields are kept for potential future detailed diagnostics.
    #[allow(dead_code)]
    pub start: usize,
    #[allow(dead_code)]
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

pub fn lex(input: &str) -> Result<Vec<Token>> {
    let mut lex = TokenKind::lexer(input);
    // Precompute line starts for line/col mapping
    let mut line_starts: Vec<usize> = vec![0];
    for (i, ch) in input.char_indices() {
        if ch == '\n' {
            line_starts.push(i + 1);
        }
    }
    let find_line_col = |start: usize| -> (usize, usize) {
        // binary search for greatest index with line_starts[idx] <= start
        let mut lo = 0usize;
        let mut hi = line_starts.len();
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            if line_starts[mid] <= start {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let line = lo + 1; // 1-based
        let col = start - line_starts[lo] + 1; // 1-based
        (line, col)
    };
    let mut tokens = Vec::new();
    while let Some(kind) = lex.next() {
        let text = lex.slice().to_string();
        let span = lex.span();
        let (line, col) = find_line_col(span.start);
        if matches!(kind, TokenKind::Whitespace) {
            continue;
        }
        if matches!(kind, TokenKind::Error) {
            return Err(anyhow!("lex error at {}:{} near '{}'", line, col, text));
        }
        tokens.push(Token {
            kind,
            text,
            start: span.start,
            end: span.end,
            line,
            col,
        });
    }
    Ok(tokens)
}
