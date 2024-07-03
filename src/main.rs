use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 1 {
        Err(format!("args error {:?}", args))?;
    }
    let arg = &args[0];
    println!("  .globl main");
    println!("main:");
    println!("  mov ${}, %rax", arg);
    println!("  ret");
    Ok(())
}
