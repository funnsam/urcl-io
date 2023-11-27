use crate::{
    instruction,
    compiler::{
        common::{Any, Instruction as Inst},
        frontend::ast::*,
        backend::{ssa::*, builder::*}
    },
};

pub fn generate_ssa(ast: Ast) -> (Body, usize, usize) {
    let mut builder = Builder::default();

    let alloc = builder.append_block("alloc", None);

    let ram = builder.allocate_variable();
    let ram_size = builder.allocate_value();
    builder.append_instruction(alloc, instruction!(Operation::Integer((ast.minheap + ast.minstack + ast.dw.len()) as u64) => ram_size));
    builder.append_instruction(alloc, instruction!(Operation::Allocate(ram, ram_size)));

    let reg = builder.allocate_variable();
    let reg_size = builder.allocate_value();
    builder.append_instruction(alloc, instruction!(Operation::Integer(ast.minreg as u64) => reg_size));
    builder.append_instruction(alloc, instruction!(Operation::Allocate(reg, reg_size)));

    let init = builder.append_block("init", None);
    builder.set_terminator(alloc, Terminator::Jump(init));

    let bit_mask = builder.allocate_value();
    builder.append_instruction(init, instruction!(Operation::Integer((1_u64 << ast.bits) - 1) => bit_mask));
    let zero = builder.allocate_value();
    builder.append_instruction(init, instruction!(Operation::Integer(0) => zero));

    for (i, w) in ast.dw.iter().enumerate() {
        let idx = builder.allocate_value();
        let wrd = builder.allocate_value();
        builder.append_instruction(init, instruction!(Operation::Integer(i as u64) => idx));
        builder.append_instruction(init, instruction!(Operation::Integer(*w) => wrd));
        builder.append_instruction(init, instruction!(Operation::StoreIndex(ram, idx, wrd)));
    }

    let mut blocks = Vec::with_capacity(ast.instructions.len()+1);
    for (i, (_, span)) in ast.instructions.iter().enumerate() {
        blocks.push(builder.append_block(format!("inst_{i}"), Some(span.clone())));
    }

    let end = builder.append_block("end", None);
    blocks.push(end);

    builder.set_terminator(init, Terminator::Jump(blocks.first().unwrap().clone()));

    for (i, (inst, _)) in ast.instructions.iter().enumerate() {
        let block = blocks[i].clone();

        macro_rules! get_value {
            (any $a: expr) => {{
                match $a {
                    Any::Register(ref a)  => get_value!(reg *a),
                    Any::Immediate(ref a) => get_value!(imm **a),
                    _ => unreachable!()
                }
            }};
            (reg $a: expr) => {{
                let a = $a;
                if a != 0 {
                    let a_value = builder.allocate_value();
                    let res_a = builder.allocate_value();
                    builder.append_instruction(block, instruction!(Operation::Integer(a as u64 - 1) => a_value));
                    builder.append_instruction(block, instruction!(Operation::LoadIndex(reg, a_value) => res_a));
                    res_a
                } else {
                    zero
                }
            }};
            (imm $a: expr) => {{
                let a_value = builder.allocate_value();
                builder.append_instruction(block, instruction!(Operation::Integer($a) => a_value));
                a_value
            }};
        }

        macro_rules! set {
            (reg $d: expr => $v: expr) => {{
                let d = $d;
                if d != 0 {
                    let d_nth = builder.allocate_value();
                    builder.append_instruction(block, instruction!(Operation::Integer(d as u64 - 1) => d_nth));
                    builder.append_instruction(block, instruction!(Operation::StoreIndex(reg, d_nth, $v)));
                } else {}
            }};
        }

        match inst {
            Inst::ADD(d, a, b) => {
                let a = get_value!(any a);
                let b = get_value!(any b);
                let d_tmp = builder.allocate_value();
                builder.append_instruction(block, instruction!(Operation::BinOp(BinOp::Add, a, b) => d_tmp));
                set!(reg *d => d_tmp);
                builder.set_terminator(block, Terminator::Jump(blocks[i+1]));
            },
            Inst::NOR(d, a, b) => {
                let a = get_value!(any a);
                let b = get_value!(any b);
                let d_1 = builder.allocate_value();
                let d_2 = builder.allocate_value();
                builder.append_instruction(block, instruction!(Operation::BinOp(BinOp::Or, a, b) => d_1));
                builder.append_instruction(block, instruction!(Operation::BinOp(BinOp::Xor, d_1, bit_mask) => d_2));
                set!(reg *d => d_2);
                builder.set_terminator(block, Terminator::Jump(blocks[i+1]));
            },
            Inst::BGE(addr, a, b) => {
                let a = get_value!(any a);
                let b = get_value!(any b);
                let cond = builder.allocate_value();
                builder.append_instruction(block, instruction!(Operation::BinOp(BinOp::Ge, a, b) => cond));

                match addr {
                    Any::Immediate(imm) => {
                        builder.set_terminator(block, Terminator::Branch(cond, blocks.get(**imm as usize).unwrap_or_else(|| blocks.last().unwrap()).clone(), blocks[i+1]));
                    },
                    Any::Register(_) => todo!(),
                    _ => unreachable!(),
                }
            },
            Inst::MOV(d, a) | Inst::IMM(d, a) => {
                let a = get_value!(any a);
                set!(reg *d => a);
                builder.set_terminator(block, Terminator::Jump(blocks[i+1]));
            },
            Inst::IN(d, p) => {
                let p = get_value!(any p);
                let d_tmp = builder.allocate_value();
                builder.append_instruction(block, instruction!(Operation::Call(Function::PortRead, vec![p]) => d_tmp));
                set!(reg *d => d_tmp);
                builder.set_terminator(block, Terminator::Jump(blocks[i+1]));
            },
            Inst::OUT(p, d) => {
                let p = get_value!(any p);
                let d = get_value!(any d);
                builder.append_instruction(block, instruction!(Operation::Call(Function::PortWrite, vec![p, d])));
                builder.set_terminator(block, Terminator::Jump(blocks[i+1]));
            },
            _ => {
                todo!()
                // builder.set_terminator(block, Terminator::Jump(blocks[i+1]));
            },
        }
    }

    builder.set_terminator(end, Terminator::Return);

    builder.get_ssa()
}
