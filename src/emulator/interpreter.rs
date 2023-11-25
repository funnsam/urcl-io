use super::{*, ast::*, error::*};

#[derive(Debug)]
pub struct Interpreter {
    ast: Ast,

    pc: usize,
    sp: usize,

    ram: Vec<u64>,
}

impl Interpreter {
    pub fn new(ast: Ast) -> Self {
        let ram_size = ast.dw.len() + ast.minheap + ast.minstack;
        Self {
            ast,
            pc: 0,
            sp: 0,
            ram: vec![0; ram_size]
        }
    }

    pub fn step(&mut self) -> StepResult {
        StepResult::Halted
    }
}

#[derive(Debug)]
pub enum StepResult {
    Error(Error<InterpreterError>),
    Running,
    Halted,
}
