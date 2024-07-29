use crate::{Node, Parser};

pub struct CodeGenerator {
    depth: usize,
    parser: Parser,
    counter: usize,
}

impl CodeGenerator {
    pub fn new(parser: Parser) -> CodeGenerator {
        Self {
            depth: 0,
            counter: 0,
            parser,
        }
    }
    fn count(&mut self) -> usize {
        self.counter += 1;
        self.counter
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
        if let Node::Var { name, .. } = &node {
            let offset = self.parser.locals.get(name).expect("name not found");
            println!("  lea -{}(%rbp), %rax", *offset)
        }
    }

    pub fn generate(&mut self, nodes: Vec<Node>) {
        self.parser.assign_lvar_offset();
        println!("  .global main");
        println!("main:");
        // prologur
        println!("  push %rbp");
        println!("  mov %rsp, %rbp");
        println!("  sub ${}, %rsp", self.parser.stack_size);

        for node in nodes {
            self.gen_stmt(Some(&node));
            assert!(self.depth == 0);
        }
        println!(".L.return:");
        println!("  mov %rbp, %rsp");
        println!("  pop %rbp");
        println!("  ret");
    }
    // generate code for a given node
    pub fn gen_expr(&mut self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        };
        match node {
            Node::Num { val } => {
                println!("  mov ${}, %rax", val);
                return;
            }
            Node::Neg { lhs } => {
                self.gen_expr(Some(lhs.as_ref()));
                println!("  neg %rax");
                return;
            }
            Node::Var { .. } => {
                self.gen_addr(Some(node));
                println!("  mov (%rax), %rax");
                return;
            }
            Node::Assign { lhs, rhs } => {
                self.gen_addr(Some(lhs.as_ref()));
                self.push();
                self.gen_expr(Some(rhs.as_ref()));
                self.pop("rdi");
                println!(" mov %rax, (%rdi)");
                return;
            }
            _ => {}
        }
        match node {
            Node::Add { lhs, rhs }
            | Node::Sub { lhs, rhs }
            | Node::Mul { lhs, rhs }
            | Node::Div { lhs, rhs }
            | Node::Eq { lhs, rhs }
            | Node::Ne { lhs, rhs }
            | Node::Lt { lhs, rhs }
            | Node::Le { lhs, rhs } => {
                self.gen_expr(Some(rhs.as_ref()));
                self.push();
                self.gen_expr(Some(lhs.as_ref()));
                self.pop("rdi");
            }
            _ => {
                panic!("invalid expression")
            }
        }
        let print_eq = |eq_str: &str| {
            println!("  cmp %rdi, %rax");
            println!("{}", eq_str);
            println!("  movzb %al, %rax");
        };
        match node {
            Node::Add { .. } => {
                println!("  add %rdi, %rax");
            }
            Node::Sub { .. } => {
                println!("  sub %rdi, %rax");
            }
            Node::Mul { .. } => {
                println!("  imul %rdi, %rax");
            }
            Node::Div { .. } => {
                println!("  cqo");
                println!("  idiv %rdi");
            }
            Node::Eq { .. } => {
                print_eq("  sete %al");
            }
            Node::Ne { .. } => {
                print_eq("  setne %al");
            }
            Node::Lt { .. } => {
                print_eq("  setl %al");
            }
            Node::Le { .. } => {
                print_eq("  setle %al");
            }
            _ => {
                panic!("invalid expression")
            }
        }
    }
    fn gen_stmt(&mut self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        };
        match node {
            Node::Return { lhs } => {
                self.gen_expr(lhs.as_deref());
                println!("  jmp .L.return");
            }
            Node::ExprStmt { expr } => {
                self.gen_expr(Some(expr.as_ref()));
            }

            Node::If { cond, then, els } => {
                let c = self.count();
                self.gen_expr(Some(cond.as_ref()));
                println!("  cmp $0, %rax");
                println!("  je .L.else.{}", c);
                self.gen_stmt(then.as_deref());
                println!("  jmp .L.end.{}", c);
                println!(".L.else.{}:", c);
                self.gen_stmt(els.as_deref());
                println!(".L.end.{}:", c);
            }
            Node::For {
                init,
                cond,
                inc,
                then,
            } => {
                let c = self.count();
                self.gen_stmt(init.as_deref());
                println!(".L.begin.{}:", c);
                if cond.is_some() {
                    self.gen_expr(cond.as_deref());
                    println!("  cmp $0, %rax");
                    println!("  je .L.end.{}", c);
                }
                self.gen_stmt(then.as_deref());
                self.gen_expr(inc.as_deref());
                println!("  jmp .L.begin.{}", c);
                println!(".L.end.{}:", c);
            }
            Node::Block { nodes } => {
                for node in nodes {
                    self.gen_stmt(Some(node));
                }
            }

            _ => {
                panic!("invalid statement")
            }
        }
    }
}
