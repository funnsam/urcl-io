#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::perf,
    clippy::nursery,
    clippy::suspicious,
    clippy::style,
)]
#![allow(
    clippy::semicolon_inside_block,
    clippy::just_underscores_and_digits,
)]

use urcl_io::emulator::{lexer::*, ast::*, parser::*, interpreter::*, error::*};
use std::{time::*, io::*};

fn main() {
    let src = std::fs::read_to_string("test.urcl").unwrap();

    let mut lex = Token::lexer(&src);
    let mut parser = match Parser::new(&mut lex) {
        Ok(p) => p,
        Err(errors) => {
            let segments = errors_to_formats(errors, &src);
            for s in segments {
                eprint!("{}", s.to_ansi());
            }

            std::process::exit(1);
        },
    };
    match parse(&mut parser) {
        Ok(()) => {},
        Err(errors) => {
            let segments = errors_to_formats(errors, &src);
            for s in segments {
                eprint!("{}", s.to_ansi());
            }

            std::process::exit(2);
        },
    }
    eprintln!("{:#?}", parser.ast);

    let mut interpreter = Interpreter::new(parser.ast);

    let start_int = Instant::now();
    let mut instructons = 0;

    let mut writer = BufWriter::with_capacity(16 * 0x20, stdout());

    loop {
        let step = interpreter.step(&mut writer);
        instructons += 1;
        if interpreter.debugging {
            eprintln!("{interpreter:#?}");
        }
        match step {
            StepResult::Halted => {
                let _ = writer.flush();

                let duration = start_int.elapsed().as_secs_f64();
                eprintln!("\x1b[1;32mInterpreter:\x1b[0m program halted (ran for {:.04}s / {:.01}Hz / {instructons} cycles)", duration, 1.0 / (duration / instructons as f64));
                break;
            },
            StepResult::Error(err) => {
                let _ = writer.flush();

                let segments = errors_to_formats(vec![err], &src);
                for s in segments {
                    eprint!("{}", s.to_ansi());
                }

                std::process::exit(3);
            }
            StepResult::Running => {},
        }
    }
}
