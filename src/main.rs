use chibicc_rust::CodeGenerator;
use chibicc_rust::MyError;
use chibicc_rust::Parser;
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
    let mut parser = Parser::new();
    let nodes = parser.program(&mut tokens)?;
    // Traverse the AST to emit assembly
    let mut generator = CodeGenerator::new(parser);
    generator.generate(nodes);
    Ok(())
}
