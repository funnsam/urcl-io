#[cfg(not(target_arch = "wasm32"))]
pub fn now() -> f64 {
    use std::time::*;
    SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs_f64()
}

#[macro_export]
macro_rules! segment {
    ($text: expr, $fg: ident $bg: ident) => {
        FormatSegment {
            text: $text,
            fg: Color::$fg,
            bg: Color::$bg,
            styling: Styling {
                bold: false,
                italic: false,
                underline: false,
                blink: false,
            }
        }
    };
    ($text: expr, $fg: ident $bg: ident $sty: tt) => {
        FormatSegment {
            text: $text,
            fg: Color::$fg,
            bg: Color::$bg,
            styling: Styling {
                bold: stringify!($sty).contains('b'),
                italic: stringify!($sty).contains('i'),
                underline: stringify!($sty).contains('u'),
                blink: stringify!($sty).contains('f'),
            }
        }
    };
}

pub struct FormatSegment {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
    pub styling: Styling
}

pub enum Color {
    Black, Red, Green, Yellow, Blue, Magenta, Cyan, White,
    BrightBlack, BrightRed, BrightGreen, BrightYellow, BrightBlue, BrightMagenta, BrightCyan, BrightWhite,
    None
}

pub struct Styling {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
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

impl Styling {
    pub fn to_ansi(&self) -> String {
        let mut ansi = Vec::with_capacity(4);
        if self.bold        { ansi.push("1") }
        if self.italic      { ansi.push("3") }
        if self.underline   { ansi.push("4") }
        if self.blink       { ansi.push("5") }
        ansi.join(";")
    }

    pub fn is_some(&self) -> bool {
        self.bold | self.italic | self.underline | self.blink
    }
}

impl FormatSegment {
    pub fn to_ansi(&self) -> String {
        let mut ansi = Vec::with_capacity(3);
        if !matches!(self.fg, Color::None) { ansi.push(self.fg.ansi_fg().to_string()); }
        if !matches!(self.bg, Color::None) { ansi.push(self.bg.ansi_bg().to_string()); }
        if self.styling.is_some() { ansi.push(self.styling.to_ansi()); }

        format!("{}{}{}{}\x1b[0m",
            if ansi.is_empty() { "" } else { "\x1b[" },
            ansi.join(";"),
            if ansi.is_empty() { "" } else { "m" },
            self.text
        )
    }
}
