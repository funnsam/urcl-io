use wasm_bindgen::prelude::*;

extern crate console_error_panic_hook;
use std::panic;

mod emulator;

#[wasm_bindgen(raw_module="../script.js")]
extern {
    pub fn now() -> f64;
}

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace=console)]
    fn log(s: &str);
}

#[macro_export]
macro_rules! logprintln {
    ($($arg:tt)*) => {{
        log(&format!($($arg)*).to_string());
    }};
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn test(src: &str) {
    use emulator::lexer::*;
    let tok = Token::lexer(src).collect::<Vec<Result<Token, ()>>>();
    logprintln!("{tok:#?}");
}

static mut RAND_SEED: u64 = 0;

pub fn rand() -> u64 {
    unsafe {
        let mut x = RAND_SEED;
        if x == 0 {x = now() as u64;}
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        RAND_SEED = x;
        x
    }
}

pub fn srand(seed: u64) {
    unsafe {
        RAND_SEED = seed;
    }
}
