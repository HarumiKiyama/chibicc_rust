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
    let arg = &args[0];
    // Tokenize
    let mut tokens = TokenQueue::tokenizer(&arg)?;
    // Parse
    let nodes = Node::program(&mut tokens)?;

    // Traverse the AST to emit assembly
    let code_generator = CodeGenerator::new();
    code_generator.generate(nodes);
    Ok(())
}
