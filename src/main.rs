use std::collections::VecDeque;
use std::env;
use std::mem;
use std::ops::Index;

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

#[derive(Debug, PartialEq)]
enum Token {
    TkReserved { raw: String },      // Keywords or punctuators
    TkNum { raw: String, val: i32 }, // Integer literals
    TkEof,                           // End-of-file markers
}

#[derive(Debug)]
struct TokenQueue(VecDeque<Token>);

impl Index<usize> for TokenQueue {
    type Output = Token;
    fn index<'a>(&'a self, i: usize) -> &'a Token {
        &self.0[i]
    }
}

impl TokenQueue {
    fn except_num(&mut self) -> Result<i32, MyError> {
        match self.0.pop_front() {
            Some(Token::TkNum { val, .. }) => {Ok(val)}
            _ => Err(MyError {
                info: "wrong token need TkNum".to_string(),
            })?,
        }
    }

    fn comsume(&mut self, op: char) -> Result<bool, MyError> {
        match self.0.front() {
            Some(Token::TkReserved { raw }) => {
                if raw == &op.to_string() {
                    self.0.pop_front();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Err(MyError {
                info: "not a TkReserved token".to_string(),
            }),
        }
    }

    fn tokenizer(s: &str) -> Result<Self, MyError> {
        let mut tokens: VecDeque<Token> = VecDeque::new();
        let mut chars = s.chars();
        let mut num = String::new();
        while let Some(c) = chars.next() {
            match c {
                ' ' => {}
                '0'..='9' => {
                    num.push(c);
                }
                '+' | '-' => {
                    tokens.push_back(Token::TkNum {
                        val: num.parse::<i32>().map_err(|e| MyError {
                            info: e.to_string(),
                        })?,
                        raw: mem::take(&mut num),
                    });
                    tokens.push_back(Token::TkReserved { raw: c.to_string() });
                }
                _ => {
                    Err(MyError {
                        info: format!("invalid char {}", c),
                    })?;
                }
            }
        }

        if num.len() > 0 {
            tokens.push_back(Token::TkNum {
                val: num.parse::<i32>().map_err(|e| MyError {
                    info: e.to_string(),
                })?,
                raw: num,
            });
        }

        tokens.push_back(Token::TkEof);
        Ok(Self(tokens))
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
    let mut tokens = TokenQueue::tokenizer(arg)?;
    println!("  mov rax, {}", tokens.except_num()?);
    while tokens[0] != Token::TkEof {
        if tokens.comsume('+')? {
            println!("  add rax, {}", tokens.except_num()?);
        } else if tokens.comsume('-')? {
            println!("  sub rax, {}", tokens.except_num()?);
        }
    }
    println!("  ret");
    Ok(())
}
