use std::fmt::{self, Display, Formatter};

macro_rules! type_wrapper {
    ($tv: vis $type: ident = $iv: vis $inner: ty : $fmt: tt) => {
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $tv struct $type($iv $inner);

        impl ::std::ops::Deref for $type {
            type Target = $inner;

            fn deref(&self) -> &$inner {
                &self.0
            }
        }

        impl ::std::ops::DerefMut for $type {
            fn deref_mut(&mut self) -> &mut $inner {
                &mut self.0
            }
        }

        impl ::std::fmt::Display for $type {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::std::write!(fmt, $fmt, self.0)
            }
        }
    };
}

type_wrapper!(pub ValueId = pub(crate) usize    : "%{}");
type_wrapper!(pub VariableId = pub(crate) usize : "#{}");
type_wrapper!(pub BlockId = pub(crate) usize    : "${}");

#[derive(Default)]
pub struct Body {
    pub blocks: Vec<Block>,
}

pub struct Block {
    pub name: String,
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

pub struct Instruction {
    pub destination: Option<ValueId>,
    pub operation: Operation
}

pub enum Operation {
    Integer(u64),
    BinOp(BinOp, ValueId, ValueId),
    Call(Function, Vec<ValueId>),
    LoadVar(VariableId),
    StoreVar(VariableId, ValueId),
    Phi(Vec<(ValueId, BlockId)>),
}

#[derive(Debug)]
pub enum Function {
    LastOk,             // LastOk() -> int
    PortRead,           // PortRead(port: int) -> int
    PortWrite,          // PortWrite(port: int, data: int)
}

pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

pub enum Terminator {
    Return,
    Jump(BlockId),
    Branch(ValueId, BlockId, BlockId),
    None,
}

impl Display for Body {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        for blk in self.blocks.iter() {
            write!(fmt, "{blk}")?;
        }

        Ok(())
    }
}

impl Display for Block {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        writeln!(fmt, "\x1b[1;34mblk\x1b[0m {}: \x1b[90m// id: {}\x1b[0m", self.name, *self.id)?;

        for instr in &self.instructions {
            writeln!(fmt, "    {}", instr)?;
        }

        writeln!(fmt, "    \x1b[32m{}\x1b[0m", self.terminator)?;

        Ok(())
    }
}

impl Display for Terminator {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Self::Return => write!(fmt, "ret"),
            Self::Jump(block) => write!(fmt, "jmp {block}"),
            Self::Branch(cond, if_, else_) => write!(fmt, "br {cond} {if_} {else_}"),
            Self::None => Ok(()),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        if let Some(dst) = self.destination {
            write!(fmt, "\x1b[32m{dst}\x1b[0m = ")?;
        }

        write!(fmt, "\x1b[33m{}\x1b[0m", self.operation)
    }
}

impl Display for Operation {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Self::Integer(int)      => write!(fmt, "{int}"),
            Self::BinOp(op, l, r)   => write!(fmt, "{op} {l} {r}"),
            Self::Call(f, args)     => write!(fmt, "call {f:?}({})", args.iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(", ")
            ),
            Self::LoadVar(var)      => write!(fmt, "load {var}"),
            Self::StoreVar(v, val)  => write!(fmt, "store {v} {val}"),
            Self::Phi(branches)     => write!(fmt, "Φ {}", branches.iter()
                .map(|(val, block)| format!("{}: {}", block, val))
                .collect::<Vec<String>>()
                .join(", ")
            ),
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "add"),
            BinOp::Sub => write!(f, "sub"),
            BinOp::Mul => write!(f, "mul"),
            BinOp::Div => write!(f, "div"),
            BinOp::Mod => write!(f, "mod"),
            BinOp::And => write!(f, "and"),
            BinOp::Or  => write!(f, "or"),
            BinOp::Xor => write!(f, "xor"),
            BinOp::Shl => write!(f, "shl"),
            BinOp::Shr => write!(f, "shr"),
            BinOp::Eq  => write!(f, "eq"),
            BinOp::Ne  => write!(f, "ne"),
            BinOp::Lt  => write!(f, "lt"),
            BinOp::Le  => write!(f, "le"),
            BinOp::Gt  => write!(f, "gt"),
            BinOp::Ge  => write!(f, "ge"),
        }
    }
}

#[macro_export]
macro_rules! instruction {
    ($oper: expr => $dest: expr) => {{
        $crate::backend::ssa::Instruction { operation: $oper, destination: $dest }
    }};
}
