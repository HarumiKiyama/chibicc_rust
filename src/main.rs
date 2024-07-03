use std::env;

#[derive(Debug)]
struct MyError {
    info: String,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MyError: {}", self.info)
    }
}

impl std::error::Error for MyError {
    fn description(&self) -> &str {
        &self.info
    }
}

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
    let mut nums: Vec<String> = vec![];
    let mut chars = arg.chars();
    let mut num = String::new();
    while let Some(c) = chars.next() {
        match c {
            '0'..='9' => {
                num.push(c);
            }
            '+' | '-' => {
                nums.push(num.clone());
                num.clear();
                nums.push(c.to_string());
            }
            _ => {
                Err(MyError {
                    info: format!("invalid char {}", c),
                })?;
            }
        }
    }
    if num.len() > 0 {
        nums.push(num);
    }
    println!("  mov rax, {}", nums[0]);
    for num in nums.iter().skip(1) {
        match num.as_str() {
            "+" => {
                print!("  add rax, ");
            }
            "-" => {
                print!("  sub rax, ");
            }
            _ => {
                println!("{}", num);
            }
        }
    }
    println!("  ret");
    Ok(())
}
