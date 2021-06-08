#![allow(dead_code)]
#![allow(unused_variables)]
mod backend;
mod error;
mod frontend;

use std::env;
use std::process;

use crate::backend::vm::VM;

fn main() {
    let mut vm = VM::new(true);
    let args: Vec<String> = env::args().collect();
    let result = match args.len() {
        1 => vm.repl().map_err(|e| format!("{}", e)),
        2 => vm.run_file(&args[1]).map_err(|e| format!("{}", e)),
        _ => Err(String::from("Usage: blox [path]")),
    };

    if let Err(message) = result {
        eprintln!("{}", message);
        process::exit(1);
    }
}
