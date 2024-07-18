use crate::{Node, NodeKind, Parser};

pub struct CodeGenerator {
    depth: usize,
    parser: Parser,
}

impl CodeGenerator {
    pub fn new(parser: Parser) -> CodeGenerator {
        Self { depth: 0, parser }
    }
    fn push(&mut self) {
        println!("  push %rax");
        self.depth += 1;
    }

    fn pop(&mut self, register: &str) {
        println!("  pop %{}", register);
        self.depth -= 1;
    }

    fn gen_addr(&self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        };
        if let NodeKind::Var { name } = &node.kind {
            let offset = self.parser.locals.get(name).expect("name not found");
            println!("  lea {}(%rbp), %rax", -(*offset as i32))
        }
    }

    pub fn generate(&mut self, nodes: Vec<Node>) {
        self.parser.assign_lvar_offset();
        println!(".global main");
        println!("main:");
        // prologur
        println!(" push %rbp");
        println!(" mov %rsp, %rbp");
        println!(" sub ${}, %rsp", self.parser.stack_size);

        for node in nodes {
            self.gen_stmt(Some(&node));
            assert!(self.depth == 0);
        }
        println!(".L.return:");
        println!(" mov %rbp, %rsp");
        println!(" pop %rbp");
        println!(" ret");
    }
    // generate code for a given node
    pub fn gen_expr(&mut self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        };
        match node.kind {
            NodeKind::Num { val } => {
                println!("  mov ${}, %rax", val);
                return;
            }
            NodeKind::Neg => {
                self.gen_expr(node.lhs.as_deref());
                println!("  neg %rax");
                return;
            }
            NodeKind::Var { .. } => {
                self.gen_addr(Some(node));
                println!("  mov (%rax), %rax");
                return;
            }
            NodeKind::Assign => {
                self.gen_addr(node.lhs.as_deref());
                self.push();
                self.gen_expr(node.rhs.as_deref());
                self.pop("rdi");
                println!(" mov %rax, (%rdi)");
                return;
            }
            _ => {}
        }
        self.gen_expr(node.rhs.as_deref());
        self.push();
        self.gen_expr(node.lhs.as_deref());
        self.pop("rdi");
        match node.kind {
            NodeKind::Add => {
                println!("  add %rdi, %rax");
            }
            NodeKind::Sub => {
                println!("  sub %rdi, %rax");
            }
            NodeKind::Mul => {
                println!("  imul %rdi, %rax");
            }
            NodeKind::Div => {
                println!("  cqo");
                println!("  idiv %rdi");
            }
            NodeKind::Eq | NodeKind::Ne | NodeKind::Lt | NodeKind::Le => {
                println!("  cmp %rdi, %rax");
                if node.kind == NodeKind::Eq {
                    println!("  sete %al");
                } else if node.kind == NodeKind::Ne {
                    println!("  setne %al");
                } else if node.kind == NodeKind::Lt {
                    println!("  setl %al");
                } else if node.kind == NodeKind::Le {
                    println!("  setle %al");
                }
                println!("  movzb %al, %rax");
                return;
            }
            _ => {
                panic!("invalid expression")
            }
        }
    }
    fn gen_stmt(&mut self, node: Option<&Node>) {
        let node = node.expect("node is not None");
        match node.kind {
            NodeKind::Return => {
                self.gen_expr(node.lhs.as_deref());
                println!("  jmp .L.return");
            }
            NodeKind::ExprStmt => {
                self.gen_expr(node.lhs.as_deref());
            }
            _ => {
                panic!("invalid statement")
            }
        }
    }
}
