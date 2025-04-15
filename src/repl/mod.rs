mod compile;
mod eval;
mod eval_file;

use crate::{
    compiler::{symbol_table::SymbolTable, Compiler},
    lexer,
    object::{Object, ObjectType},
    parser::Parser,
    vm::{GLOBAL_SIZE, VM},
};
pub use compile::compile;
pub use eval::repl_start;
pub use eval_file::eval;
use std::{
    io::{self, Write},
    process::{self},
};

const PROMPT: &str = ">> ";

pub fn parse_errors(parser: &Parser) -> Result<(), ()> {
    if !parser.errors.is_empty() {
        eprintln!("{MONKEY_FACE}");
        eprintln!("Whoops! We ran into some monkey business here!");
        eprintln!("parser errors:");
        for error in parser.errors.iter() {
            eprintln!("\t{error}");
        }
        return Err(());
    }

    Ok(())
}

pub fn start(
    constants: &mut Vec<ObjectType>,
    symbol_table: SymbolTable,
    globals: &mut [ObjectType; GLOBAL_SIZE],
) -> SymbolTable {
    print!("{PROMPT}");
    let _ = io::stdout().flush();

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();

    if buffer.trim() == "/bye" || buffer.trim() == "exit" {
        process::exit(0)
    }

    let lexer = lexer::Lexer::new(&buffer);
    let mut parser = Parser::new(lexer);

    let program = parser.parse_program();

    match parse_errors(&parser) {
        Ok(_) => {}
        Err(_) => {
            return symbol_table;
        }
    };

    if let Ok(program) = program {
        let mut comp = Compiler::new(constants, symbol_table);
        if comp.compile(program).is_err() {
            eprintln!("woops! compilation failed");
        }

        let symbols = comp.symbol_table.clone();
        let mut machine = VM::new(comp, globals);
        if let Err(e) = machine.run() {
            eprintln!("whoops! executing the bytecode failed:, {e}");
        }

        let stack_top = machine.last_popped_stack_elem();
        println!("{}", stack_top.inspect());

        return symbols;
    }

    symbol_table
}

const MONKEY_FACE: &str = r#"            __,__
   .--.  .-"     "-.  .--.
  / .. \/  .-. .-.  \/ .. \
 | |  '|  /   Y   \  |'  | |
 | \   \  \ 0 | 0 /  /   / |
  \ '- ,\.-"""""""-./, -' /
   ''-' /_   ^ ^   _\ '-''
       |  \._   _./  |
       \   \ '~' /   /
        '._ '-=-' _.'
           '-----'
"#;
