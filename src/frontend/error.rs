use logos::Span;
use crate::*;

pub trait ErrorKind {
    fn message(&self) -> String;
}

#[derive(Debug)]
pub struct Error<Kind> {
    pub kind: Kind,
    pub span: Span,
}

macro_rules! error_kind {
    ($error_name: ident = $($internal: ident $message: tt $(+ $param: ty)?),* $(,)?) => {
        #[derive(Debug)]
        pub enum $error_name {
            $($internal$(($param))?),*
        }

        impl ErrorKind for $error_name {
            fn message(&self) -> String {
                match self {
                    $($crate::frontend::error::$error_name::$internal$((error_kind!(ident a $param)))? => format!($message $(, error_kind!(ident a $param))?)),*
                }
            }
        }
    };
    (ident $id: ident $_: ty) => {
        $id
    };
}

error_kind!(InterpreterError =
    StackOverflow       "stack overflowed",
    StackUnderflow      "stack underflowed",
    UnsupportedPort     "unsupported port {}" + u64,
    MemoryAccessOob     "accessed out-of-bound memory location {}" + u64
);

error_kind!(ParserError =
    SyntaxError             "syntax error",
    LabelNotDefined         "label is not defined anywhere",
    ExpectingValue          "expecting value",
    ExpectingName           "expecting name",
    ExpectingImmediate      "expecting immediate",
    UnknownMacro            "unknown macro",
    OperandWrongType        "the operand have a incompatable type with the instruction",
    UnknownOpcode           "unknown opcode is used",
    OperandCountNotMatch    "opcode doesn't support the amound of operand currently specified",
    NameNotDefined          "name is not defined previously",
    UnexpectedEof           "unexpected end of file",
);

pub struct LexerError;
impl ErrorKind for LexerError {
    fn message(&self) -> String {
        "lexer error".to_string()
    }
}

#[derive(Debug)]
pub struct ErrorContext<'a> {
    pub source: &'a str,
    pub cat: Vec<usize>
}

impl<Kind: ErrorKind> Error<Kind> {
    pub fn to_formats(&self, ctx: &ErrorContext<'_>) -> Vec<FormatSegment> {
        let mut i = ctx.source[..self.span.start].matches('\n').count()+1;
        let end = ctx.source[..self.span.end].matches('\n').count()+1;
        let chw = format!("{end}").len();

        let mut segments = vec![
            segment!("Error:".to_string(), BrightRed None b),
            segment!(format!(" {}{}\n", self.kind.message(), " ".repeat(chw)), None None),
            segment!(format!("{} \u{2502}\n", " ".repeat(chw)), BrightBlue None),
        ];

        for el in ctx.source.lines().skip(i-1) {
            let spaces = self.span.start.saturating_sub(ctx.cat[i-1]);

            segments.extend([
                segment!(
                    format!("{i}{} \u{2502} ", " ".repeat(chw - format!("{i}").len())),
                    BrightBlue None
                ),
                segment!(el.trim_end().replace('\t', "    "), None None),
                segment!(
                    format!(
                        "\n {}\u{2502}",
                        " ".repeat(chw),
                    ),
                    BrightBlue None
                ),
                segment!(
                    format!(
                        " {}{}\n",
                        " ".repeat(spaces + el.matches('\t').count() * 3),
                        "^".repeat(el.len()-spaces-ctx.cat.get(i).unwrap_or(&0).checked_sub(self.span.end).unwrap_or(0)+1),
                    ),
                    BrightYellow None
                ),
            ]);

            if i >= end {
                break;
            }
            i += 1;
        }

        segments.extend([
            segment!(format!("{} \u{2502}\n", " ".repeat(chw)), BrightBlue None),
            segment!("\n".to_string(), None None)
        ]);

        segments
    }
}

pub fn errors_to_formats<Kind: ErrorKind>(errors: Vec<Error<Kind>>, src: &str) -> Vec<FormatSegment> {
    let mut cat = Vec::with_capacity(src.split('\n').count());
    let mut j = 0;
    for el in src.split('\n') {
        cat.push(j);
        j += el.len()+1;
    }

    let ctx = ErrorContext { cat, source: src };

    let mut out = Vec::new();
    for err in errors {
        out.append(&mut err.to_formats(&ctx));
    }

    out
}
