use logos::Span;

pub trait ErrorKind {
    fn message(&self) -> &'static str;
}

#[derive(Debug)]
pub struct Error<Kind> {
    pub kind: Kind,
    pub span: Span,
}

macro_rules! error_kind {
    ($error_name: ident = $($internal: ident $message: tt),* $(,)?) => {
        #[derive(Debug)]
        pub enum $error_name {
            $($internal),*
        }

        impl ErrorKind for $error_name {
            fn message(&self) -> &'static str {
                match self {
                    $($crate::emulator::error::$error_name::$internal => $message),*
                }
            }
        }
    };
}

error_kind!(InterpreterError =
    StackOverflow       "stack overflowed",
    StackUnderflow      "stack underflowed",
);

error_kind!(ParserError =
    SyntaxError             "syntax error",
    LabelNotDefined         "label is not defined anywhere",
    ExpectingValue          "expecting value",
    ExpectingName           "expecting name",
    UnknownMacro            "unknown macro",
    OperandWrongType        "the operand have a incompatable type with the instruction",
    UnknownOpcode           "unknown opcode is used",
    OperandCountNotMatch    "opcode doesn't support the amound of operand currently specified",
    NameNotDefined          "name is not defined previously",
    UnexpectedEof           "unexpected end of file",
);

macro_rules! segment {
    ($text: expr, $fg: ident $bg: ident) => { FormatSegment { text: $text, fg: Color::$fg, bg: Color::$bg } };
}

#[derive(Debug)]
pub struct ErrorContext<'a> {
    pub source: &'a str,
    pub cat: Vec<usize>
}

impl<Kind: ErrorKind> Error<Kind> {
    pub fn to_formats<'a>(&self, ctx: &ErrorContext<'a>) -> Vec<FormatSegment> {
        let mut i = ctx.source[..self.span.start].matches('\n').count()+1;
        let end = ctx.source[..self.span.end].matches('\n').count()+1;
        let chw = format!("{end}").len();

        let mut segments = vec![
            segment!("Error".to_string(), BrightRed None),
            segment!(format!(": {}{}\n", self.kind.message(), " ".repeat(chw)), None None)
        ];

        for el in ctx.source.lines().skip(i-1) {
            let spaces = self.span.start.checked_sub(ctx.cat[i-1]).unwrap_or(0);

            segments.push(segment!(
                format!("{i}{} \u{2502} ", " ".repeat(chw - format!("{i}").len())),
                BrightBlue None
            ));

            segments.push(segment!(el.trim_end().replace("\t", "    "), None None));

            segments.push(segment!(
                format!(
                    "\n {}\u{2502}",
                    " ".repeat(chw),
                ),
                BrightBlue None
            ));

            segments.push(segment!(
                format!(
                    " {}{}\n",
                    " ".repeat(spaces + el.matches('\t').count() * 3),
                    "^".repeat(el.len()-spaces-ctx.cat.get(i).unwrap_or(&0).checked_sub(self.span.end).unwrap_or(0)+1),
                ),
                BrightYellow None
            ));

            if i >= end {
                break;
            }
            i += 1;
        }

        segments.push(segment!("\n".to_string(), None None));

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

pub struct FormatSegment {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
}

pub enum Color {
    Black, Red, Green, Yellow, Blue, Magenta, Cyan, White,
    BrightBlack, BrightRed, BrightGreen, BrightYellow, BrightBlue, BrightMagenta, BrightCyan, BrightWhite,
    None
}

impl Color {
    pub fn ansi_fg(&self) -> &'static str {
        use Color::*;
        match self {
            Black           => "30",
            Red             => "31",
            Green           => "32",
            Yellow          => "33",
            Blue            => "34",
            Magenta         => "35",
            Cyan            => "36",
            White           => "37",
            BrightBlack     => "90",
            BrightRed       => "91",
            BrightGreen     => "92",
            BrightYellow    => "93",
            BrightBlue      => "94",
            BrightMagenta   => "95",
            BrightCyan      => "96",
            BrightWhite     => "97",
            None            => "",
        }
    }
    pub fn ansi_bg(&self) -> &'static str {
        use Color::*;
        match self {
            Black           => "40",
            Red             => "41",
            Green           => "42",
            Yellow          => "43",
            Blue            => "44",
            Magenta         => "45",
            Cyan            => "46",
            White           => "47",
            BrightBlack     => "100",
            BrightRed       => "101",
            BrightGreen     => "102",
            BrightYellow    => "103",
            BrightBlue      => "104",
            BrightMagenta   => "105",
            BrightCyan      => "106",
            BrightWhite     => "107",
            None => "",
        }
    }
}

impl FormatSegment {
    pub fn to_ansi(&self) -> String {
        format!("{}{}\x1b[0m", match (&self.fg, &self.bg) {
            (Color::None, Color::None) => "".to_string(),
            (Color::None, one_of) | (one_of, Color::None) => format!("\x1b[{}m", one_of.ansi_fg()),
            (fg, bg) => format!("\x1b[{};{}m", fg.ansi_fg(), bg.ansi_bg())
        }, self.text)
    }
}
