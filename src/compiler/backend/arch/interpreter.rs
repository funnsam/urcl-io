use crate::compiler::{error::*, backend::ssa::*};
use std::{collections::HashMap, io::{Read, Write}};
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Interpreter {
    #[derivative(Debug="ignore")]
    ssa: Body,

    block_id: BlockId,
    instr_id: usize,
    block_id_old: BlockId,

    values: HashMap<ValueId, u64>,
    variables: HashMap<VariableId, Vec<u64>>,
}

impl Interpreter {
    pub fn new(ssa: (Body, usize, usize)) -> Self {
        Self {
            ssa: ssa.0,

            block_id: BlockId(0),
            instr_id: 0,
            block_id_old: BlockId(0),

            values: HashMap::with_capacity(ssa.1),
            variables: HashMap::with_capacity(ssa.2),
        }
    }

    pub fn step(&mut self, stdout: &mut impl Write, stdin: &mut impl Read) -> StepResult {
        let block = self.ssa.blocks.get(*self.block_id).unwrap();
        let instr = block.instructions.get(self.instr_id).unwrap();

        macro_rules! get {
            (val $a: expr) => {
                *self.values.get($a).unwrap()
            };
        }

        let val = match &instr.operation {
            Operation::Integer(imm) => Some(*imm),
            Operation::LoadIndex(var, off) => Some(*self.variables.get(var).unwrap().get(get!(val off) as usize).unwrap()),
            Operation::StoreIndex(var, off, dat) => {
                *self.variables.get_mut(var).unwrap().get_mut(get!(val off) as usize).unwrap() = get!(val dat);
                None
            },
            Operation::Allocate(var, siz) => {
                self.variables.insert(*var, vec![0; get!(val siz) as usize]);
                None
            },
            Operation::BinOp(op, l, r) => {
                Some(op.operate(get!(val l), get!(val r)))
            },
            _ => todo!(),
        };

        if let Some(dest) = instr.destination {
            self.values.insert(dest, val.unwrap());
        }

        self.instr_id += 1;
        if self.instr_id == block.instructions.len() {
            match block.terminator {
                Terminator::Jump(blk) => {
                    self.block_id_old = self.block_id;
                    self.block_id = blk;
                },
                Terminator::Return => return StepResult::Halted,
                _ => todo!(),
            }

            self.instr_id = 0;
        }

        StepResult::Running
    }
}

#[derive(Debug)]
pub enum StepResult {
    Error(Error<InterpreterError>),
    Running,
    Halted,
}
