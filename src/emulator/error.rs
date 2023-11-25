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
