#![feature(box_syntax)]
#![feature(box_patterns)]

use std::fmt;
use std::io;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("arg count should be 2");
        std::process::exit(1);
    }
    let input = &args[1];
    if let Err(_) = run(input) {
        println!("ng");
    }
}

fn run(input: &str) -> io::Result<()> {
    let tokens = tokenize(input);

    // for debug
    // for token in tokens.clone() {
    //     println!("{}", token);
    // }

    let node_result = parse(tokens);
    match node_result {
        Ok(node) => {
            println!(".intel_syntax noprefix");
            println!(".global _main");
            println!("_main:");
            generate(node);
            println!("  pop rax");
            println!("  ret");
            Ok(())
        }
        Err(err) => {
            eprintln!("err: {}", err);
            std::process::exit(1);
        }
    }
}

fn generate(node: Node) {
    match node.ty {
        NodeType::Number(n) => println!("  push {}", n),
        NodeType::Add(lhs, rhs) => {
            generate(*lhs);
            generate(*rhs);
            println!("  pop rdi");
            println!("  pop rax");
            println!("  add rax, rdi");
            println!("  push rax");
        }
        NodeType::Sub(lhs, rhs) => {
            generate(*lhs);
            generate(*rhs);
            println!("  pop rdi");
            println!("  pop rax");
            println!("  sub rax, rdi");
            println!("  push rax");
        }
        NodeType::Multi(lhs, rhs) => {
            generate(*lhs);
            generate(*rhs);
            println!("  pop rdi");
            println!("  pop rax");
            println!("  mul rdi");
            println!("  push rax");
        }
        NodeType::Div(lhs, rhs) => {
            generate(*lhs);
            generate(*rhs);
            println!("  pop rdi");
            println!("  pop rax");
            println!("  mov rdx, 0");
            println!("  div rdi");
            println!("  push rax");
        }
    }
}

fn parse(tokens: Vec<Token>) -> Result<Node, &'static str> {
    let (node, tokens) = parse_expr(tokens);
    if node.is_err() {
        return node;
    }
    if tokens.len() != 0 {
        return Err("tokens are not fully consumed");
    }
    return node;
}

// expr: mul | mul "+" expr | mul "-" expr | "(" expr ")"
fn parse_expr(mut tokens: Vec<Token>) -> (Result<Node, &'static str>, Vec<Token>) {
    if let Some(token) = tokens.first() {
        if token.ty == TokenType::LParen {
            tokens.remove(0); // skip "("
            let (res, mut expr_tokens) = parse_expr(tokens);
            if res.is_err() {
                return (res, expr_tokens);
            }
            if let Some(token) = expr_tokens.first() {
                if token.ty == TokenType::RParen {
                    expr_tokens.remove(0); // skip ")"
                    return (res, expr_tokens);
                }
            }
            return (Err("mismatch paren"), expr_tokens);
        }
    }
    let (lhs_opt, mut plus_tokens) = parse_mul(tokens);
    if let Ok(lhs) = lhs_opt {
        if let Some(token) = plus_tokens.first() {
            if token.ty != TokenType::Plus && token.ty != TokenType::Minus {
                return (Err("expect plus/minus but not found"), plus_tokens);
            }
            let is_plus = token.ty == TokenType::Plus;
            plus_tokens.remove(0); // skip "+" / "-"
            let (rhs_opt, after_tokens) = parse_expr(plus_tokens);
            if let Ok(rhs) = rhs_opt {
                let p = if is_plus {
                    Ok(Node {
                        ty: NodeType::Add(Box::new(lhs), Box::new(rhs)),
                    })
                } else {
                    Ok(Node {
                        ty: NodeType::Sub(Box::new(lhs), Box::new(rhs)),
                    })
                };
                return (p, after_tokens);
            } else {
                return (Err("after \"+\", it should be number"), after_tokens);
            }
        } else {
            return (Ok(lhs), plus_tokens);
        }
    } else {
        return (Err("lhs is not a expr"), plus_tokens);
    }
}

#[test]
fn parse_expr_test() {
    let tokens = vec![Token {
        ty: TokenType::Number(1),
        text: "1",
    }];

    let (res, next_tokens) = parse_expr(tokens);
    assert_eq!(0, next_tokens.len());
    assert!(res.is_ok());

    let node = res.unwrap();
    match node.ty {
        NodeType::Number(num) => {
            assert_eq!(1, num);
        }
        _ => assert!(false),
    }
    let tokens = vec![
        Token {
            ty: TokenType::Number(1),
            text: "1",
        },
        Token {
            ty: TokenType::Plus,
            text: "+",
        },
        Token {
            ty: TokenType::Number(2),
            text: "2",
        },
    ];

    let (res, next_tokens) = parse_expr(tokens);
    assert_eq!(0, next_tokens.len());
    assert!(res.is_ok());

    let node = res.unwrap();
    match node.ty {
        NodeType::Add(box lhs, box rhs) => {
            assert!(lhs.ty == NodeType::Number(1));
            assert!(rhs.ty == NodeType::Number(2));
        }
        _ => assert!(false),
    }
}

// mul: number | number "*" mul | number "/" mul
fn parse_mul(tokens: Vec<Token>) -> (Result<Node, &'static str>, Vec<Token>) {
    let (res, mut num_tokens) = parse_number(tokens);
    if res.is_err() {
        return (Err("no number"), num_tokens);
    }
    if let Some(token) = num_tokens.first() {
        match token.ty {
            TokenType::Asterisk | TokenType::Slash => {
                let lhs = res.unwrap();
                let is_mul = token.ty == TokenType::Asterisk;
                num_tokens.remove(0); // skip "*" or "/"
                let (rhs_res, mul_tokens) = parse_mul(num_tokens);
                if let Ok(rhs) = rhs_res {
                    let node = if is_mul {
                        Node {
                            ty: NodeType::Multi(box lhs, box rhs),
                        }
                    } else {
                        Node {
                            ty: NodeType::Div(box lhs, box rhs),
                        }
                    };
                    return (Ok(node), mul_tokens);
                } else {
                    return (Err("there should be a mul after *"), mul_tokens);
                }
            }
            _ => (),
        }
    }
    return (res, num_tokens);
}

fn parse_number(mut tokens: Vec<Token>) -> (Result<Node, &'static str>, Vec<Token>) {
    if let Some(token) = tokens.first() {
        if let TokenType::Number(n) = token.ty {
            tokens.remove(0);
            return (
                Ok(Node {
                    ty: NodeType::Number(n),
                }),
                tokens,
            );
        }
    }
    return (Err("not a number"), tokens);
}

#[test]
fn paser_number_test() {
    let tokens = vec![Token {
        ty: TokenType::Number(123),
        text: "123",
    }];
    let (res, after_token) = parse_number(tokens);
    assert!(res.is_ok());
    assert_eq!(0, after_token.len());
    let node = res.unwrap();
    match node.ty {
        NodeType::Number(num) => {
            assert_eq!(123, num);
        }
        _ => assert!(false),
    }

    let tokens = vec![Token {
        ty: TokenType::Plus,
        text: "+",
    }];
    let (res, after_token) = parse_number(tokens);
    assert_eq!(1, after_token.len());
    assert!(res.is_err());
}

enum NodeType {
    Number(i32),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Multi(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeType::Number(n) => write!(f, "Number {}", n),
            NodeType::Add(lhs_box, rhs_box) => write!(f, "Add {} + {}", lhs_box, rhs_box),
            NodeType::Sub(lhs_box, rhs_box) => write!(f, "Sub {} - {}", lhs_box, rhs_box),
            NodeType::Multi(lhs_box, rhs_box) => write!(f, "Multi {} * {}", lhs_box, rhs_box),
            NodeType::Div(lhs_box, rhs_box) => write!(f, "Div {} * {}", lhs_box, rhs_box),
        }
    }
}

impl PartialEq for NodeType {
    fn eq(&self, other: &NodeType) -> bool {
        match self {
            NodeType::Number(n) => match other {
                NodeType::Number(o) => n == o,
                _ => false,
            },
            NodeType::Add(box l, box r) => match other {
                NodeType::Add(box ol, box or) => l == ol && r == or,
                _ => false,
            },
            NodeType::Sub(box l, box r) => match other {
                NodeType::Sub(box ol, box or) => l == ol && r == or,
                _ => false,
            },
            NodeType::Multi(box l, box r) => match other {
                NodeType::Multi(box ol, box or) => l == ol && r == or,
                _ => false,
            },
            NodeType::Div(box l, box r) => match other {
                NodeType::Div(box ol, box or) => l == ol && r == or,
                _ => false,
            },
        }
    }
}

#[derive(PartialEq)]
struct Node {
    ty: NodeType,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(type = {})", self.ty)
    }
}

#[derive(PartialEq, Clone)]
enum TokenType {
    Number(i32),
    Plus,
    Minus,
    Asterisk,
    Slash,
    LParen,
    RParen,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::Number(n) => write!(f, "Number {}", n),
            TokenType::Plus => write!(f, "Plus"),
            TokenType::Minus => write!(f, "Minus"),
            TokenType::Asterisk => write!(f, "Asterisk"),
            TokenType::Slash => write!(f, "Slash"),
            TokenType::LParen => write!(f, "LParen"),
            TokenType::RParen => write!(f, "RParen"),
        }
    }
}

#[derive(Clone)]
struct Token<'a> {
    ty: TokenType,
    text: &'a str,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(type = {}, text = {})", self.ty, self.text)
    }
}

// #[derive(PartialEq)]
// enum TokenizerState {
//     Normal,
//     Number(usize),
// }

fn tokenize(input: &str) -> Vec<Token> {
    let mut ret = Vec::new();
    let mut idx = 0;
    let bytes = input.as_bytes();
    'token_loop: loop {
        if idx >= bytes.len() {
            break;
        }
        let non_digit_idx = seek_until_non_digits(input, idx);
        if idx != non_digit_idx {
            let text = &input[idx..non_digit_idx];
            let num = text.parse::<i32>().unwrap();
            ret.push(Token {
                ty: TokenType::Number(num),
                text: text,
            });
            idx = non_digit_idx;
            continue 'token_loop;
        }
        let c = char::from(bytes[idx]);
        if is_space(c) {
            idx += 1;
            continue 'token_loop;
        }
        if c == '+' {
            ret.push(Token {
                ty: TokenType::Plus,
                text: "+",
            });
            idx += 1;
            continue 'token_loop;
        }
        if c == '-' {
            ret.push(Token {
                ty: TokenType::Minus,
                text: "-",
            });
            idx += 1;
            continue 'token_loop;
        }
        if c == '*' {
            ret.push(Token {
                ty: TokenType::Asterisk,
                text: "*",
            });
            idx += 1;
            continue 'token_loop;
        }
        if c == '/' {
            ret.push(Token {
                ty: TokenType::Slash,
                text: "/",
            });
            idx += 1;
            continue 'token_loop;
        }
        if c == '(' {
            ret.push(Token {
                ty: TokenType::LParen,
                text: "(",
            });
            idx += 1;
            continue 'token_loop;
        }
        if c == ')' {
            ret.push(Token {
                ty: TokenType::RParen,
                text: ")",
            });
            idx += 1;
            continue 'token_loop;
        }
        panic!("oops!");
    }
    return ret;
}

fn is_space(c: char) -> bool {
    return c == ' ';
}

fn is_digits(c: char) -> bool {
    return '0' <= c && c <= '9';
}

fn seek_until_non_digits(input: &str, beg: usize) -> usize {
    let mut idx = beg;
    let l = input.len();
    loop {
        if idx >= l {
            break;
        }
        let b = input.as_bytes()[idx];
        if !is_digits(char::from(b)) {
            break;
        }
        idx += 1;
    }

    return idx;
}

#[test]
fn seek_until_non_digits_test() {
    let text = "123 ";
    assert_eq!(3, seek_until_non_digits(text, 0));
    let text = " 123";
    assert_eq!(0, seek_until_non_digits(text, 0));
    assert_eq!(4, seek_until_non_digits(text, 1));
}
