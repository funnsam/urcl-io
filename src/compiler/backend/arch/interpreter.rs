use crate::compiler::{error::*, common::*, backend::ssa::*};
use std::{collections::HashMap, io::{Read, Write}};
use derivative::Derivative;
use logos::Span;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Interpreter {
    #[derivative(Debug="ignore")]
    ssa: Body,

    block_id: BlockId,
    instr_id: usize,
    block_id_old: BlockId,

    pub inst_count: usize,

    values: Vec<u64>,
    variables: Vec<Option<Vec<u64>>>,
}

impl Interpreter {
    pub fn new(ssa: (Body, usize, usize)) -> Self {
        Self {
            ssa: ssa.0,

            block_id: BlockId(0),
            instr_id: 0,
            block_id_old: BlockId(0),

            inst_count: 0,

            values: vec![0; ssa.1],
            variables: vec![None; ssa.2],
        }
    }

    pub fn step(&mut self, stdout: &mut impl Write, stdin: &mut impl Read) -> StepResult {
        let block = self.ssa.blocks.get(*self.block_id).unwrap();
        let instr = block.instructions.get(self.instr_id);

        macro_rules! get {
            (val $a: expr) => {
                *self.values.get($a.0).unwrap()
            };
        }

        let val = match instr.map(|a| &a.operation) {
            Some(Operation::Integer(imm)) => Some(*imm),
            Some(Operation::LoadIndex(var, off)) => Some(*self.variables.get(var.0).unwrap().as_ref().unwrap().get(get!(val off) as usize).unwrap()),
            Some(Operation::StoreIndex(var, off, dat)) => {
                *self.variables.get_mut(var.0).unwrap().as_mut().unwrap().get_mut(get!(val off) as usize).unwrap() = get!(val dat);
                None
            },
            Some(Operation::Allocate(var, siz)) => {
                self.variables[var.0] = Some(vec![0; get!(val siz) as usize]);
                None
            },
            Some(Operation::BinOp(op, l, r)) => Some(op.operate(get!(val l), get!(val r))),
            Some(Operation::Call(f, arg)) => match f {
                Function::PortWrite => {
                    let p = get!(val arg[0]);
                    let d = get!(val arg[1]);
                    let _ = unsafe { std::mem::transmute::<&Self, &mut Self>(self) }.port_write(p, d, stdout, stdin);
                    None
                },
                Function::PortRead => {
                    let p = get!(val arg[0]);
                    Some(unsafe { std::mem::transmute::<&Self, &mut Self>(self) }.port_read(p, stdout, stdin).unwrap())
                },
                _ => todo!()
            },
            None => None,
            _ => todo!(),
        };

        if let Some(Some(dest)) = instr.map(|a| a.destination) {
            self.values[dest.0] = val.unwrap();
        }

        self.instr_id += 1;
        if self.instr_id >= block.instructions.len() {
            self.inst_count += 1;
            match &block.terminator {
                Terminator::Jump(blk) => {
                    self.block_id_old = self.block_id;
                    self.block_id = blk.clone();
                },
                Terminator::Branch(cond, if_, else_) => {
                    self.block_id_old = self.block_id;
                    if get!(val cond) != 0 {
                        self.block_id = if_.clone();
                    } else {
                        self.block_id = else_.clone();
                    }
                },
                Terminator::Return => return StepResult::Halted,
                _ => todo!(),
            }

            self.instr_id = 0;
        }

        StepResult::Running
    }

    pub fn port_write(&mut self, port: u64, data: u64, stdout: &mut impl Write, _stdin: &mut impl Read) -> Result<(), InterpreterError> {
        match unsafe { std::mem::transmute(port) } {
            Port::Text => write!(stdout, "{}", unsafe { char::from_u32_unchecked(data as u32) }).unwrap(),
            Port::Number => write!(stdout, "{data}").unwrap(),

            /*Port::Profile => {
                self.debugging = data & 1 != 0;
                println!("\x1b[1;32mInterpreter:\x1b[0m debugging is now {}", if self.debugging { "enabled" } else { "disabled" });
            },*/
            _ => return Err(InterpreterError::UnsupportedPort(port)),
        }

        Ok(())
    }

    pub fn port_read(&mut self, port: u64, _stdout: &mut impl Write, stdin: &mut impl Read) -> Result<u64, InterpreterError> {
        match unsafe { std::mem::transmute(port) } {
            Port::Text => Ok({
                let mut buf = [0];
                stdin.read_exact(&mut buf).unwrap();
                buf[0] as u64
            }),
            _ => Err(InterpreterError::UnsupportedPort(port))
        }
    }
}

#[derive(Debug)]
pub enum StepResult {
    Error(Error<InterpreterError>),
    Running,
    Halted,
}
