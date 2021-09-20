mod backend;
mod error;
mod frontend;

use std::env;
use std::process;

use crate::backend::vm::Vm;

// Debug flags
const DEBUG_TRACE: bool = false;
const DEBUG_PRINT_CODE: bool = false;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut vm = Vm::new();

    let success = match args.len() {
        1 => vm.repl(),
        2 => vm.run_file(&args[1]),
        _ => {
            eprintln!("Usage: blox [path]");
            false
        }
    };

    if !success {
        process::exit(1);
    }
}
