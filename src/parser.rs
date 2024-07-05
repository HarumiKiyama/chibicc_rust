use crate::{MyError, TokenQueue};

#[derive(PartialEq, Debug)]
pub enum NodeKind {
    ADD, // +
    SUB, // -
    MUL, // *
    DIV, // /
    EQ,  // ==
    NE,  // !=
    LT,  // <
    LE,  // <=
    NUM, // Integer
}

type ParseResult = Result<Node, MyError>;

pub struct Node {
    pub kind: NodeKind,         // Node kind
    pub lhs: Option<Box<Node>>, // Left-hand side
    pub rhs: Option<Box<Node>>, // Right-hand side
    pub val: Option<i32>,       // Used if kind == ND_NUM
}

impl Node {
    fn new_num(val: i32) -> Self {
        Node {
            kind: NodeKind::NUM,
            lhs: None,
            rhs: None,
            val: Some(val),
        }
    }

    fn new_binary(kind: NodeKind, lhs: Node, rhs: Node) -> Self {
        Node {
            kind,
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
            val: None,
        }
    }

    pub fn expr(token_queue: &mut TokenQueue) -> ParseResult {
        Self::equality(token_queue)
    }

    // equality = relational ("==" relational | "!=" relational)*
    fn equality(token_queue: &mut TokenQueue) -> ParseResult {
        let mut node = Self::relational(token_queue)?;
        loop {
            if token_queue.consume("==")? {
                node = Self::new_binary(NodeKind::EQ, node, Self::relational(token_queue)?);
            } else if token_queue.consume("!=")? {
                node = Self::new_binary(NodeKind::NE, node, Self::relational(token_queue)?);
            } else {
                return Ok(node);
            }
        }
    }

    // relational = add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(token_queue: &mut TokenQueue) -> ParseResult {
        let mut node = Self::add(token_queue)?;
        loop {
            if token_queue.consume("<")? {
                node = Self::new_binary(NodeKind::LT, node,
                     Self::add(token_queue)?);
            } else if token_queue.consume("<=")? {
                node = Self::new_binary(NodeKind::LE, node, 
                    Self::add(token_queue)?);
            } else if token_queue.consume(">")? {
                node = Self::new_binary(NodeKind::LT,
                     Self::add(token_queue)?, node);
            } else if token_queue.consume(">=")? {
                node = Self::new_binary(NodeKind::LE,
                     Self::add(token_queue)?, node);
            } else {
                return Ok(node);
            }
        }
    }

    // add = mul ("+" mul | "-" mul)*
    fn add(token_queue: &mut TokenQueue) -> ParseResult {
        let mut node = Self::mul(token_queue)?;
        loop {
            if token_queue.consume("+")? {
                node = Self::new_binary(NodeKind::ADD, node,
                     Self::mul(token_queue)?);
            } else if token_queue.consume("-")? {
                node = Self::new_binary(NodeKind::SUB, node, 
                    Self::mul(token_queue)?);
            } else {
                return Ok(node);
            }
        }
    }
    // mul = unary ("*" unary | "/" unary)*
    fn mul(token_queue: &mut TokenQueue) -> ParseResult {
        let mut node = Self::unary(token_queue)?;
        loop {
            if token_queue.consume("*")? {
                node = Self::new_binary(NodeKind::MUL, node,
                     Self::unary(token_queue)?);
            } else if token_queue.consume("/")? {
                node = Self::new_binary(NodeKind::DIV, node, 
                    Self::unary(token_queue)?);
            } else {
                return Ok(node);
            }
        }
    }

    // unary = ("+" | "-")? unary
    //       | primary
    fn unary(token_queue: &mut TokenQueue) -> ParseResult {
        if token_queue.consume("+")? {
            return Self::unary(token_queue);
        }
        if token_queue.consume("-")? {
            let node = Self::new_binary(NodeKind::SUB,
                 Self::new_num(0), Self::unary(token_queue)?);
            return Ok(node);
        }
        return Self::primary(token_queue);
    }

    // primary = "(" expr ")" | num
    fn primary(token_queue: &mut TokenQueue) -> ParseResult {
        if token_queue.consume("(")? {
            let node = Self::expr(token_queue)?;
            token_queue.expect(")")?;
            return Ok(node);
        }
        Ok(Self::new_num(token_queue.except_num()?))
    }
}
