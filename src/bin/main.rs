use rhon_rs::frontend::lexer::Lexer;

fn main() {
    let source = r#"x = "hi";"#;
    let lexer = Lexer::new(source);

    println!("Tokens:");
    for token in lexer {
        println!("  {:?}", token.kind);
    }
}
