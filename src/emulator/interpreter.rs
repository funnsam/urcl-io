use super::{*, ast::*, error::*};
use logos::Span;
use std::io::Write;

#[derive(Debug)]
pub struct Interpreter {
    ast: Ast,

    pc: usize,
    sp: usize,

    ram: Vec<u64>,
    reg: Vec<u64>,

    pub debugging: bool
}

impl Interpreter {
    pub fn new(ast: Ast) -> Self {
        let ram_size = ast.dw.len() + ast.minheap + ast.minstack;
        let regs = ast.minreg;
        Self {
            ast,

            pc: 0,
            sp: 0,

            ram: vec![0; ram_size],
            reg: vec![0; regs],

            debugging: false
        }
    }

    pub fn step(&mut self, stdout: &mut impl Write) -> StepResult {
        macro_rules! step {
            ($op: expr => $sr: expr) => {{
                $op;
                $sr
            }};
        }

        macro_rules! trunc {
            ($v: expr) => {
                $v & ((1 << self.ast.bits as u64) - 1)
            };
        }

        macro_rules! some_or_error {
            ($opt: expr, $kind: expr, in $span: expr) => {
                if let Some(a) = $opt {
                    a
                } else {
                    return StepResult::Error(Error {
                        kind: $kind,
                        span: $span,
                    })
                }
            };
        }

        macro_rules! op {
            (set reg $reg: expr => $val: expr) => {{
                let reg = $reg;
                if reg != 0 {
                    self.reg[reg-1] = $val;
                }
            }};
            (get reg $reg: expr) => {{
                let reg = $reg;
                if reg != 0 {
                    self.reg[reg-1]
                } else {
                    0
                }
            }};
            (get any $val: expr) => {{
                match $val {
                    Any::Register(reg) => op!(get reg *reg),
                    Any::Immediate(imm) => trunc!(**imm),
                    _ => unreachable!(),
                }
            }};
            (get mem $adr: expr, in $span: expr) => {{
                let adr = $adr;
                *some_or_error!(self.ram.get(adr as usize), InterpreterError::MemoryAccessOob(adr), in $span)
            }};
            (set mem $adr: expr => $dt: expr, in $span: expr) => {{
                let adr = $adr;
                *some_or_error!(self.ram.get_mut(adr as usize), InterpreterError::MemoryAccessOob(adr), in $span) = $dt;
            }};
        }

        macro_rules! _if {
            ($cond: expr => $main: expr $(; $else: expr)?) => {
                if $cond {
                    $main
                } $( else {
                    $else
                })?
            };
        }

        macro_rules! branch {
            ($pc: expr) => {
                self.pc = $pc
            }
        }

        macro_rules! match_opcode {
            (|$span:ident| $($opc: ident $($iv: ident)* = $action: expr),* $(,)?) => {
                if let Some((inst, span)) = self.fetch().cloned() {
                    match &inst {
                        $(
                            Instruction::$opc($($iv),*) => {
                                #[allow(unused_variables)]
                                let $span = span;
                                $action
                            },
                        )*
                    }
                } else {
                    StepResult::Halted
                }
            };
        }

        match_opcode!(|s|
            ADD d a b = step!(op!(set reg *d => trunc!(op!(get any a) + op!(get any b))) => StepResult::Running),
            NOR d a b = step!(op!(set reg *d => !(op!(get any a) | op!(get any b))) => StepResult::Running),
            BGE d a b = step!(_if!((op!(get any a) >= op!(get any b)) => branch!(op!(get any d) as usize)) => StepResult::Running),
            STR d v = step!(op!(set mem op!(get any d) => op!(get any v), in s) => StepResult::Running),
            LOD d v = step!(op!(set reg *d => op!(get mem op!(get any v), in s)) => StepResult::Running),
            RSH d v = step!(op!(set reg *d => op!(get any v) >> 1) => StepResult::Running),
            IMM d v = step!(op!(set reg *d => op!(get any v)) => StepResult::Running),
            MOV d v = step!(op!(set reg *d => op!(get any v)) => StepResult::Running),
            OUT p v = self.port_out(op!(get any p), op!(get any v), s, stdout),
            IN d p = todo!("IN r{d} {p:?}"),
        )
    }

    fn fetch(&mut self) -> Option<&(Instruction, Span)> {
        self.pc += 1;
        self.ast.instructions.get(self.pc-1)
    }

    fn port_out(&mut self, port: u64, data: u64, span: Span, stdout: &mut impl Write) -> StepResult {
        match unsafe { std::mem::transmute(port) } {
            Port::Text => write!(stdout, "{}", unsafe { char::from_u32_unchecked(data as u32) }).unwrap(),
            Port::Number => write!(stdout, "{data}").unwrap(),

            Port::Profile => {
                self.debugging = data & 1 != 0;
                println!("\x1b[1;32mInterpreter:\x1b[0m debugging is now {}", if self.debugging { "enabled" } else { "disabled" });
            },
            _ => return StepResult::Error(Error { kind: InterpreterError::UnsupportedPort(port), span }),
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
