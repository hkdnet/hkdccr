fn main() {
    println!(".intel_syntax noprefix");
    println!(".global _main");
    println!("_main:");
    println!("  push 0");
    println!("  pop rax");
    println!("  ret");
}
