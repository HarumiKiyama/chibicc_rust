use std::collections::{HashMap, VecDeque};

use crate::{MyError, TokenQueue};

#[derive(PartialEq, Debug, Clone)]
pub enum Node {
    Add {
        lhs: Box<Node>,
        rhs: Box<Node>,
        r#type: Type,
    }, // +

    Sub {
        lhs: Box<Node>,
        rhs: Box<Node>,
        r#type: Type,
    }, // -
    Mul {
        lhs: Box<Node>,
        rhs: Box<Node>,
        r#type: Type,
    }, // *
    Div {
        lhs: Box<Node>,
        rhs: Box<Node>,
        r#type: Type,
    }, // /
    Neg {
        lhs: Box<Node>,
        r#type: Type,
    }, // unary -
    Eq {
        lhs: Box<Node>,
        rhs: Box<Node>,
        r#type: Type,
    }, // ==
    Ne {
        lhs: Box<Node>,
        rhs: Box<Node>,
        r#type: Type,
    }, // !=
    Lt {
        lhs: Box<Node>,
        rhs: Box<Node>,
        r#type: Type,
    }, // <
    Le {
        lhs: Box<Node>,
        rhs: Box<Node>,
        r#type: Type,
    }, // <=
    Assign {
        lhs: Box<Node>,
        rhs: Box<Node>,
        r#type: Type,
    }, // =
    Addr {
        lhs: Box<Node>,
        r#type: Type,
    }, // unary &
    Deref {
        lhs: Box<Node>,
        r#type: Type,
    }, // unary *
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
        r#type: Type,
    }, // Local variable
    Num {
        val: i32,
        r#type: Type,
    }, // Integer
}

impl Node {
    pub fn is_add(&self) -> bool {
        matches!(self, Self::Add { .. })
    }

    pub fn get_type(&self) -> Option<Type> {
        match self {
            Node::Var { r#type, .. }
            | Node::Add { r#type, .. }
            | Node::Sub { r#type, .. }
            | Node::Mul { r#type, .. }
            | Node::Div { r#type, .. }
            | Node::Neg { r#type, .. }
            | Node::Assign { r#type, .. }
            | Node::Eq { r#type, .. }
            | Node::Ne { r#type, .. }
            | Node::Lt { r#type, .. }
            | Node::Le { r#type, .. }
            | Node::Num { r#type, .. }
            | Node::Addr { r#type, .. }
            | Node::Deref { r#type, .. } => Some(r#type.clone()),
            _ => None,
        }
    }

    pub fn is_ptr_node(&self) -> bool {
        match self {
            Node::Var { r#type, .. }
            | Node::Add { r#type, .. }
            | Node::Sub { r#type, .. }
            | Node::Mul { r#type, .. }
            | Node::Div { r#type, .. }
            | Node::Neg { r#type, .. }
            | Node::Assign { r#type, .. }
            | Node::Eq { r#type, .. }
            | Node::Ne { r#type, .. }
            | Node::Lt { r#type, .. }
            | Node::Le { r#type, .. }
            | Node::Num { r#type, .. }
            | Node::Addr { r#type, .. }
            | Node::Deref { r#type, .. } => match r#type {
                Type::I32 => false,
                Type::Ptr { .. } => true,
            },
            _ => false,
        }
    }
    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var { .. })
    }

    pub fn is_num(&self) -> bool {
        matches!(self, Self::Num { .. })
    }

    pub fn assign_type(&mut self) {}
}

#[derive(PartialEq, Debug, Clone)]
pub enum Type {
    I32,
    Ptr { base: Box<Type> },
}

type ParseResult = Result<Node, MyError>;

#[derive(Clone)]
pub struct VarTableItem {
    pub offset: usize,
    pub r#type: Type,
}

type VarTable = HashMap<String, VarTableItem>; // variable name offset hashtable

pub struct Parser {
    pub locals: VarTable,
    pub locals_dequeue: VecDeque<String>,
    pub stack_size: usize,
    pub nodes: Vec<Node>,
    pub token_queue: TokenQueue,
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
    fn find_var(&self, name: &String) -> Option<VarTableItem> {
        self.locals.get(name).cloned()
    }

    fn push_var(&mut self, name: String, r#type: Type) -> usize {
        match self.locals.get(&name) {
            None => {
                self.locals_dequeue.push_front(name.clone());
                let item = VarTableItem {
                    offset: self.locals_dequeue.len() * 8,
                    r#type,
                };
                self.locals.insert(name, item);
            }
            _ => {}
        };
        self.locals_dequeue.len() * 8
    }

    // declspec = "int"
    fn declspec(&mut self) -> Result<Type, MyError> {
        self.token_queue.expect_reserve("int")?;
        Ok(Type::I32)
    }

    // declarator = "*"* ident
    fn declarator(&mut self, base_type: Type) -> ParseResult {
        let mut num = 0;
        while self.token_queue.consume_reserve("*")? {
            num += 1;
        }
        if let Some(name) = self.token_queue.consume_ident()? {
            let r#type = if num > 0 {
                let mut t = Type::Ptr {
                    base: Box::new(base_type),
                };
                for _ in 0..num - 1 {
                    t = Type::Ptr { base: Box::new(t) }
                }
                t
            } else {
                Type::I32
            };

            self.push_var(name.clone(), r#type.clone());
            Ok(Node::Var { name, r#type })
        } else {
            Err(MyError {
                info: "expect a variable name".to_string(),
            })
        }
    }

    //declaration = declspec (declarator ("=" expr)? ("," declarator ("=" expr)?)*)? ";"
    fn declaration(&mut self) -> ParseResult {
        let base_type = self.declspec()?;
        let mut head = true;
        let mut nodes = Vec::new();
        while !self.token_queue.consume_reserve(";")? {
            if !head {
                self.token_queue.expect_reserve(",")?;
            }
            if head {
                head = false;
            }

            let declarator = self.declarator(base_type.clone())?;
            if !self.token_queue.consume_reserve("=")? {
                // TODO: support initialization variable use empty value
                continue;
            }
            let assign_node = Node::Assign {
                lhs: Box::new(declarator),
                rhs: Box::new(self.expr()?),
                r#type: base_type.clone(),
            };
            let node = Node::ExprStmt {
                expr: Box::new(assign_node),
            };
            nodes.push(node);
        }
        return Ok(Node::Block { nodes });
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

    // compound-stmt = (declaration | stmt)* "}"
    fn compound_stmt(&mut self) -> ParseResult {
        let mut nodes = Vec::new();
        while !self.token_queue.consume_reserve("}")? {
            let node = if self.token_queue.is_reserve("int") {
                self.declaration()?
            } else {
                self.stmt()?
            };
            nodes.push(node);
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
                r#type: node.get_type().expect("should have a type"),
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
                    r#type: Type::I32,
                };
            } else if self.token_queue.consume_reserve("!=")? {
                node = Node::Ne {
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()?),
                    r#type: Type::I32,
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
                    r#type: Type::I32,
                };
            } else if self.token_queue.consume_reserve("<=")? {
                node = Node::Le {
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()?),
                    r#type: Type::I32,
                };
            } else if self.token_queue.consume_reserve(">")? {
                node = Node::Lt {
                    lhs: Box::new(self.add()?),
                    rhs: Box::new(node),
                    r#type: Type::I32,
                };
            } else if self.token_queue.consume_reserve(">=")? {
                node = Node::Le {
                    lhs: Box::new(self.add()?),
                    rhs: Box::new(node),
                    r#type: Type::I32,
                };
            } else {
                return Ok(node);
            }
        }
    }

    // Canonicalize `num + ptr` to `ptr + num`.
    fn new_add(&self, mut node: Node) -> Result<Node, MyError> {
        let Node::Add {
            ref mut lhs,
            ref mut rhs,
            ..
        } = node
        else {
            return Err(MyError {
                info: format!(
                    "not a add node, current node: {:?}, current token: {:?}",
                    node, self.token_queue
                ),
            });
        };
        if lhs.is_ptr_node() && rhs.is_ptr_node() {
            return Err(MyError {
                info: format!(
                    "two pointer add error, current node: {:?}, current token: {:?}",
                    node, self.token_queue
                ),
            });
        }
        if (lhs.is_num() && rhs.is_var()) || rhs.is_ptr_node() {
            std::mem::swap(lhs, rhs);
            return Ok(node);
        }

        // ptr + num
        if lhs.is_ptr_node() {
            let new_rhs = Box::new(Node::Mul {
                lhs: Box::new(*rhs.clone()),
                rhs: Box::new(Node::Num {
                    val: 8,
                    r#type: Type::I32,
                }),
                r#type: Type::I32,
            });
            let _ = std::mem::replace(rhs, new_rhs);
            return Ok(node);
        }

        return Ok(node);
    }

    // for support pointer - pointer and pointer - number
    fn new_sub(&self, node: Node) -> Result<Node, MyError> {
        let Node::Sub {
            ref lhs, ref rhs, ..
        } = node
        else {
            return Err(MyError {
                info: format!(
                    "not a sub node, current node: {:?}, current token: {:?}",
                    node, self.token_queue
                ),
            });
        };
        if rhs.is_ptr_node() && !lhs.is_ptr_node() {
            return Err(MyError {
                info: format!(
                    "minus pointer error, current node: {:?}, current token : {:?}",
                    node, self.token_queue
                ),
            });
        }

        if lhs.is_ptr_node() && rhs.is_ptr_node() {
            let new_node = Node::Div {
                lhs: Box::new(node),
                rhs: Box::new(Node::Num {
                    val: 8,
                    r#type: Type::I32,
                }),
                r#type: Type::I32,
            };
            return Ok(new_node);
        }
        if lhs.is_ptr_node() {
            let new_rhs = Box::new(Node::Mul {
                lhs: Box::new(*rhs.clone()),
                rhs: Box::new(Node::Num {
                    val: 8,
                    r#type: Type::I32,
                }),
                r#type: Type::I32,
            });
            return Ok(Node::Sub {
                lhs: Box::new(*lhs.clone()),
                rhs: new_rhs,
                r#type: lhs.get_type().expect("should have a type"),
            });
        }
        return Ok(node);
    }

    // add = mul ("+" mul | "-" mul)*
    fn add(&mut self) -> ParseResult {
        let mut node = self.mul()?;
        loop {
            if self.token_queue.consume_reserve("+")? {
                node = Node::Add {
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()?),
                    r#type: node.get_type().expect("should have a type"),
                };
                node = self.new_add(node)?;
            } else if self.token_queue.consume_reserve("-")? {
                node = Node::Sub {
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()?),
                    r#type: node.get_type().expect("should have a type"),
                };
                node = self.new_sub(node)?;
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
                    r#type: node.get_type().expect("should have a type"),
                };
            } else if self.token_queue.consume_reserve("/")? {
                node = Node::Div {
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()?),
                    r#type: node.get_type().expect("should have a type"),
                };
            } else {
                return Ok(node);
            }
        }
    }

    // unary = ("+" | "-" | "*" | "&") unary
    //       | primary
    fn unary(&mut self) -> ParseResult {
        if self.token_queue.consume_reserve("+")? {
            return self.unary();
        }
        if self.token_queue.consume_reserve("-")? {
            let lhs = self.unary()?;
            let node = Node::Neg {
                lhs: Box::new(lhs),
                r#type: lhs.get_type().expect("should have a type"),
            };
            return Ok(node);
        }
        if self.token_queue.consume_reserve("*")? {
            let lhs = self.unary()?;
            let node = Node::Deref {
                lhs: Box::new(self.unary()?),
                r#type: todo!("complete this")
            };
            return Ok(node);
        }
        if self.token_queue.consume_reserve("&")? {
            let node = Node::Addr {
                lhs: Box::new(self.unary()?),
                r#type: todo!("complete this")
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
            let item = self.find_var(&name).ok_or(MyError {
                info: format!("undefined variable: {}", name),
            })?;
            Ok(Node::Var {
                name,
                r#type: item.r#type,
            })
        } else {
            Ok(Node::Num {
                val: self.token_queue.expect_num()?,
                r#type: Type::I32
            })
        }
    }

    pub fn assign_lvar_offset(&mut self) {
        let offset = self.locals_dequeue.len() * 8;
        self.stack_size = Self::align_to(offset, 16);
        for (i, name) in self.locals_dequeue.iter().enumerate() {
            let v = self.locals.get_mut(name).expect("local variable get error");
            v.offset = (i + 1) * 8;
        }
    }

    fn align_to(n: usize, align: usize) -> usize {
        (n + align - 1) / align * align
    }
}
