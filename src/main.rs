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
    // for token in tokens {
    //     println!("{}", token);
    // }

    println!(".intel_syntax noprefix");
    println!(".global _main");
    println!("_main:");
    let TokenType::Number(n) = tokens[0].ty;
    println!("  push {}", n);
    println!("  pop rax");
    println!("  ret");
    Ok(())
}

enum TokenType {
    Number(i32),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::Number(n) => write!(f, "Number {}", n),
        }
    }
}

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
        panic!("oops!");
    }
    return ret;
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
