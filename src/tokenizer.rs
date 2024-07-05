use crate::MyError;
use std::collections::VecDeque;
use std::ops::Index;

#[derive(Debug, PartialEq)]
pub enum Token {
    TkReserved { raw: String },      // Keywords or punctuators
    TkNum { raw: String, val: i32 }, // Integer literals
    TkEof,                           // End-of-file markers
}

#[derive(Debug)]
pub struct TokenQueue(VecDeque<Token>);

impl Index<usize> for TokenQueue {
    type Output = Token;
    fn index<'a>(&'a self, i: usize) -> &'a Token {
        &self.0[i]
    }
}

impl TokenQueue {
    pub fn except_num(&mut self) -> Result<i32, MyError> {
        match self.0.pop_front() {
            Some(Token::TkNum { val, .. }) => Ok(val),
            _ => Err(MyError {
                info: "wrong token need TkNum".to_string(),
            })?,
        }
    }

    pub fn at_eof(&self) -> bool {
        self[0] == Token::TkEof
    }

    pub fn consume(&mut self, op: &str) -> Result<bool, MyError> {
        match self.0.front() {
            None => Err(MyError {
                info: format!("need {}, but no token left", op),
            }),
            Some(Token::TkReserved { raw }) if raw == op => {
                self.0.pop_front();
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    pub fn expect(&mut self, op: &str) -> Result<(), MyError> {
        if self.consume(op)? {
            Ok(())
        } else {
            Err(MyError {
                info: format!("expected '{}'", op),
            })
        }
    }

    fn skip_whitespace(&self, s: &str, i: &mut usize) {
        if *i >= s.len() {
            return;
        }
        let mut chars = s.chars().skip(*i);
        while let Some(c) = chars.next() {
            if c != ' ' {
                break;
            }
            *i += 1;
        }
    }

    fn extract_digit(&self, s: &str, i: &mut usize) -> Option<String> {
        if *i >= s.len() {
            return None;
        }
        let mut rv = String::new();
        let mut chars = s.chars().skip(*i);
        while let Some(c) = chars.next() {
            if c.is_digit(10) {
                rv.push(c);
                *i += 1;
            } else {
                break;
            }
        }
        if rv.is_empty() {
            return None;
        }
        Some(rv)
    }

    fn extract_op(&self, s: &str, i: &mut usize) -> Option<String> {
        if *i >= s.len() {
            return None;
        }
        if *i + 1 < s.len() {
            let double_rv = match &s[*i..*i + 2] {
                "==" => Some("==".to_string()),
                "!=" => Some("!=".to_string()),
                "<=" => Some("<=".to_string()),
                ">=" => Some(">=".to_string()),
                _ => None,
            };
            if double_rv.is_some() {
                *i += 2;
                return double_rv;
            }
        }
        let Some(c) = s.chars().nth(*i) else {
            return None;
        };
        match c {
            '+' | '-' | '*' | '/' | '(' | ')' | '<' | '>' => {
                *i += 1;
                return Some(c.to_string());
            }
            _ => None,
        }
    }

    fn generate_token(&mut self, s: &str, i: &mut usize) -> Result<(), MyError> {
        self.skip_whitespace(s, i);
        let num = self.extract_digit(s, i);
        if num.is_some() {
            let n = num.unwrap();
            self.0.push_back(Token::TkNum {
                val: n.parse::<i32>().map_err(|e| MyError {
                    info: e.to_string(),
                })?,
                raw: n,
            });
            return Ok(());
        }
        let op = self.extract_op(s, i);
        if op.is_some() {
            self.0.push_back(Token::TkReserved { raw: op.unwrap() });
            return Ok(());
        }

        if *i >= s.len() {
            Ok(())
        } else {
            Err(MyError {
                info: format!("unexpected character: {:?}, at {}", s.chars().nth(*i), *i),
            })
        }
    }

    fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn tokenizer(s: &str) -> Result<Self, MyError> {
        let mut rv = Self::new();
        let mut i = 0;
        while i < s.len() {
            rv.generate_token(s, &mut i)?;
        }
        rv.0.push_back(Token::TkEof);
        Ok(rv)
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;
    #[test]
    fn test_tokenizer_add() {
        let token_queue = TokenQueue::tokenizer("1+2");
        match token_queue {
            Ok(token_queue) => {
                assert_eq!(
                    token_queue[0],
                    Token::TkNum {
                        raw: "1".to_string(),
                        val: 1
                    }
                );
                assert_eq!(
                    token_queue[1],
                    Token::TkReserved {
                        raw: "+".to_string()
                    }
                );
                assert_eq!(
                    token_queue[2],
                    Token::TkNum {
                        raw: "2".to_string(),
                        val: 2
                    }
                );
                assert_eq!(token_queue[3], Token::TkEof);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    #[test]
    fn test_tokenizer_add_with_whitespace() {
        let token_queue = TokenQueue::tokenizer(" 1 + 2 ");
        match token_queue {
            Ok(token_queue) => {
                assert_eq!(
                    token_queue[0],
                    Token::TkNum {
                        raw: "1".to_string(),
                        val: 1
                    }
                );
                assert_eq!(
                    token_queue[1],
                    Token::TkReserved {
                        raw: "+".to_string()
                    }
                );
                assert_eq!(
                    token_queue[2],
                    Token::TkNum {
                        raw: "2".to_string(),
                        val: 2
                    }
                );
                assert_eq!(token_queue[3], Token::TkEof);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
    #[test]
    fn test_tokenizer_with_whitespace() {
        let token_queue = TokenQueue::tokenizer(" 12 + 34 - 5  +    2 ");
        match token_queue {
            Ok(token_queue) => {
                assert_eq!(
                    token_queue.0,
                    vec![
                        Token::TkNum {
                            raw: "12".to_string(),
                            val: 12
                        },
                        Token::TkReserved {
                            raw: "+".to_string()
                        },
                        Token::TkNum {
                            raw: "34".to_string(),
                            val: 34
                        },
                        Token::TkReserved {
                            raw: "-".to_string()
                        },
                        Token::TkNum {
                            raw: "5".to_string(),
                            val: 5
                        },
                        Token::TkReserved {
                            raw: "+".to_string()
                        },
                        Token::TkNum {
                            raw: "2".to_string(),
                            val: 2
                        },
                        Token::TkEof
                    ]
                );
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    #[test]
    fn test_tokenizer_mul() {
        let token_queue = TokenQueue::tokenizer("3+1*2");
        match token_queue {
            Ok(token_queue) => {
                assert_eq!(
                    token_queue.0,
                    vec![
                        Token::TkNum {
                            raw: "3".to_string(),
                            val: 3
                        },
                        Token::TkReserved {
                            raw: "+".to_string()
                        },
                        Token::TkNum {
                            raw: "1".to_string(),
                            val: 1
                        },
                        Token::TkReserved {
                            raw: "*".to_string()
                        },
                        Token::TkNum {
                            raw: "2".to_string(),
                            val: 2
                        },
                        Token::TkEof,
                    ]
                );
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    #[test]
    fn test_tokenizer_double_op() {
        let token_queue = TokenQueue::tokenizer("3+1==2");
        match token_queue {
            Ok(token_queue) => {
                assert_eq!(
                    token_queue.0,
                    vec![
                        Token::TkNum {
                            raw: "3".to_string(),
                            val: 3
                        },
                        Token::TkReserved {
                            raw: "+".to_string()
                        },
                        Token::TkNum {
                            raw: "1".to_string(),
                            val: 1
                        },
                        Token::TkReserved {
                            raw: "==".to_string()
                        },
                        Token::TkNum {
                            raw: "2".to_string(),
                            val: 2
                        },
                        Token::TkEof
                    ]
                );
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}
