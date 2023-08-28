use super::{*, lexer::*};
use logos::Span;

pub struct AST {
    pub instructions: Vec<Instruction>
}

pub type _Token = (Token, Span);

pub struct Parser<'a> {
    pub tokens: Vec<_Token>,
    pub index : usize,
    pub ast   : &'a mut AST
}

impl<'a> Parser<'a> {
    pub fn new(t: Vec<_Token>, ast: &'a mut AST) -> Self {
        Self {
            tokens: t,
            index : 0,
            ast,
        }
    }

    fn next<'a>(&'a mut self) -> Option<&'a _Token> {
        self.index += 1;
        self.tokens.get(self.index-1)
    }

    fn peek<'a>(&'a mut self) -> Option<&'a _Token> {
        self.tokens.get(self.index+1)
    }
}
