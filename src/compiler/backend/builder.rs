use super::ssa::*;
use logos::Span;

#[derive(Default)]
pub struct Builder {
    body: Body,

    block_id: BlockId,
    variable_id: VariableId,
    value_id: ValueId,
}

impl Builder {
    pub fn get_ssa(self) -> (Body, usize, usize) {
        (self.body, *self.value_id, *self.variable_id)
    }

    pub fn append_block<A: Into<String>>(&mut self, name: A, span: Option<Span>) -> BlockId {
        self.body.blocks.push(Block {
            name: name.into(),
            id: self.block_id,
            instructions: Vec::new(),
            terminator: Terminator::None,
            span
        });
        *self.block_id += 1;
        BlockId(*self.block_id - 1)
    }

    pub fn append_instruction(&mut self, block: BlockId, inst: Instruction) {
        self.body.blocks[*block].instructions.push(inst)
    }

    pub fn set_terminator(&mut self, block: BlockId, term: Terminator) {
        self.body.blocks[*block].terminator = term;
    }

    pub fn allocate_value(&mut self) -> ValueId {
        *self.value_id += 1;
        ValueId(*self.value_id - 1)
    }

    pub fn allocate_variable(&mut self) -> VariableId {
        *self.variable_id += 1;
        VariableId(*self.variable_id - 1)
    }
}
