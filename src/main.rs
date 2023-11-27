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

use urcl_io::compiler::{
    frontend::{lexer::*, ast::*, parser::*},
    backend::{codegen::*, arch::interpreter::*},
    error::*,
};
use std::{time::*, io::*};
use thousands::Separable;

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

    let ssa = generate_ssa(parser.ast);

    eprintln!("{}", ssa.0);

    let mut interpreter = Interpreter::new(ssa);

    let start_int = Instant::now();

    let mut stdout = BufWriter::with_capacity(16 * 0x20, stdout());
    let mut stdin  = stdin();

    loop {
        let step = interpreter.step(&mut stdout, &mut stdin);
        // if interpreter.debugging {
            // println!("{interpreter:#?}");
        // }
        match step {
            StepResult::Halted => {
                let _ = stdout.flush();

                let duration = start_int.elapsed().as_secs_f64();
                eprintln!(
                    "\x1b[1;32mInterpreter:\x1b[0m program halted (ran for {}s / {}Hz / {} cycles)",
                    (duration).separate_with_commas(),
                    (interpreter.inst_count as f64 / duration).separate_with_commas(),
                    interpreter.inst_count.separate_with_commas(),
                );
                std::process::exit(0);
            },
            StepResult::Error(err) => {
                let _ = stdout.flush();

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
