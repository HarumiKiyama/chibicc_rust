use crate::MyError;
use std::collections::VecDeque;
use std::ops::Index;

#[derive(Debug, PartialEq)]
pub enum Token {
    Reserved { keyword: String },  // Keywords or punctuators
    Num { raw: String, val: i32 }, // Integer literals
    Ident { name: String },        // Identifiers
    Eof,                           // End-of-file markers
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
    pub fn expect_num(&mut self) -> Result<i32, MyError> {
        match self.0.pop_front() {
            Some(Token::Num { val, .. }) => Ok(val),
            _ => Err(MyError {
                info: format!("expected Num, current tokens: {:?}", self.0),
            })?,
        }
    }

    pub fn expect_reserve(&mut self, op: &str) -> Result<(), MyError> {
        if self.consume_reserve(op)? {
            Ok(())
        } else {
            Err(MyError {
                info: format!("expected '{}', current tokens: {:?}", op, self.0),
            })
        }
    }

    pub fn at_eof(&self) -> bool {
        self[0] == Token::Eof
    }

    pub fn is_reserve(&self, op: &str) -> bool {
        matches!(&self[0], Token::Reserved { keyword: raw } if raw == op)
    }

    pub fn consume_reserve(&mut self, op: &str) -> Result<bool, MyError> {
        match self.0.front() {
            None => Err(MyError {
                info: format!("need {}, but no token left", op),
            }),
            Some(Token::Reserved { keyword: raw }) if raw == op => {
                self.0.pop_front();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn consume_ident(&mut self) -> Result<Option<String>, MyError> {
        if self.0.is_empty() {
            return Err(MyError {
                info: "no token left".to_string(),
            });
        }
        let found = matches!(self.0.front(), Some(Token::Ident { .. }));
        if found {
            let Some(Token::Ident { name }) = self.0.pop_front() else {
                Err(MyError {
                    info: "pop token error".to_string(),
                })?
            };
            Ok(Some(name))
        } else {
            Ok(None)
        }
    }

    fn is_alpha(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }

    fn is_alpha_num(c: char) -> bool {
        Self::is_alpha(c) || c.is_digit(10)
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

    fn extract_reserve(&self, s: &str, i: &mut usize) -> Option<String> {
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
            '+' | '-' | '*' | '/' | '(' | ')' | '<' | '>' | ';' | '=' | '{' | '}' | '&' => {
                *i += 1;
                return Some(c.to_string());
            }
            _ => None,
        }
    }

    fn extract_ident(&self, s: &str, i: &mut usize) -> Option<String> {
        let Some(c) = s.chars().nth(*i) else {
            return None;
        };
        if !Self::is_alpha(c) {
            return None;
        }
        let mut rv = c.to_string();
        *i += 1;
        let mut chars = s.chars().skip(*i);
        while let Some(c) = chars.next() {
            if Self::is_alpha_num(c) {
                rv.push(c);
                *i += 1;
            } else {
                break;
            }
        }
        if rv.is_empty() {
            None
        } else {
            Some(rv)
        }
    }

    fn generate_token(&mut self, s: &str, i: &mut usize) -> Result<(), MyError> {
        self.skip_whitespace(s, i);

        if let Some(num) = self.extract_digit(s, i) {
            self.0.push_back(Token::Num {
                val: num.parse::<i32>().map_err(|e| MyError {
                    info: e.to_string(),
                })?,
                raw: num,
            });
            return Ok(());
        }

        if let Some(reserve) = self.extract_reserve(s, i) {
            self.0.push_back(Token::Reserved { keyword: reserve });
            return Ok(());
        }

        if let Some(ident) = self.extract_ident(s, i) {
            match ident.as_str() {
                key @ ("return" | "if" | "else" | "for" | "while" | "int") => {
                    self.0.push_back(Token::Reserved {
                        keyword: key.to_string(),
                    });
                }
                _ => {
                    self.0.push_back(Token::Ident { name: ident });
                }
            }
            return Ok(());
        }

        if *i >= s.len() {
            Ok(())
        } else {
            Err(MyError {
                info: format!(
                    "unexpected character: {:?}, in {} at {}, token queue: {:?}",
                    s.chars().nth(*i),
                    s,
                    *i,
                    self.0
                ),
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
        rv.0.push_back(Token::Eof);
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
                    Token::Num {
                        raw: "1".to_string(),
                        val: 1
                    }
                );
                assert_eq!(
                    token_queue[1],
                    Token::Reserved {
                        keyword: "+".to_string()
                    }
                );
                assert_eq!(
                    token_queue[2],
                    Token::Num {
                        raw: "2".to_string(),
                        val: 2
                    }
                );
                assert_eq!(token_queue[3], Token::Eof);
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
                    Token::Num {
                        raw: "1".to_string(),
                        val: 1
                    }
                );
                assert_eq!(
                    token_queue[1],
                    Token::Reserved {
                        keyword: "+".to_string()
                    }
                );
                assert_eq!(
                    token_queue[2],
                    Token::Num {
                        raw: "2".to_string(),
                        val: 2
                    }
                );
                assert_eq!(token_queue[3], Token::Eof);
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
                        Token::Num {
                            raw: "12".to_string(),
                            val: 12
                        },
                        Token::Reserved {
                            keyword: "+".to_string()
                        },
                        Token::Num {
                            raw: "34".to_string(),
                            val: 34
                        },
                        Token::Reserved {
                            keyword: "-".to_string()
                        },
                        Token::Num {
                            raw: "5".to_string(),
                            val: 5
                        },
                        Token::Reserved {
                            keyword: "+".to_string()
                        },
                        Token::Num {
                            raw: "2".to_string(),
                            val: 2
                        },
                        Token::Eof
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
                        Token::Num {
                            raw: "3".to_string(),
                            val: 3
                        },
                        Token::Reserved {
                            keyword: "+".to_string()
                        },
                        Token::Num {
                            raw: "1".to_string(),
                            val: 1
                        },
                        Token::Reserved {
                            keyword: "*".to_string()
                        },
                        Token::Num {
                            raw: "2".to_string(),
                            val: 2
                        },
                        Token::Eof,
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
        let token_queue = TokenQueue::tokenizer("3+1==2").expect("tokenizer error");
        assert_eq!(
            token_queue.0,
            vec![
                Token::Num {
                    raw: "3".to_string(),
                    val: 3
                },
                Token::Reserved {
                    keyword: "+".to_string()
                },
                Token::Num {
                    raw: "1".to_string(),
                    val: 1
                },
                Token::Reserved {
                    keyword: "==".to_string()
                },
                Token::Num {
                    raw: "2".to_string(),
                    val: 2
                },
                Token::Eof
            ]
        );
    }
    #[test]
    fn test_tokenizer_return_assign() {
        let token_queue =
            TokenQueue::tokenizer("foo123=3; bar=5; return foo123+bar;").expect("tokenizer error");
        assert_eq!(
            token_queue.0,
            vec![
                Token::Ident {
                    name: "foo123".to_string()
                },
                Token::Reserved {
                    keyword: "=".to_string()
                },
                Token::Num {
                    raw: "3".to_string(),
                    val: 3
                },
                Token::Reserved {
                    keyword: ";".to_string()
                },
                Token::Ident {
                    name: "bar".to_string()
                },
                Token::Reserved {
                    keyword: "=".to_string()
                },
                Token::Num {
                    raw: "5".to_string(),
                    val: 5
                },
                Token::Reserved {
                    keyword: ";".to_string()
                },
                Token::Reserved {
                    keyword: "return".to_string()
                },
                Token::Ident {
                    name: "foo123".to_string()
                },
                Token::Reserved {
                    keyword: "+".to_string()
                },
                Token::Ident {
                    name: "bar".to_string()
                },
                Token::Reserved {
                    keyword: ";".to_string()
                },
                Token::Eof
            ]
        );
    }
}
