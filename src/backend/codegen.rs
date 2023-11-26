use crate::{
    instruction,
    frontend::{ast::*, *},
    backend::{ssa::*, builder::*},
};

pub fn generate_ssa(ast: Ast) -> Body {
    let mut builder = Builder::default();

    let alloc = builder.append_block("alloc");
    let null = builder.allocate_value();
    builder.append_instruction(alloc, instruction!(Operation::Integer(0) => Some(null)));

    builder.get_body()
}
