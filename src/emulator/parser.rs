use super::{ast::*, lexer::*, common::*};
use logos::Span;
use std::collections::HashMap;

#[allow(clippy::cognitive_complexity)]
pub fn parse(parser: &mut Parser) -> Result<(), Vec<(ParserError, Span)>> {
    let mut args = Vec::new();
    let mut opcd = String::new();
    let mut start_span = 0;

    let mut name  = HashMap::<String, Any>::new();
    let mut label = (HashMap::<String, usize>::new(), 0, HashMap::<usize, u64>::new());
    let mut replace_labels = Vec::<(usize, Immediate)>::new();

    macro_rules! give_id {
        ($name: expr) => {{
            if let Some(v) = label.0.get(&$name) {
                *v
            } else {
                label.0.insert($name, label.1);
                label.1 += 1;
                label.1 - 1
            }
        }};
    }

    while let Some((tok, span)) = parser.next().cloned() {
        match tok {
            Token::Name(n) => if opcd.is_empty() {
                opcd = n.clone();
                start_span = span.start;
            } else {
                args.push(Any::Name(n.clone()));
            },
            Token::Label(lb) => args.push(Any::UnresolvedLabel(give_id!(lb.clone()))),
            Token::Register(rth) => args.push(Any::Register(rth as usize)),
            Token::Number(num) => args.push(Any::Immediate(Box::new(num as u64))),
            Token::Macro(m) => match m.to_lowercase().as_str() {
                "define" => {
                    let f = parser.next().cloned().unwrap(); // TODO: eof or nl handle
                    let t = parser.next().cloned().unwrap();

                    let f = match f.0 {
                        Token::Name(n) => n,
                        _ => todo!("unexpected token type")
                    };

                    let t = match t.0 {
                        Token::Name(n) => name.get(&n).cloned().unwrap(),
                        Token::Label(lb) => Any::UnresolvedLabel(give_id!(lb.clone())),
                        Token::Register(rth) => Any::Register(rth as usize),
                        Token::Number(num) => Any::Immediate(Box::new(num as u64)),
                        _ => todo!("unexpected token type"),
                    };

                    name.insert(f, t);
                },
                _ => todo!("unknown macro"),
            },
            Token::Newline => {
                macro_rules! count {
                    () => (0_usize);
                    ( $x:tt $($xs:tt)* ) => (1_usize + count!($($xs)*));
                }

                macro_rules! any_or {
                    (Any) => {{
                        let a = args.remove(0);
                        if let Any::UnresolvedLabel(id) = a {
                            let imm = Box::new(69);
                            let imm_clone = unsafe { std::ptr::read::<Box<u64>>(&imm as *const Immediate) };
                            replace_labels.push((id, imm_clone));
                            Any::Immediate(imm)
                        } else {
                            a
                        }
                    }};
                    ($variant: ident) => {{
                        let a = args.remove(0);
                        if let Any::$variant(ok) = a {
                            ok
                        } else if let Any::Name(id) = a {
                            let r = name.get(&id).clone();
                            if let Some(Any::$variant(ok)) = r {
                                *ok
                            } else {
                                todo!("name type not match");
                            }
                        } else {
                            todo!("arg not match");
                        }
                    }};
                }

                macro_rules! match_opcode {
                    ($($name: ident $($variant: ident)*),* $(,)?) => {
                        match opcd.to_uppercase().as_str() {
                            $(
                                stringify!($name) => {
                                    if count!($($variant)*) != args.len() {
                                        todo!("ne args {args:?}");
                                    }

                                    parser.ast.instructions.push((Instruction::$name($(
                                        any_or!($variant)
                                    ),*), Span { start: start_span, end: span.end }));

                                    opcd.clear();
                                    args.clear();
                                },
                            )*
                            "" => {
                                match args.get(0).unwrap() {
                                    Any::UnresolvedLabel(l) => {
                                        label.2.insert(*l, parser.ast.instructions.len() as u64);
                                    },
                                    _ => todo!("unexpected"),
                                }

                                args.clear();
                            },
                            _ => todo!("{}", opcd),
                        }
                    };
                }

                match_opcode!(
                    ADD Register Any Any,
                    RSH Register Any,
                    LOD Register Any,
                    STR Any Any,
                    BGE Any Any Any,
                    NOR Register Any Any,
                    IMM Register Any,
                );
            },
            _ => todo!("{:?}", tok)
        }
    }

    for (id, mut ptr) in replace_labels.into_iter() {
        *ptr = *label.2.get(&id).unwrap();
        std::mem::forget(ptr);
    }

    Ok(())
}
