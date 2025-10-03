use anyhow::{anyhow, bail, Result};

use crate::lexer::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum Expr {
    Str(String),
    Float(f64),
    Int(i64),
    Bool(bool),
    Ident(String),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Array(Vec<Expr>),
    Call {
        callee: String,
        args: Vec<Expr>,
    },
    ConstructorCall {
        name: String,
        args: Vec<Expr>,
    },
    MethodCall {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    Property {
        target: Box<Expr>,
        name: String,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Index(Box<Expr>, Box<Expr>),
    UnaryNeg(Box<Expr>),
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<(Pattern, Option<Expr>, Expr)>, // pattern, optional guard, result
    },
    ListComp {
        elem: Box<Expr>,
        generators: Vec<(String, Expr)>, // (var, list_expr) pairs, evaluated left-to-right
        pred: Option<Box<Expr>>,         // optional predicate applied after all bindings
    },
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Ident(String),
    Int(i64),
    Bool(bool),
    Constructor {
        name: String,
        arity: usize,
        sub: Vec<Pattern>,
    },
    Tuple(Vec<Pattern>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr),
    LetTuple(Vec<String>, Expr),
    Assign(String, Expr),
    Print(Vec<Expr>),
    FnMain(Vec<Stmt>),
    FnDef(String, Vec<String>, Vec<Stmt>),
    FnDefGuarded(String, Vec<String>, Expr, Vec<Stmt>),
    FnDefPattern(String, Vec<Pattern>, Option<Expr>, Vec<Stmt>), // pattern parameters with optional guard
    Import(String, String),
    Return(Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    Parallel(Vec<Stmt>),
    Expr(Expr),
    #[allow(dead_code)] // type name currently only used for display/planned type passes
    DataDecl(String, Vec<Constructor>),
}

#[derive(Debug, Clone)]
pub struct Constructor {
    pub name: String,
    pub arity: usize,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

pub fn parse(tokens: Vec<Token>) -> Result<Program> {
    let mut p = Parser { tokens, pos: 0, known_ctors: std::collections::HashMap::new() };
    p.parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    known_ctors: std::collections::HashMap<String, usize>, // constructor name -> arity
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }
    fn eat(&mut self, kind: TokenKind) -> Result<Token> {
        if let Some(t) = self.peek() {
            if t.kind == kind {
                return Ok(self.bump().unwrap());
            }
        }
        let where_ = self
            .peek()
            .map(|t| format!(" at line {}, col {}", t.line, t.col))
            .unwrap_or_default();
        Err(anyhow!("expected {:?}{}", kind, where_))
    }
    fn peek_n_is(&self, n: usize, k: TokenKind) -> bool {
        self.tokens.get(self.pos + n).map(|t| t.kind.clone()) == Some(k)
    }

    fn parse_program(&mut self) -> Result<Program> {
        let mut stmts = Vec::new();
        while self.peek().is_some() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program { stmts })
    }

    fn peek_is(&self, k: TokenKind) -> bool {
        self.peek().map(|t| t.kind.clone()) == Some(k)
    }

    fn parse_fn(&mut self) -> Result<Stmt> {
        if self.peek_is(TokenKind::Fn) {
            self.eat(TokenKind::Fn)?;
        } else {
            self.eat(TokenKind::Def)?;
        }
        let name = self.eat_ident()?;
        self.eat(TokenKind::LParen)?;
        // Capture raw parameter token spans to decide between simple identifiers and patterns.
        let mut raw_params: Vec<(usize, usize)> = Vec::new();
        if !self.peek_is(TokenKind::RParen) {
            loop {
                let start = self.pos;
                let mut depth = 0usize;
                while let Some(tok) = self.peek() {
                    if depth == 0 && (tok.kind == TokenKind::Comma || tok.kind == TokenKind::RParen)
                    {
                        break;
                    }
                    match tok.kind {
                        TokenKind::LParen => {
                            depth += 1;
                            self.bump();
                        }
                        TokenKind::RParen => {
                            if depth == 0 {
                                break;
                            }
                            depth -= 1;
                            self.bump();
                        }
                        _ => {
                            self.bump();
                        }
                    }
                }
                raw_params.push((start, self.pos));
                if self.peek_is(TokenKind::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
        }
        self.eat(TokenKind::RParen)?;
        let mut simple_idents: Vec<String> = Vec::new();
        let mut patterns: Vec<Pattern> = Vec::new();
        let mut all_simple = true;
        for (s, e) in &raw_params {
            if s == e {
                continue;
            }
            if *e - *s == 1 {
                if let Some(tok) = self.tokens.get(*s) {
                    if tok.kind == TokenKind::Ident && tok.text != "_" {
                        // Prefer semantic detection: is token a known zero-arity constructor? Fallback heuristic.
                        let text = &tok.text;
                        let semantic_ctor = self
                            .known_ctors
                            .get(text)
                            .copied()
                            .unwrap_or(usize::MAX)
                            == 0;
                        let heuristic_ctor = text.len() > 1
                            && text
                                .chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false);
                        let is_ctor_like = semantic_ctor || heuristic_ctor;
                        if is_ctor_like {
                            all_simple = false; // force pattern function path
                            patterns.push(Pattern::Constructor {
                                name: tok.text.clone(),
                                arity: 0,
                                sub: Vec::new(),
                            });
                            continue;
                        } else {
                            simple_idents.push(tok.text.clone());
                            patterns.push(Pattern::Ident(tok.text.clone()));
                            continue;
                        }
                    }
                }
            }
            // Complex or non-simple param: parse as pattern
            all_simple = false;
            let slice = self.tokens[*s..*e].to_vec();
            let mut sub = Parser {
                tokens: slice,
                pos: 0,
                known_ctors: self.known_ctors.clone(),
            };
            let pat = sub.parse_pattern()?; // reuse pattern parser
            patterns.push(pat);
        }
        // Optional guard
        let guard_expr = if self.peek_is(TokenKind::If) {
            self.bump();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.eat(TokenKind::LBrace)?;
        let mut body = Vec::new();
        while !self.peek_is(TokenKind::RBrace) {
            body.push(self.parse_stmt()?);
        }
        self.eat(TokenKind::RBrace)?;
        if name == "main" && all_simple && simple_idents.is_empty() {
            return Ok(Stmt::FnMain(body));
        }
        if all_simple {
            if let Some(g) = guard_expr {
                return Ok(Stmt::FnDefGuarded(name, simple_idents, g, body));
            }
            return Ok(Stmt::FnDef(name, simple_idents, body));
        }
        Ok(Stmt::FnDefPattern(name, patterns, guard_expr, body))
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        if self.peek_is(TokenKind::Data) {
            self.bump();
            let type_name = self.eat_ident()?;
            if !self.peek_is(TokenKind::Equal) {
                bail!("expected '=' after type name in data declaration");
            }
            self.eat(TokenKind::Equal)?;
            // constructors separated by '|', each name optionally followed by arity counted via Ident placeholders (simplified)
            let mut ctors = Vec::new();
            loop {
                let cname = self.eat_ident()?;
                // Count following Ident parameters until '|' or end-of-stmt token (reuse Else/While/Fn/Let/Var/If/Data/Return/Parallel as boundaries or EOF)
                let mut arity = 0usize;
                while self.peek_is(TokenKind::Ident) {
                    // lookahead stop if next is '=' which would start assignment, but for simplicity treat consecutive identifiers as params
                    arity += 1;
                    self.bump();
                }
                self.known_ctors.insert(cname.clone(), arity);
                ctors.push(Constructor { name: cname, arity });
                if self.peek_is(TokenKind::Pipe) {
                    self.bump();
                    continue;
                }
                break;
            }
            return Ok(Stmt::DataDecl(type_name, ctors));
        }
        // Desugar `let x = import("sdl")` into a simple Assign to a special Import expr
        if self.peek_is(TokenKind::Let) || self.peek_is(TokenKind::Var) {
            self.bump();
            // Tuple destructuring: let (a,b,c) = expr
            if self.peek_is(TokenKind::LParen) {
                self.eat(TokenKind::LParen)?;
                let mut names = Vec::new();
                if !self.peek_is(TokenKind::RParen) {
                    names.push(self.eat_ident()?);
                    while self.peek_is(TokenKind::Comma) {
                        self.bump();
                        names.push(self.eat_ident()?);
                    }
                }
                self.eat(TokenKind::RParen)?;
                self.eat(TokenKind::Equal)?;
                let value = self.parse_expr()?;
                return Ok(Stmt::LetTuple(names, value));
            }
            let name = self.eat_ident()?;
            self.eat(TokenKind::Equal)?;
            // Detect import("module")
            if self.peek_is(TokenKind::Ident) && self.peek_text() == "import" {
                self.bump();
                self.eat(TokenKind::LParen)?;
                let module_tok = self.eat(TokenKind::String)?;
                let module = module_tok.text.trim_matches('"').to_string();
                self.eat(TokenKind::RParen)?;
                return Ok(Stmt::Import(name, module));
            }
            let value = self.parse_expr()?;
            return Ok(Stmt::Let(name, value));
        }
        if self.peek_is(TokenKind::Fn) || self.peek_is(TokenKind::Def) {
            return self.parse_fn();
        }
        if self.peek_is(TokenKind::Parallel) {
            self.bump();
            self.eat(TokenKind::LBrace)?;
            let mut body = Vec::new();
            while !self.peek_is(TokenKind::RBrace) {
                body.push(self.parse_stmt()?);
            }
            self.eat(TokenKind::RBrace)?;
            return Ok(Stmt::Parallel(body));
        }
        if self.peek_is(TokenKind::While) {
            self.bump();
            let cond = self.parse_expr()?;
            self.eat(TokenKind::LBrace)?;
            let mut body = Vec::new();
            while !self.peek_is(TokenKind::RBrace) {
                body.push(self.parse_stmt()?);
            }
            self.eat(TokenKind::RBrace)?;
            return Ok(Stmt::While(cond, body));
        }
        if self.peek_is(TokenKind::Return) {
            self.bump();
            let e = self.parse_expr()?;
            return Ok(Stmt::Return(e));
        }
        if self.peek_is(TokenKind::If) {
            self.bump();
            let cond = self.parse_expr()?;
            self.eat(TokenKind::LBrace)?;
            let mut body = Vec::new();
            while !self.peek_is(TokenKind::RBrace) {
                body.push(self.parse_stmt()?);
            }
            self.eat(TokenKind::RBrace)?;
            let else_body = if self.peek_is(TokenKind::Else) {
                self.bump();
                self.eat(TokenKind::LBrace)?;
                let mut ebody = Vec::new();
                while !self.peek_is(TokenKind::RBrace) {
                    ebody.push(self.parse_stmt()?);
                }
                self.eat(TokenKind::RBrace)?;
                Some(ebody)
            } else {
                None
            };
            return Ok(Stmt::If(cond, body, else_body));
        }
        if self.peek_is(TokenKind::Ident) && self.peek_text() == "print" {
            self.bump(); // print
            self.eat(TokenKind::LParen)?;
            let mut args = Vec::new();
            if !self.peek_is(TokenKind::RParen) {
                args.push(self.parse_expr()?);
                while self.peek_is(TokenKind::Comma) {
                    self.bump();
                    args.push(self.parse_expr()?);
                }
            }
            self.eat(TokenKind::RParen)?;
            Ok(Stmt::Print(args))
        } else if self.peek_is(TokenKind::Ident) && self.peek_n_is(1, TokenKind::Equal) {
            // assignment
            let name = self.eat_ident()?;
            self.eat(TokenKind::Equal)?;
            let value = self.parse_expr()?;
            Ok(Stmt::Assign(name, value))
        } else {
            // expression statement (covers calls, method calls, property chains, etc.)
            let e = self.parse_expr()?;
            Ok(Stmt::Expr(e))
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_comparison()
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut node = self.parse_additive()?;
        loop {
            if self.peek_is(TokenKind::EqualEqual) {
                self.bump();
                let rhs = self.parse_additive()?;
                node = Expr::Binary {
                    op: BinOp::Eq,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else if self.peek_is(TokenKind::BangEqual) {
                self.bump();
                let rhs = self.parse_additive()?;
                node = Expr::Binary {
                    op: BinOp::Ne,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut node = self.parse_equality()?;
        loop {
            if self.peek_is(TokenKind::Less) {
                self.bump();
                let rhs = self.parse_equality()?;
                node = Expr::Binary {
                    op: BinOp::Lt,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else if self.peek_is(TokenKind::LessEqual) {
                self.bump();
                let rhs = self.parse_equality()?;
                node = Expr::Binary {
                    op: BinOp::Le,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else if self.peek_is(TokenKind::Greater) {
                self.bump();
                let rhs = self.parse_equality()?;
                node = Expr::Binary {
                    op: BinOp::Gt,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else if self.peek_is(TokenKind::GreaterEqual) {
                self.bump();
                let rhs = self.parse_equality()?;
                node = Expr::Binary {
                    op: BinOp::Ge,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut node = self.parse_multiplicative()?;
        loop {
            if self.peek_is(TokenKind::Plus) {
                self.bump();
                let rhs = self.parse_multiplicative()?;
                node = Expr::Binary {
                    op: BinOp::Add,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else if self.peek_is(TokenKind::Minus) {
                self.bump();
                let rhs = self.parse_multiplicative()?;
                node = Expr::Binary {
                    op: BinOp::Sub,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut node = self.parse_unary()?;
        loop {
            if self.peek_is(TokenKind::Star) {
                self.bump();
                let rhs = self.parse_unary()?;
                node = Expr::Binary {
                    op: BinOp::Mul,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else if self.peek_is(TokenKind::Slash) {
                self.bump();
                let rhs = self.parse_unary()?;
                node = Expr::Binary {
                    op: BinOp::Div,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else if self.peek_is(TokenKind::Percent) {
                self.bump();
                let rhs = self.parse_unary()?;
                node = Expr::Binary {
                    op: BinOp::Mod,
                    left: Box::new(node),
                    right: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.peek_is(TokenKind::Minus) {
            self.bump();
            let e = self.parse_unary()?;
            return Ok(Expr::UnaryNeg(Box::new(e)));
        }
        if self.peek_is(TokenKind::Plus) {
            self.bump();
            return self.parse_unary();
        }
        self.parse_primary()
    }

    fn parse_atom(&mut self) -> Result<Expr> {
        // Backslash lambda: \x y -> expr
        if self.peek_is(TokenKind::Backslash) {
            self.bump();
            let mut params = Vec::new();
            while self.peek_is(TokenKind::Ident) {
                params.push(self.eat_ident()?);
            }
            self.eat(TokenKind::Arrow)?;
            let body = self.parse_expr()?;
            return Ok(Expr::Lambda {
                params,
                body: Box::new(body),
            });
        }
        // Lambda: |x| expr
        if self.peek_is(TokenKind::Pipe) {
            self.eat(TokenKind::Pipe)?;
            let params = vec![self.eat_ident()?];
            self.eat(TokenKind::Pipe)?;
            let body = self.parse_expr()?;
            return Ok(Expr::Lambda {
                params,
                body: Box::new(body),
            });
        }
        if self.peek_is(TokenKind::LParen) {
            self.eat(TokenKind::LParen)?;
            let e = self.parse_expr()?;
            self.eat(TokenKind::RParen)?;
            return Ok(e);
        }
        if self.peek_is(TokenKind::Match) {
            self.bump();
            let scrut = self.parse_expr()?;
            self.eat(TokenKind::LBrace)?;
            let mut arms = Vec::new();
            while !self.peek_is(TokenKind::RBrace) {
                let pat = self.parse_pattern()?;
                let guard = if self.peek_is(TokenKind::If) {
                    self.bump();
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.eat(TokenKind::Arrow)?;
                let body = self.parse_expr()?;
                if self.peek_is(TokenKind::Comma) {
                    self.bump();
                }
                arms.push((pat, guard, body));
            }
            self.eat(TokenKind::RBrace)?;
            return Ok(Expr::Match {
                scrutinee: Box::new(scrut),
                arms,
            });
        }
        if self.peek_is(TokenKind::String) {
            Ok(Expr::Str(
                self.bump().unwrap().text.trim_matches('"').to_string(),
            ))
        } else if self.peek_is(TokenKind::Float) {
            Ok(Expr::Float(self.bump().unwrap().text.parse()?))
        } else if self.peek_is(TokenKind::Int) {
            Ok(Expr::Int(self.bump().unwrap().text.parse()?))
        } else if self.peek_is(TokenKind::True) {
            self.bump();
            Ok(Expr::Bool(true))
        } else if self.peek_is(TokenKind::False) {
            self.bump();
            Ok(Expr::Bool(false))
        } else if self.peek_is(TokenKind::LBracket) {
            self.eat(TokenKind::LBracket)?;
            // Detect list comprehension: [ expr | ident in expr (if expr)? ]
            if !self.peek_is(TokenKind::RBracket) {
                let first = self.parse_expr()?;
                if self.peek_is(TokenKind::Pipe) {
                    self.bump();
                    // Parse one or more generators: x in xs, y in ys, ...
                    let mut gens: Vec<(String, Expr)> = Vec::new();
                    loop {
                        let var = self.eat_ident()?;
                        if !self.peek_is(TokenKind::In) {
                            bail!("expected 'in' in list comprehension");
                        }
                        self.bump();
                        let list_expr = self.parse_expr()?;
                        gens.push((var, list_expr));
                        if self.peek_is(TokenKind::Comma) {
                            // Lookahead: if next after comma starts an Ident then keep parsing more gens; otherwise break (will treat as array elements)
                            self.bump();
                            // If next token is Ident and followed by 'in', continue parsing another generator
                            if self.peek_is(TokenKind::Ident) && self.peek_n_is(1, TokenKind::In) {
                                continue;
                            } else {
                                bail!("unexpected comma in list comprehension generators; expected another '<ident> in <expr>'");
                            }
                        } else {
                            break;
                        }
                    }
                    let pred = if self.peek_is(TokenKind::If) {
                        self.bump();
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    self.eat(TokenKind::RBracket)?;
                    Ok(Expr::ListComp {
                        elem: Box::new(first),
                        generators: gens,
                        pred,
                    })
                } else {
                    let mut elems = vec![first];
                    while self.peek_is(TokenKind::Comma) {
                        self.bump();
                        elems.push(self.parse_expr()?);
                    }
                    self.eat(TokenKind::RBracket)?;
                    Ok(Expr::Array(elems))
                }
            } else {
                self.eat(TokenKind::RBracket)?;
                Ok(Expr::Array(Vec::new()))
            }
        } else if self.peek_is(TokenKind::Ident) {
            // Could be name, function call, or method chain like sdl.create_renderer(win)
            let mut node = Expr::Ident(self.bump().unwrap().text);
            // function call on identifier
            if self.peek_is(TokenKind::LParen) {
                // Distinguish constructor vs function using semantic table (preferred) then capitalization
                let name = match &node {
                    Expr::Ident(n) => n.clone(),
                    _ => String::new(),
                };
                let is_ctor_like = if let Some(_a) = self.known_ctors.get(&name) {
                    true
                } else {
                    name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                };
                self.eat(TokenKind::LParen)?;
                let mut args = Vec::new();
                if !self.peek_is(TokenKind::RParen) {
                    args.push(self.parse_expr()?);
                    while self.peek_is(TokenKind::Comma) {
                        self.bump();
                        args.push(self.parse_expr()?);
                    }
                }
                self.eat(TokenKind::RParen)?;
                if is_ctor_like {
                    node = Expr::ConstructorCall { name, args };
                } else {
                    node = Expr::Call { callee: name, args };
                }
            }
            // property/method chaining: .name or .method(args)
            while self.peek_is(TokenKind::Dot) {
                self.eat(TokenKind::Dot)?;
                let name = self.eat_ident()?;
                if self.peek_is(TokenKind::LParen) {
                    self.eat(TokenKind::LParen)?;
                    let mut args = Vec::new();
                    if !self.peek_is(TokenKind::RParen) {
                        args.push(self.parse_expr()?);
                        while self.peek_is(TokenKind::Comma) {
                            self.bump();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.eat(TokenKind::RParen)?;
                    node = Expr::MethodCall {
                        target: Box::new(node),
                        method: name,
                        args,
                    };
                } else {
                    node = Expr::Property {
                        target: Box::new(node),
                        name,
                    };
                }
            }
            Ok(node)
        } else if let Some(t) = self.peek() {
            bail!(
                "unexpected token {:?} '{}' at {}:{}",
                t.kind,
                t.text,
                t.line,
                t.col
            );
        } else {
            Err(anyhow!("unexpected end of input"))?
        }
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let mut node = self.parse_atom()?;
        while self.peek_is(TokenKind::LBracket) {
            self.eat(TokenKind::LBracket)?;
            let idx = self.parse_expr()?;
            self.eat(TokenKind::RBracket)?;
            node = Expr::Index(Box::new(node), Box::new(idx));
        }
        Ok(node)
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        if self.peek_is(TokenKind::Ident) {
            let name = self.eat_ident()?;
            if name == "_" {
                return Ok(Pattern::Wildcard);
            }
            let is_ctor = name
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false);
            if is_ctor {
                // Parenthesized constructor pattern arguments: Ctor(p1, p2, ...)
                if self.peek_is(TokenKind::LParen) {
                    self.eat(TokenKind::LParen)?;
                    let mut subs = Vec::new();
                    if !self.peek_is(TokenKind::RParen) {
                        subs.push(self.parse_pattern()?);
                        while self.peek_is(TokenKind::Comma) {
                            self.bump();
                            subs.push(self.parse_pattern()?);
                        }
                    }
                    self.eat(TokenKind::RParen)?;
                    return Ok(Pattern::Constructor {
                        name,
                        arity: subs.len(),
                        sub: subs,
                    });
                }
                // Fallback: space-separated simple subpatterns (kept for backward compatibility: Just x, Node left right)
                let mut subs = Vec::new();
                loop {
                    if self.peek_is(TokenKind::Arrow)
                        || self.peek_is(TokenKind::Comma)
                        || self.peek_is(TokenKind::If)
                        || self.peek_is(TokenKind::Pipe)
                        || self.peek_is(TokenKind::RBrace)
                        || self.peek_is(TokenKind::RParen)
                    {
                        break;
                    }
                    let starts = self.peek_is(TokenKind::Ident)
                        || self.peek_is(TokenKind::Int)
                        || self.peek_is(TokenKind::True)
                        || self.peek_is(TokenKind::False)
                        || self.peek_is(TokenKind::LParen);
                    if !starts {
                        break;
                    }
                    if self.peek_is(TokenKind::Ident) && self.peek_text() == "_" {
                        self.bump();
                        subs.push(Pattern::Wildcard);
                    } else {
                        subs.push(self.parse_pattern()?);
                    }
                }
                return Ok(Pattern::Constructor {
                    name,
                    arity: subs.len(),
                    sub: subs,
                });
            }
            return Ok(Pattern::Ident(name));
        }
        if self.peek_is(TokenKind::LParen) {
            self.eat(TokenKind::LParen)?;
            let mut subs = Vec::new();
            if !self.peek_is(TokenKind::RParen) {
                subs.push(self.parse_pattern()?);
                while self.peek_is(TokenKind::Comma) {
                    self.bump();
                    subs.push(self.parse_pattern()?);
                }
            }
            self.eat(TokenKind::RParen)?;
            return Ok(Pattern::Tuple(subs));
        }
        if self.peek_is(TokenKind::Int) {
            return Ok(Pattern::Int(self.bump().unwrap().text.parse()?));
        }
        if self.peek_is(TokenKind::True) {
            self.bump();
            return Ok(Pattern::Bool(true));
        }
        if self.peek_is(TokenKind::False) {
            self.bump();
            return Ok(Pattern::Bool(false));
        }
        bail!("unsupported pattern");
    }

    fn eat_ident(&mut self) -> Result<String> {
        if self.peek_is(TokenKind::Ident) {
            Ok(self.bump().unwrap().text)
        } else if let Some(t) = self.peek() {
            bail!("expected identifier at {}:{}", t.line, t.col);
        } else {
            bail!("expected identifier at end of input");
        }
    }

    fn peek_text(&self) -> String {
        self.peek().map(|t| t.text.clone()).unwrap_or_default()
    }
}
