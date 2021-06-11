#![allow(dead_code)]
#![allow(unused_variables)]
mod backend;
mod error;
mod frontend;

use std::env;
use std::process;

use crate::backend::vm::Vm;
use crate::error::codes::ErrCode;

fn main() {
    let mut vm = Vm::new(true);
    let args: Vec<String> = env::args().collect();
    let result = match args.len() {
        1 => vm.repl(),
        2 => vm.run_file(&args[1]),
        _ => {
            eprintln!("Usage: blox [path]");
            Err(ErrCode::RuntimeError)
        },
    };

    if let Err(_) = result {
        process::exit(1);
    }
}
