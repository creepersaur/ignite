use crate::{
    compiler::compiler::Compiler,
    language::{lexer::Lexer, parser::Parser},
    virtual_machine::vm::VM,
};
#[allow(unused)]
use std::{error::Error, fs, rc::Rc};

mod compiler;
mod language;
mod macros;
mod misc;
mod virtual_machine;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();

    let text = fs::read_to_string("sigma.ignite")?;
    let mut lex = Lexer::new(&text);
    let tokens = lex.get_tokens();

    if args.contains(&"tokens".to_string()) {
        println!("Lexed Tokens:");
        println!("---------------------------");
        println!("{:#?}", tokens);
    }

    /////////////////////
    // NODES
    /////////////////////

    let mut parser = Parser::new(text, tokens);
    let mut nodes = vec![];

    while parser.current().is_ok() {
        nodes.push(parser.parse()?);
    }

    if args.contains(&"nodes".to_string()) {
        println!("Generated Nodes:");
        println!("---------------------------");
        println!("{nodes:#?}");
    }

    /////////////////////
    // COMPILER
    /////////////////////

    let mut compiler = Compiler::new();
    for i in nodes.iter() {
        compiler.compile_node(i);
    }

    let mut vm = VM::new();
    vm.constants = compiler.constants;
    vm.instructions = compiler.instructions;

    if args.contains(&"inst".to_string()) {
        println!("\nCompiled instructions:");
        println!("---------------------------");
        vm.print_instructions();
    }

    println!("\nRunning:");
    println!("---------------------------");
    vm.run(false, false);

    Ok(())
}
