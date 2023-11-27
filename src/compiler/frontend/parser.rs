use crate::compiler::{error::*, common::*, frontend::{ast::*, lexer::*}};
use logos::Span;
use std::collections::HashMap;

#[allow(clippy::cognitive_complexity)]
pub fn parse(parser: &mut Parser) -> Result<(), Vec<Error<ParserError>>> {
    let mut args = Vec::new();
    let mut opcd = (String::new(), Span::default());

    let mut errors = Vec::new();

    let mut name  = HashMap::<String, Any>::new();
    let mut label = (HashMap::<String, usize>::new(), 0, HashMap::<usize, u64>::new());
    let mut replace_labels = Vec::<(usize, Immediate, Span)>::new();

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

    macro_rules! options_get_value {
        ($tok: expr) => {
            match $tok {
                Token::Name(name)       => Some(Any::Name(name.clone())),
                Token::Label(lb)        => Some(Any::UnresolvedLabel(give_id!(lb.clone()))),
                Token::Register(rth)    => Some(Any::Register(rth as usize)),
                Token::Number(num)      => Some(Any::Immediate(Box::new(num as u64))),
                Token::Char(chr)        => Some(Any::Immediate(Box::new(chr as u64))),
                _ => None,
            }
        };
    }

    macro_rules! get_value {
        ($main_loop: tt: $tok_span_tuple: expr) => {
            some_or_error!($main_loop: options_get_value!($tok_span_tuple.0), ExpectingValue in $tok_span_tuple.1)
        };
        ($main_loop: tt: $tok: expr, $span: expr) => { get_value!($main_loop: ($tok, $span)) };
    }

    macro_rules! options_get_value_fold {
        ($main_loop: tt: $tok: expr, $span: expr) => {
            match $tok {
                Token::Name(n) => Some(some_or_error!($main_loop: name.get(&n), NameNotDefined in $span).clone()),
                other          => options_get_value!(other),
            }
        };
    }

    macro_rules! get_value_fold {
        ($main_loop: tt: $tok_span_tuple: expr) => {
            some_or_error!($main_loop: options_get_value_fold!($main_loop: $tok_span_tuple.0, $tok_span_tuple.1), ExpectingValue in $tok_span_tuple.1)
        };
        ($main_loop: tt: $tok: expr, $span: expr) => { get_value_fold!($main_loop, ($tok, $span)) };
    }

    macro_rules! options_get_name {
        ($tok: expr) => {
            match $tok {
                Token::Name(name) => Some(name.clone()),
                _ => None,
            }
        }
    }

    macro_rules! get_name {
        ($main_loop: tt: $tok_span_tuple: expr) => {
            some_or_error!($main_loop: options_get_name!($tok_span_tuple.0), ExpectingValue in $tok_span_tuple.1)
        };
        ($main_loop: tt: $tok: expr, $span: expr) => { get_name!($main_loop: ($tok, $span)) };
    }

    macro_rules! options_get_imm {
        ($tok: expr) => {
            match $tok {
                Token::Number(imm) => Some(imm),
                _ => None,
            }
        }
    }

    macro_rules! get_imm {
        ($main_loop: tt: $tok_span_tuple: expr) => {
            some_or_error!($main_loop: options_get_imm!($tok_span_tuple.0), ExpectingImmediate in $tok_span_tuple.1)
        };
        ($main_loop: tt: $tok: expr, $span: expr) => { get_imm!($main_loop: ($tok, $span)) };
    }

    macro_rules! some_or_error {
        ($main_loop: tt: $opt: expr, $kind: ident in $span: expr) => {
            if let Some(ok) = $opt {
                ok
            } else {
                error!($main_loop: $kind in $span);
            }
        }
    }

    macro_rules! error {
        ($main_loop: tt: $kind: ident in $span: expr) => {{
            errors.push(Error { kind: ParserError::$kind, span: $span });
            opcd.0.clear();
            args.clear();

            if !matches!(parser.current(), Some((Token::Newline, _))) {
                while let Some((tok, _)) = parser.next() {
                    match tok {
                        Token::Newline => break,
                        _ => {},
                    }
                }
            }

            continue $main_loop;
        }};
        (not_in_loop: $kind: ident in $span: expr) => {{
            errors.push(Error { kind: $kind, span: $span });
        }};
    }

    'main_loop: while let Some((tok, span)) = parser.next().cloned() {
        match tok {
            Token::Name(n) => if opcd.0.is_empty() && args.is_empty() {
                opcd = (n.clone(), span);
            } else {
                args.push((Any::Name(n.clone()), span));
            },
            Token::Macro(m) => match m.to_lowercase().as_str() {
                "define" => {
                    let k = get_name!('main_loop: some_or_error!('main_loop: parser.next().cloned(), UnexpectedEof in span));
                    let v = get_value_fold!('main_loop: some_or_error!('main_loop: parser.next().cloned(), UnexpectedEof in span));

                    name.insert(k, v);
                },
                "bits"      => parser.ast.bits      = get_imm!('main_loop: some_or_error!('main_loop: parser.next().cloned(), UnexpectedEof in span)) as usize,
                "minheap"   => parser.ast.minheap   = get_imm!('main_loop: some_or_error!('main_loop: parser.next().cloned(), UnexpectedEof in span)) as usize,
                "minstack"  => parser.ast.minstack  = get_imm!('main_loop: some_or_error!('main_loop: parser.next().cloned(), UnexpectedEof in span)) as usize,
                "minreg"    => parser.ast.minreg    = get_imm!('main_loop: some_or_error!('main_loop: parser.next().cloned(), UnexpectedEof in span)) as usize,
                _ => error!('main_loop: UnknownMacro in span),
            },
            Token::Dw => {
                let word = get_imm!('main_loop: some_or_error!('main_loop: parser.next().cloned(), UnexpectedEof in span)) as u64;
                parser.ast.dw.push(word);
            },
            Token::Newline => {
                macro_rules! count {
                    () => (0_usize);
                    ( $x:tt $($xs:tt)* ) => (1_usize + count!($($xs)*));
                }

                macro_rules! any_or {
                    (Any) => {{
                        let a = args.remove(0);
                        if let Any::UnresolvedLabel(id) = a.0 {
                            let imm = Box::new(69);
                            let imm_clone = unsafe { std::ptr::read::<Box<u64>>(&imm as *const Immediate) };
                            replace_labels.push((id, imm_clone, span.clone()));
                            Any::Immediate(imm)
                        } else if let Any::Name(id) = a.0 {
                            some_or_error!('main_loop: name.get(&id).clone(), NameNotDefined in a.1).clone()
                        } else {
                            a.0
                        }
                    }};
                    ($variant: ident) => {{
                        let a = args.remove(0);
                        if let Any::$variant(ok) = a.0 {
                            ok
                        } else if let Any::Name(id) = a.0 {
                            let r = some_or_error!('main_loop: name.get(&id).clone(), NameNotDefined in a.1);
                            if let Any::$variant(ok) = r {
                                ok.clone()
                            } else {
                                error!('main_loop: OperandWrongType in a.1);
                            }
                        } else {
                            error!('main_loop: OperandWrongType in a.1);
                        }
                    }};
                }

                macro_rules! match_opcode {
                    ($($name: ident $($variant: ident)*),* $(,)?) => {
                        match opcd.0.to_uppercase().as_str() {
                            $(
                                stringify!($name) => {
                                    if count!($($variant)*) != args.len() {
                                        error!('main_loop: OperandCountNotMatch in Span { start: opcd.1.start, end: span.end });
                                    }

                                    parser.ast.instructions.push((Instruction::$name($(
                                        any_or!($variant)
                                    ),*), Span { start: opcd.1.start, end: span.end }));

                                    opcd.0.clear();
                                    args.clear();
                                },
                            )*
                            "" => {
                                match args.get(0) {
                                    Some((Any::UnresolvedLabel(l), _)) => {
                                        label.2.insert(*l, parser.ast.instructions.len() as u64);
                                    },
                                    None => {},
                                    Some((_, s)) => error!('main_loop: SyntaxError in s.clone()),
                                }

                                args.clear();
                            },
                            _ => error!('main_loop: UnknownOpcode in opcd.1.clone()),
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

                    MOV Register Any,

                    IN Register Any,
                    OUT Any Any,
                );
            },
            _ => args.push((get_value!('main_loop: tok.clone(), span.clone()), span)),
        }
    }

    'main_loop: for (id, mut ptr, span) in replace_labels.into_iter() {
        *ptr = *some_or_error!('main_loop: label.2.get(&id), LabelNotDefined in span);
        std::mem::forget(ptr);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
