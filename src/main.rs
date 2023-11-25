use urcl_io::emulator::{lexer::*, ast::*, parser::*, interpreter::*, error::*};

fn main() {
    let src = std::fs::read_to_string("test.urcl").unwrap();

    let mut lex = Token::lexer(&src);
    let mut parser = Parser::new(&mut lex).unwrap();
    match parse(&mut parser) {
        Ok(()) => {},
        Err(errors) => {
            let segments = errors_to_formats(errors, &src);
            for s in segments {
                print!("{}", s.to_ansi());
            }

            std::process::exit(1);
        },
    }
    println!("{:#?}", parser.ast);

    return;

    let mut interpreter = Interpreter::new(parser.ast);
    loop {
        let step = interpreter.step();
        println!("{interpreter:#?}");
        match step {
            StepResult::Halted => {
                println!("Interpreter halted");
                break;
            },
            StepResult::Error(err) => {
                println!("Interpreter error: {}\n    at: {:?}", err.kind.message(), err.span);
                break;
            }
            StepResult::Running => {},
        }
    }
}
