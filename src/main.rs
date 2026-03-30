use crate::{
    compiler::compiler::Compiler,
    language::{ast::AST, lexer::Lexer, parser::Parser},
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

    let text = fs::read_to_string("sigma.ign")?;
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

    let mut ast = AST::new(nodes);
    ast.prune_ast();
    let nodes = ast.nodes;

    if args.contains(&"nodes".to_string()) {
        println!("Generated Nodes:");
        println!("---------------------------");
        println!("{nodes:#?}");
    }

    /////////////////////
    // COMPILER
    /////////////////////

    let mut vm = VM::new();
    if args.contains(&"bc".to_string()) {
        vm.read_bytecode_file("bytecode.igb");
    } else if args.contains(&"bc2".to_string()) {
        vm.read_bytecode_file("bytecode2.igb");
    } else {
        let mut compiler = Compiler::new();
        for i in nodes.iter() {
            compiler.compile_node(i);
        }
        if args.contains(&"opt".to_string()) {
            vm.constants = compiler.constants.clone();
            vm.instructions = compiler.instructions.clone();

            if args.contains(&"inst".to_string()) {
                println!("\n[Pre-optimization] Compiled instructions:");
                println!("---------------------------");
                vm.print_instructions();
            }

            compiler.optimize();
        }

        if args.contains(&"no_debug".to_string()) {
            compiler.finalize_bytecode();
        }

        vm.constants = compiler.constants;
        vm.instructions = compiler.instructions;
    }

    if args.contains(&"inst".to_string()) {
        println!("\nCompiled instructions:");
        println!("---------------------------");
        vm.print_instructions();
    }

    if args.contains(&"bytecode".to_string()) {
        vm.write_bytecode_file("bytecode.igb");
    } else if args.contains(&"bytecode2".to_string()) {
        vm.write_bytecode_file("bytecode2.igb");
    } else {
        println!("\nRunning:");
        println!("---------------------------");
        vm.run(false, false);

        if args.contains(&"stack".to_string()) {
            println!("\nOutput VM Stack:");
            println!("---------------------------");
            println!("{:#?}", vm.stack);
        }
    }

    Ok(())
}
