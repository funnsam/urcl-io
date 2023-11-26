pub use logos::{Logos, Lexer};


#[derive(Clone, Debug, Logos)]
#[logos(skip r"//[^\n]+")]
#[logos(skip r"[\s^\n]")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    #[regex(r"(\+|\-)?(0[xX][A-Fa-f0-9]+|0[bB][0-1]+|0[oO][0-7]+|[0-9]+)", callback = |lex| parse_number(lex, 0).unwrap(), priority = 2)]
    Number(i64),

    #[regex(r"@[\S]+", callback = |lex| rm_prefix(lex, 1))]
    #[regex(r"bits"    , callback = |_| "bits".to_string(), ignore(case))]
    #[regex(r"minheap" , callback = |_| "minheap".to_string(), ignore(case))]
    #[regex(r"minstack", callback = |_| "minstack".to_string(), ignore(case))]
    #[regex(r"minreg"  , callback = |_| "minreg".to_string(), ignore(case))]
    Macro(String),

    #[regex(r"\.[\S]+", callback = |lex| rm_prefix(lex, 1))]
    Label(String),

    #[regex(r"%[\S]+", callback = |lex| rm_prefix(lex, 1))]
    Port(String),

    #[regex(r"[a-zA-Z_\u0100-\uFFFF][a-zA-Z_0-9\u0100-\uFFFF]*", callback = |lex| rm_prefix(lex, 0), priority = 0)]
    Name(String),

    #[regex(r"(R|r|\$)(\+|\-)?(0[xX][A-Fa-f0-9]+|0[bB][0-1]+|0[oO][0-7]+|[0-9]+)", callback = |lex| parse_number(lex, 1).unwrap())]
    Register(i64),

    #[regex(r"(M|m|\#)(\+|\-)?(0[xX][A-Fa-f0-9]+|0[bB][0-1]+|0[oO][0-7]+|[0-9]+)", callback = |lex| parse_number(lex, 1).unwrap())]
    Memory(i64),

    #[regex(r"'([^']|\\.|\\x[0-9a-fA-F]+|\\u[0-9a-fA-F]+)*'", callback = |lex| parse_char(lex).unwrap(), priority = 2)]
    Char(char),

    #[token("\n")]
    Newline,

    #[token("dw", ignore(case))]
    Dw,

    #[token("[")]
    ArrayStart,
    #[token("]")]
    ArrayEnd,
}

fn parse_char(lex: &Lexer<Token>) -> Option<char> {
    let c = lex.slice();
    match c.chars().nth(1) {
        Some('\\') => match c.chars().nth(2) {
            Some('a')  => Some('\x07'),
            Some('b')  => Some('\x08'),
            Some('e')  => Some('\x1b'),
            Some('f')  => Some('\x0c'),
            Some('n')  => Some('\x0a'),
            Some('r')  => Some('\x0d'),
            Some('t')  => Some('\x09'),
            Some('v')  => Some('\x0b'),
            Some('0')  => Some('\0'),
            Some('\\') => Some('\\'),
            Some('\'') => Some('\''),
            Some('\"') => Some('\"'),
            Some('x')  => unsafe { if c.len() != 6 { None } else { Some(char::from_u32_unchecked(u32::from_str_radix(&c[3..5], 16).unwrap())) }},
            Some('u')  => unsafe { if c.len() != 8 { None } else { Some(char::from_u32_unchecked(u32::from_str_radix(&c[3..7], 16).unwrap())) }},
            _ => {
                if c.len() == 4 {
                    c.chars().nth(2)
                } else {
                    None
                }
            }
        },
        _ => {
            if c.len() == 3 {
                c.chars().nth(1)
            } else {
                None
            }
        }
    }
}

fn parse_number(lex: &Lexer<Token>, skip: usize) -> Option<i64> {
    let number = lex.slice();
    let mut i  = skip;
    let mut neg = false;
    match number.chars().nth(i).unwrap() {
        '-' => { i += 1; neg = true },
        '+' => { i += 1; },
        _   => {},
    }
    let (radix, skip) = match number.chars().nth(i+1) {
        Some('x' | 'X') => (16, 2),
        Some('b' | 'B') => (2 , 2),
        Some('o' | 'O') => (8 , 2),
        _ => (10, 0)
    };
    i += skip;
    let number = i64::from_str_radix(&number[i..], radix).ok();
    match (number, neg) {
        (Some(v), true) => Some(-v),
        _ => number
    }
}

fn rm_prefix(lex: &Lexer<Token>, skip: usize) -> String {
    lex.slice()[skip..].to_string()
}
