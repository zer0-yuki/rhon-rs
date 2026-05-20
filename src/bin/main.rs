use rhon_rs::{
    common::arena::Arena,
    frontend::{
        lexer::{Lexer, Token, TokenKind},
        parser::Parser,
    },
};

fn main() {
    let source = r#"
    x = 1 + 2 + 3;
    y = 1 + 2 * 3;
    z = (1 + 2) * 3;
    cube x = x * x * x;
    pipe x f = f x y;
    "#;
    let lexer = Lexer::new(source);
    let mut arena = Arena::new();

    let mut parser = Parser::new(lexer, &mut arena);

    let scs = parser.parse();

    for sc in &scs {
        println!("{} {:?} =", sc.name, sc.args);
        println!("  ->\n{:?}", sc.body.kind);
    }

    for diag in parser.diagnostics() {
        eprintln!("parse error: {:?}", diag);
    }
    let vec = vec![1];
    drop(vec);
}
