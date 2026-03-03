use grift::Lisp;
use wasm_bindgen::prelude::*;

const ARENA_SIZE: usize = 100_000;

/// A persistent Grift Lisp REPL session backed by a fixed-size arena.
#[wasm_bindgen]
pub struct GriftRepl {
    lisp: Lisp<ARENA_SIZE>,
}

#[wasm_bindgen]
impl GriftRepl {
    /// Create a new REPL session with a fresh interpreter.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        GriftRepl {
            lisp: Lisp::new(),
        }
    }

    /// Evaluate a Grift expression and return its printed representation.
    pub fn eval(&self, input: &str) -> String {
        match self.lisp.eval_to_index(input) {
            Ok(idx) => {
                let mut buf = String::new();
                let _ = self.lisp.write_value(idx, &mut buf);
                buf
            }
            Err(e) => format!("error: {e:?}"),
        }
    }
}
