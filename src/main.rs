use crate::{
    compiler::compiler::Compiler,
    language::{ast::AST, lexer::Lexer, parser::Parser},
    virtual_machine::vm::VM,
};
use std::{cell::RefCell, panic::{AssertUnwindSafe, catch_unwind}};
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
    if args.contains(&"opt".to_string()) {
        ast.optimize();
    }
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
    if args.contains(&"no_expose".to_string()) {
        vm.expose_interns = false;
    }

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
            vm.instructions = rc!(RefCell::new(compiler.instructions.clone()));
            vm.intern_table = compiler.intern_table.clone();

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
        vm.instructions = rc!(RefCell::new(compiler.instructions));
        vm.intern_table = compiler.intern_table;
    }

    if args.contains(&"pre_run".to_string()) {
        vm.pre_run_pass();
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
        let instructions_clone = vm.instructions.clone();

        if args.contains(&"bench".to_string()) {
            bench(&mut vm);
        } else {
            let _ = catch_unwind(AssertUnwindSafe(|| vm.run(false, false)));
        }

        if args.contains(&"trace".to_string()) {
            if vm.pos < instructions_clone.borrow().len() {
                println!(
                    "Last Instruction ({}): {:?}",
                    vm.pos, instructions_clone.borrow()[vm.pos]
                );
            } else {
                println!("Completed all instructions")
            }
        }
    }

    Ok(())
}

fn bench(vm: &mut VM) {
    let runs = 1000;
    let start = std::time::Instant::now();
    for _ in 0..runs {
        let _ = catch_unwind(AssertUnwindSafe(|| vm.run(false, false)));
    }
    let avg = start.elapsed() / runs;
    println!("avg: {:?}", avg);
}
