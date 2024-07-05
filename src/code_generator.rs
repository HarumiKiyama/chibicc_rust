use crate::{Node, NodeKind};

pub struct CodeGenerator;




impl CodeGenerator {
    pub fn new() -> CodeGenerator {
        Self
    }

    pub fn generate(&self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        }; 

        if node.kind == NodeKind::NUM {
            println!("  push {}", node.val.unwrap());
            return;
        }
        self.generate(node.lhs.as_deref());
        self.generate(node.rhs.as_deref());
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

