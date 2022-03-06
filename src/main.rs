mod backend;
mod error;
mod frontend;

use std::env;
use std::process;

use crate::backend::vm::Vm;
use crate::error::codes::ErrCode;

// Debug flags
const DEBUG_TRACE: bool = false;
const DEBUG_PRINT_CODE: bool = false;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut vm = Vm::new();

    let result = match args.len() {
        1 => vm.repl(),
        2 => vm.run_file(&args[1]),
        _ => Err(ErrCode::Io(String::from("Usage: blox [path]"))),
    };

    match result {
        Err(ErrCode::Compile) => process::exit(65),
        Err(ErrCode::Runtime(e)) => {
            eprintln!("{}", e);
            process::exit(70);
        }
        Err(ErrCode::Io(e)) => {
            eprintln!("{}", e);
            process::exit(74);
        }
        _ => (),
    }
}
