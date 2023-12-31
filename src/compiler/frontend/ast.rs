use crate::compiler::{error::*, common::*, frontend::lexer::*};
use logos::Span;

#[derive(Debug)]
pub struct Ast {
    pub instructions: Vec<(Instruction, Span)>,
    pub dw: Vec<u64>,
    pub minheap: usize,
    pub minstack: usize,
    pub minreg: usize,
    pub bits: usize,
}

impl Ast {
    pub const fn new() -> Self {
        Self {
            instructions: Vec::new(),
            dw: Vec::new(),
            minheap: 16,
            minstack: 8,
            minreg: 8,
            bits: 8,
        }
    }
}

pub type AToken = (Token, Span);

#[derive(Debug)]
pub struct Parser {
    pub tokens: Vec<AToken>,
    pub index : usize,
    pub ast   : Ast
}

impl Parser {
    pub fn new(lex: &mut Lexer<Token>) -> Result<Self, Vec<Error<LexerError>>> {
        let mut t = Vec::new();
        let mut err = Vec::new();

        while let Some(tok) = lex.next() {
            tok.map_or_else(
                |_| err.push(Error { kind: LexerError, span: lex.span() }),
                |tok| t.push((tok, lex.span()))
            );
        }

        t.push((Token::Newline, t.last().map_or(Span { start: 0, end: 1 }, |a| Span { start: a.1.end - 1, end: a.1.end })));

        if !err.is_empty() {
            Err(err)
        } else {
            Ok(Self {
                tokens: t,
                index : 0,
                ast: Ast::new(),
            })
        }
    }

    #[allow(clippy::should_implement_trait)]
    // this can't be turned into an iterator without cloning
    pub fn next(&mut self) -> Option<&AToken> {
        self.index += 1;
        self.current()
    }

    pub fn current(&mut self) -> Option<&AToken> {
        self.tokens.get(self.index-1)
    }
    /*
    pub fn peek(&mut self) -> Option<&AToken> {
        self.tokens.get(self.index+1)
    } */
}
