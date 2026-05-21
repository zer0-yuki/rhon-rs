use rhon_rs::frontend::{lexer::Lexer, parser::Parser};

fn main() {
    let source = r#"
    x = 1 + 2 + 3;
    y = 1 + 2 * 3;
    z = (1 + 2) * 3;
    cube x = x * x * x;
    pipe x f = f x y;
    "#;
    let mut lexer = Lexer::new(source);

    let mut parser = Parser::new(&mut lexer);

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
