use chibicc_rust::CodeGenerator;
use chibicc_rust::MyError;
use chibicc_rust::Node;
use chibicc_rust::TokenQueue;
use std::env;


fn main() -> Result<(), MyError> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 1 {
        Err(MyError {
            info: format!("args error {:?}", args),
        })?;
    }
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("  main:");
    let arg = &args[0];
    let mut tokens = TokenQueue::tokenizer(&arg)?;
    let node = Node::expr(&mut tokens)?;
    let code_generator = CodeGenerator::new();
    code_generator.generate(Some(&node));
    println!("  pop rax");
    println!("  ret");
    Ok(())
}
