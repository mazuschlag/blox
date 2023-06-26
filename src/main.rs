mod backend;
mod error;
mod frontend;

use std::{env, process};

use crate::{backend::vm::Vm, error::codes::ErrCode};

const DEBUG_PRINT_ARG: &str = "-p";
const DEBUG_TRACE_ARG: &str = "-t";

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut vm = Vm::new();

    let result = match args.len() {
        1 => vm.repl(),
        2 => vm.run_file(&args[1], false, false),
        3 => {
            if args[2] == DEBUG_PRINT_ARG {
                vm.run_file(&args[1], true, false)
            } else if args[2] == DEBUG_TRACE_ARG {
                vm.run_file(&args[1], false, true)
            } else {
                Err(ErrCode::Io(format!("Unrecognized arg {}", args[2])))
            }
        }
        4 => {
            if (args[2] == DEBUG_PRINT_ARG || args[3] == DEBUG_PRINT_ARG)
                && (args[2] == DEBUG_TRACE_ARG || args[3] == DEBUG_TRACE_ARG)
            {
                vm.run_file(&args[1], true, true)
            } else {
                Err(ErrCode::Io(format!(
                    "Unrecognized arg {} {}",
                    args[3], args[4]
                )))
            }
        }
        _ => Err(ErrCode::Io(String::from("Usage: blox [path] -p? -t?"))),
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
