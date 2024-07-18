use std::collections::{HashMap, VecDeque};

use crate::{MyError, TokenQueue};

#[derive(PartialEq, Debug)]
pub enum NodeKind {
    Add,                  // +
    Sub,                  // -
    Mul,                  // *
    Div,                  // /
    Neg,                  // unary -
    Eq,                   // ==
    Ne,                   // !=
    Lt,                   // <
    Le,                   // <=
    Assign,               // =
    Return,               // "return"
    ExprStmt,             // Expression statement
    Var { name: String }, // Local variable
    Num { val: i32 },     // Integer
}

type ParseResult = Result<Node, MyError>;

type VarTable = HashMap<String, usize>; // variable name offset hashtable

pub struct Parser {
    pub locals: VarTable,
    pub locals_dequeue: VecDeque<String>,
    pub stack_size: usize,
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,         // Node kind
    pub lhs: Option<Box<Node>>, // Left-hand side
    pub rhs: Option<Box<Node>>, // Right-hand side
}

impl Node {
    fn new_num(val: i32) -> Self {
        Node {
            kind: NodeKind::Num { val },
            lhs: None,
            rhs: None,
        }
    }

    fn new_binary(kind: NodeKind, lhs: Node, rhs: Node) -> Self {
        Node {
            kind,
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
        }
    }

    fn new_unary(kind: NodeKind, lhs: Node) -> Self {
        Node {
            kind,
            lhs: Some(Box::new(lhs)),
            rhs: None,
        }
    }

    fn new_var(name: String) -> Self {
        Node {
            kind: NodeKind::Var { name },
            lhs: None,
            rhs: None,
        }
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            locals_dequeue: VecDeque::new(),
            stack_size: 0,
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
    pub fn program(&mut self, token_queue: &mut TokenQueue) -> Result<Vec<Node>, MyError> {
        let mut nodes = Vec::new();
        while !token_queue.at_eof() {
            nodes.push(self.stmt(token_queue)?);
        }
        Ok(nodes)
    }

    // stmt = "return" expr ";"
    //        | expr ";"
    fn stmt(&mut self, token_queue: &mut TokenQueue) -> ParseResult {
        if token_queue.consume_reserve("return")? {
            let node = Node::new_unary(NodeKind::Return, self.expr(token_queue)?);
            token_queue.expect_reserve(";")?;
            return Ok(node);
        }
        let node = self.expr(token_queue)?;
        token_queue.expect_reserve(";")?;
        Ok(Node::new_unary(NodeKind::ExprStmt, node))
    }

    // expr = assign
    fn expr(&mut self, token_queue: &mut TokenQueue) -> ParseResult {
        self.assign(token_queue)
    }

    // assign = equality ("=" assign)?
    fn assign(&mut self, token_queue: &mut TokenQueue) -> ParseResult {
        let mut node = self.equality(token_queue)?;
        if token_queue.consume_reserve("=")? {
            node = Node::new_binary(NodeKind::Assign, node, self.assign(token_queue)?);
        }
        Ok(node)
    }

    // equality = relational ("==" relational | "!=" relational)*
    fn equality(&mut self, token_queue: &mut TokenQueue) -> ParseResult {
        let mut node = self.relational(token_queue)?;
        loop {
            if token_queue.consume_reserve("==")? {
                node = Node::new_binary(NodeKind::Eq, node, self.relational(token_queue)?);
            } else if token_queue.consume_reserve("!=")? {
                node = Node::new_binary(NodeKind::Ne, node, self.relational(token_queue)?);
            } else {
                return Ok(node);
            }
        }
    }

    // relational = add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self, token_queue: &mut TokenQueue) -> ParseResult {
        let mut node = self.add(token_queue)?;
        loop {
            if token_queue.consume_reserve("<")? {
                node = Node::new_binary(NodeKind::Lt, node, self.add(token_queue)?);
            } else if token_queue.consume_reserve("<=")? {
                node = Node::new_binary(NodeKind::Le, node, self.add(token_queue)?);
            } else if token_queue.consume_reserve(">")? {
                node = Node::new_binary(NodeKind::Lt, self.add(token_queue)?, node);
            } else if token_queue.consume_reserve(">=")? {
                node = Node::new_binary(NodeKind::Le, self.add(token_queue)?, node);
            } else {
                return Ok(node);
            }
        }
    }

    // add = mul ("+" mul | "-" mul)*
    fn add(&mut self, token_queue: &mut TokenQueue) -> ParseResult {
        let mut node = self.mul(token_queue)?;
        loop {
            if token_queue.consume_reserve("+")? {
                node = Node::new_binary(NodeKind::Add, node, self.mul(token_queue)?);
            } else if token_queue.consume_reserve("-")? {
                node = Node::new_binary(NodeKind::Sub, node, self.mul(token_queue)?);
            } else {
                return Ok(node);
            }
        }
    }
    // mul = unary ("*" unary | "/" unary)*
    fn mul(&mut self, token_queue: &mut TokenQueue) -> ParseResult {
        let mut node = self.unary(token_queue)?;
        loop {
            if token_queue.consume_reserve("*")? {
                node = Node::new_binary(NodeKind::Mul, node, self.unary(token_queue)?);
            } else if token_queue.consume_reserve("/")? {
                node = Node::new_binary(NodeKind::Div, node, self.unary(token_queue)?);
            } else {
                return Ok(node);
            }
        }
    }

    // unary = ("+" | "-")? unary
    //       | primary
    fn unary(&mut self, token_queue: &mut TokenQueue) -> ParseResult {
        if token_queue.consume_reserve("+")? {
            return self.unary(token_queue);
        }
        if token_queue.consume_reserve("-")? {
            let node = Node::new_binary(NodeKind::Neg, Node::new_num(0), self.unary(token_queue)?);
            return Ok(node);
        }
        return self.primary(token_queue);
    }

    // primary = "(" expr ")" | ident | num
    fn primary(&mut self, token_queue: &mut TokenQueue) -> ParseResult {
        if token_queue.consume_reserve("(")? {
            let node = self.expr(token_queue)?;
            token_queue.expect_reserve(")")?;
            return Ok(node);
        }
        if let Ok(Some(name)) = token_queue.consume_ident() {
            self.push_var(name.clone());
            Ok(Node::new_var(name))
        } else {
            Ok(Node::new_num(token_queue.except_num()?))
        }
    }

    pub fn assign_lvar_offset(&mut self) {
        let offset = self.locals_dequeue.len() * 8;
        self.stack_size = Self::align_to(offset, 16);
        for (i, name) in self.locals_dequeue.iter().enumerate() {
            let v = self.locals.get_mut(name).expect("local variable get error");
            *v = (i+1) * 8;
        }
    }

    fn align_to(n: usize, align: usize) -> usize {
        (n + align - 1) / align * align
    }
}
