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

#[derive(Debug)]
#[repr(u64)]
pub enum Port {
    CpuBus, Text, Number, Supported = 5, Special, Profile,
    X, Y, Color, Buffer, GSpecial = 15,
    Ascii8, Char5, Char6, Ascii7, Utf8, TSpecial = 23,
    Int, UInt, Bin, Hex, Float, Fixed, NSpecial = 31,
    Addr, Bus, Page, SSpecial = 39,
    Rng, Note, Instr, NLeg, Wait, NAddr, Data, MSpecial,
}
