use std::collections::{HashMap, VecDeque};

use crate::{MyError, TokenQueue};

#[derive(PartialEq, Debug)]
pub enum Node {
    Add {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // +

    Sub {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // -
    Mul {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // *
    Div {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // /
    Neg {
        lhs: Box<Node>,
    }, // unary -
    Eq {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // ==
    Ne {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // !=
    Lt {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // <
    Le {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // <=
    Assign {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // =
    Return {
        lhs: Option<Box<Node>>,
    }, // "return"
    If {
        cond: Box<Node>,
        then: Option<Box<Node>>,
        els: Option<Box<Node>>,
    }, // "if"
    For {
        init: Option<Box<Node>>,
        cond: Option<Box<Node>>,
        inc: Option<Box<Node>>,
        then: Option<Box<Node>>,
    }, // "for" and "while"
    Block {
        nodes: Vec<Node>,
    }, // { ... }
    ExprStmt {
        expr: Box<Node>,
    }, // Expression statement
    Var {
        name: String,
    }, // Local variable
    Num {
        val: i32,
    }, // Integer
}

type ParseResult = Result<Node, MyError>;

type VarTable = HashMap<String, usize>; // variable name offset hashtable

pub struct Parser {
    pub locals: VarTable,
    pub locals_dequeue: VecDeque<String>,
    pub stack_size: usize,
    pub nodes: Vec<Node>,
    pub token_queue: TokenQueue,
}

impl Node {
    fn new_num(val: i32) -> Self {
        Node::Num { val }
    }
    fn new_var(name: String) -> Self {
        Node::Var { name }
    }
}

impl Parser {
    pub fn new(token_queue: TokenQueue) -> Self {
        Self {
            locals: HashMap::new(),
            locals_dequeue: VecDeque::new(),
            stack_size: 0,
            nodes: Vec::new(),
            token_queue,
        }
    }

    fn push_var(&mut self, name: String) -> usize {
        match self.locals.get(&name) {
            None => {
                self.locals_dequeue.push_front(name.clone());
                self.locals.insert(name, self.locals_dequeue.len() * 8);
            }
            _ => {}
        };
        self.locals_dequeue.len() * 8
    }

    // program = stmt*
    pub fn program(&mut self) -> Result<Vec<Node>, MyError> {
        let mut nodes = Vec::new();
        while !self.token_queue.at_eof() {
            nodes.push(self.stmt()?);
        }
        Ok(nodes)
    }

    // stmt = "return" expr ";"
    //      | "if" "(" expr ")" stmt ("else" stmt)?
    //      | "for" "(" expr-stmt expr? ";" expr? ")" stmt
    //      | "while" "(" expr ")" stmt
    //      | "{" compound-stmt
    //      | expr-stmt
    fn stmt(&mut self) -> ParseResult {
        // RETURN NODE
        if self.token_queue.consume_reserve("return")? {
            let node = Node::Return {
                lhs: Some(Box::new(self.expr()?)),
            };
            self.token_queue.expect_reserve(";")?;
            return Ok(node);
        }

        //      | "if" "(" expr ")" stmt ("else" stmt)?
        // IF NODE
        if self.token_queue.consume_reserve("if")? {
            self.token_queue.expect_reserve("(")?;
            let cond = self.expr()?;
            self.token_queue.expect_reserve(")")?;
            let then = self.stmt()?;
            let mut els = None;
            if self.token_queue.consume_reserve("else")? {
                els = Some(Box::new(self.stmt()?));
            }
            return Ok(Node::If {
                cond: Box::new(cond),
                then: Some(Box::new(then)),
                els,
            });
        }

        //      | "for" "(" expr-stmt expr? ";" expr? ")" stmt
        // FOR NODE
        if self.token_queue.consume_reserve("for")? {
            self.token_queue.expect_reserve("(")?;
            let init = self.expr_stmt()?;
            let cond = if self.token_queue.consume_reserve(";")? {
                None
            } else {
                let cond = self.expr()?;
                self.token_queue.expect_reserve(";")?;
                Some(Box::new(cond))
            };
            let inc = if self.token_queue.consume_reserve(")")? {
                None
            } else {
                let node = self.expr()?;
                self.token_queue.expect_reserve(")")?;
                Some(Box::new(node))
            };
            let then = self.stmt()?;
            return Ok(Node::For {
                init: Some(Box::new(init)),
                cond,
                inc,
                then: Some(Box::new(then)),
            });
        }

        //      | "while" "(" expr ")" stmt
        // WHILE NODE
        if self.token_queue.consume_reserve("while")? {
            self.token_queue.expect_reserve("(")?;
            let cond = self.expr()?;
            self.token_queue.expect_reserve(")")?;
            let then = self.stmt()?;
            return Ok(Node::For {
                init: None,
                inc: None,
                cond: Some(Box::new(cond)),
                then: Some(Box::new(then)),
            });
        }

        // block node
        if self.token_queue.consume_reserve("{")? {
            return self.compound_stmt();
        }
        return self.expr_stmt();
    }

    // compound-stmt = stmt* "}"
    fn compound_stmt(&mut self) -> ParseResult {
        let mut nodes = Vec::new();
        while !self.token_queue.consume_reserve("}")? {
            nodes.push(self.stmt()?);
        }
        Ok(Node::Block { nodes })
    }

    // expr-stmt = expr? ";"
    fn expr_stmt(&mut self) -> ParseResult {
        if self.token_queue.consume_reserve(";")? {
            return Ok(Node::Block { nodes: Vec::new() });
        };
        let node = self.expr()?;
        self.token_queue.expect_reserve(";")?;
        return Ok(Node::ExprStmt {
            expr: Box::new(node),
        });
    }
    // expr = assign
    fn expr(&mut self) -> ParseResult {
        self.assign()
    }

    // assign = equality ("=" assign)?
    fn assign(&mut self) -> ParseResult {
        let mut node = self.equality()?;
        if self.token_queue.consume_reserve("=")? {
            node = Node::Assign {
                lhs: Box::new(node),
                rhs: Box::new(self.assign()?),
            };
        }
        Ok(node)
    }

    // equality = relational ("==" relational | "!=" relational)*
    fn equality(&mut self) -> ParseResult {
        let mut node = self.relational()?;
        loop {
            if self.token_queue.consume_reserve("==")? {
                node = Node::Eq {
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()?),
                };
            } else if self.token_queue.consume_reserve("!=")? {
                node = Node::Ne {
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()?),
                };
            } else {
                return Ok(node);
            }
        }
    }

    // relational = add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self) -> ParseResult {
        let mut node = self.add()?;
        loop {
            if self.token_queue.consume_reserve("<")? {
                node = Node::Lt {
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()?),
                };
            } else if self.token_queue.consume_reserve("<=")? {
                node = Node::Le {
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()?),
                };
            } else if self.token_queue.consume_reserve(">")? {
                node = Node::Lt {
                    lhs: Box::new(self.add()?),
                    rhs: Box::new(node),
                };
            } else if self.token_queue.consume_reserve(">=")? {
                node = Node::Le {
                    lhs: Box::new(self.add()?),
                    rhs: Box::new(node),
                };
            } else {
                return Ok(node);
            }
        }
    }

    // add = mul ("+" mul | "-" mul)*
    fn add(&mut self) -> ParseResult {
        let mut node = self.mul()?;
        loop {
            if self.token_queue.consume_reserve("+")? {
                node = Node::Add {
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()?),
                };
            } else if self.token_queue.consume_reserve("-")? {
                node = Node::Sub {
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()?),
                };
            } else {
                return Ok(node);
            }
        }
    }
    // mul = unary ("*" unary | "/" unary)*
    fn mul(&mut self) -> ParseResult {
        let mut node = self.unary()?;
        loop {
            if self.token_queue.consume_reserve("*")? {
                node = Node::Mul {
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()?),
                };
            } else if self.token_queue.consume_reserve("/")? {
                node = Node::Div {
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()?),
                };
            } else {
                return Ok(node);
            }
        }
    }

    // unary = ("+" | "-")? unary
    //       | primary
    fn unary(&mut self) -> ParseResult {
        if self.token_queue.consume_reserve("+")? {
            return self.unary();
        }
        if self.token_queue.consume_reserve("-")? {
            let node = Node::Neg {
                lhs: Box::new(self.unary()?),
            };
            return Ok(node);
        }
        return self.primary();
    }

    // primary = "(" expr ")" | ident | num
    fn primary(&mut self) -> ParseResult {
        if self.token_queue.consume_reserve("(")? {
            let node = self.expr()?;
            self.token_queue.expect_reserve(")")?;
            return Ok(node);
        }
        if let Ok(Some(name)) = self.token_queue.consume_ident() {
            self.push_var(name.clone());
            Ok(Node::new_var(name))
        } else {
            Ok(Node::new_num(self.token_queue.except_num()?))
        }
    }

    pub fn assign_lvar_offset(&mut self) {
        let offset = self.locals_dequeue.len() * 8;
        self.stack_size = Self::align_to(offset, 16);
        for (i, name) in self.locals_dequeue.iter().enumerate() {
            let v = self.locals.get_mut(name).expect("local variable get error");
            *v = (i + 1) * 8;
        }
    }

    fn align_to(n: usize, align: usize) -> usize {
        (n + align - 1) / align * align
    }
}
