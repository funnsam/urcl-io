#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum Instruction {
    ADD(Register, Any, Any),
    RSH(Register, Any),
    LOD(Register, Any),
    STR(Any, Any),
    BGE(Any, Any, Any),
    NOR(Register, Any, Any),
    IMM(Register, Any),

    MOV(Register, Any),

    IN(Register, Any),
    OUT(Any, Any),
}

#[derive(Debug, Clone)]
pub enum Any {
    Register(Register),
    Immediate(Immediate),
    UnresolvedLabel(usize),
    Name(String),
}

pub type Register = usize;
pub type Immediate = Box<u64>;
