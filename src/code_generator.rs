use crate::{Node, NodeKind};

pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> CodeGenerator {
        Self{}
    }

    pub fn generate(&self, nodes: Vec<Node>) {
        println!(".intel_syntax noprefix");
        println!(".global main");
        println!("main:");
        for node in nodes{
            self.generate_single_node(Some(&node));
            println!(" pop rax")
        }
        println!(" ret")
    }

    fn generate_single_node(&self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        };
        if node.kind == NodeKind::NUM {
            println!("  push {}", node.val.unwrap());
            return;
        }
        self.generate_single_node(node.lhs.as_deref());
        self.generate_single_node(node.rhs.as_deref());
        println!(" pop rdi");
        println!(" pop rax");
        match node.kind {
            NodeKind::NUM => {}
            NodeKind::ADD => {
                println!(" add rax, rdi");
            }
            NodeKind::SUB => {
                println!(" sub rax, rdi");
            }
            NodeKind::MUL => {
                println!(" imul rax, rdi");
            }
            NodeKind::DIV => {
                println!(" cqo");
                println!(" idiv rdi");
            }
            NodeKind::EQ => {
                println!(" cmp rax, rdi");
                println!(" sete al");
                println!(" movzb rax, al");
            }
            NodeKind::NE => {
                println!(" cmp rax, rdi");
                println!(" setne al");
                println!(" movzb rax, al");
            }
            NodeKind::LT => {
                println!(" cmp rax, rdi");
                println!(" setl al");
                println!(" movzb rax, al");
            }
            NodeKind::LE => {
                println!(" cmp rax, rdi");
                println!(" setle al");
                println!(" movzb rax, al");
            }
        }
        println!(" push rax");
    }
}
