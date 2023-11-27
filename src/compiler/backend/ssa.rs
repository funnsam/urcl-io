use std::fmt::{self, Display, Formatter};
use logos::Span;

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

macro_rules! binop {
    ($($name: ident = $op: tt),* $(,)?) => {
        pub enum BinOp {
            $($name),*
        }

        impl ::std::fmt::Display for BinOp {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    $(BinOp::$name => write!(f, "{}", stringify!($name).to_lowercase())),*
                }
            }
        }

        impl BinOp {
            pub fn operate(&self, lhs: u64, rhs: u64) -> u64 {
                match self {
                    $(BinOp::$name => (lhs $op rhs) as u64),*
                }
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
    pub span: Option<Span>,
}

pub struct Instruction {
    pub destination: Option<ValueId>,
    pub operation: Operation,
}

pub enum Operation {
    Integer(u64),
    BinOp(BinOp, ValueId, ValueId),
    Call(Function, Vec<ValueId>),
    Allocate(VariableId, ValueId),
    LoadIndex(VariableId, ValueId),
    StoreIndex(VariableId, ValueId, ValueId),
    Phi(Vec<(ValueId, BlockId)>),
}

#[derive(Debug)]
pub enum Function {
    LastOk,             // LastOk() -> bool
    PortRead,           // PortRead(port: int) -> int
    PortWrite,          // PortWrite(port: int, data: int)
    ReportError,        // ReportError(kind: int, program_counter: int) -> !
}

binop!(
    Add = +,
    Sub = -,
    Mul = *,
    Div = /,
    Mod = %,
    And = &,
    Or  = |,
    Xor = ^,
    Shl = <<,
    Shr = >>,
    Eq  = ==,
    Ne  = !=,
    Lt  = <,
    Le  = <=,
    Gt  = >,
    Ge  = >=,
);

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
        writeln!(fmt, "\x1b[35m${}\x1b[0m: \x1b[90m// {} {:?}\x1b[0m", *self.id, self.name, self.span)?;

        for instr in &self.instructions {
            writeln!(fmt, "    {}", instr)?;
        }

        writeln!(fmt, "    {}\x1b[0m", self.terminator)?;

        Ok(())
    }
}

impl Display for Terminator {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Self::Return                    => write!(fmt, "\x1b[32mret"),
            Self::Jump(block)               => write!(fmt, "\x1b[32mjmp \x1b[35m{block}"),
            Self::Branch(cond, if_, else_)  => write!(fmt, "\x1b[32mbr \x1b[36m{cond} \x1b[35m{if_} {else_}"),
            Self::None                      => write!(fmt, "\x1b[1;31mno terminator!"),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        if let Some(dst) = self.destination {
            write!(fmt, "\x1b[36m{dst}\x1b[0m = ")?;
        }

        write!(fmt, "{}\x1b[0m", self.operation)
    }
}

impl Display for Operation {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Self::Integer(int)      => write!(fmt, "{int}"),
            Self::BinOp(op, l, r)   => write!(fmt, "\x1b[33m{op} \x1b[36m{l} {r}"),
            Self::Call(f, args)     => write!(fmt, "\x1b[33mcall \x1b[34m{f:?}\x1b[0m({})", args.iter()
                .map(|e| format!("\x1b[36m{}\x1b[0m", e))
                .collect::<Vec<String>>()
                .join(", ")
            ),
            Self::Allocate(v, size) => write!(fmt, "\x1b[33malloc \x1b[34m{v} \x1b[36m{size}"),
            Self::LoadIndex(var, off)       => write!(fmt, "\x1b[33mload \x1b[34m{var}\x1b[0m[\x1b[36m{off}\x1b[0m]"),
            Self::StoreIndex(v, off, val)   => write!(fmt, "\x1b[33mstore \x1b[34m{v}\x1b[0m[\x1b[36m{off}\x1b[0m] \x1b[36m{val}"),
            Self::Phi(branches)     => write!(fmt, "\x1b[33mphi \x1b[34m{}", branches.iter()
                .map(|(val, block)| format!("{}: {}", block, val))
                .collect::<Vec<String>>()
                .join(", ")
            ),
        }
    }
}

#[macro_export]
macro_rules! instruction {
    ($oper: expr => $dest: expr) => {{
        $crate::compiler::backend::ssa::Instruction { operation: $oper, destination: Some($dest) }
    }};
    ($oper: expr) => {{
        $crate::compiler::backend::ssa::Instruction { operation: $oper, destination: None }
    }};
}
