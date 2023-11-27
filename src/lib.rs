#![feature(concat_idents)]

pub mod compiler;

pub mod utils;
pub use utils::*;

#[cfg(target_arch = "wasm32")]
use std::panic;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(raw_module="../script.js")]
extern {
    pub fn now() -> f64;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace=console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! logprintln {
    ($($arg:tt),*) => {{
        $crate::log(&format!($($arg),*).to_string());
    }};
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn test(src: &str) {
    todo!()
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
