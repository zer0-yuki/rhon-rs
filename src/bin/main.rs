use rhon_rs::{
    common::arena::Arena,
    frontend::{lexer::Lexer, parser::Parser},
};

fn main() {
    let source = r#"x = "hi"; id x = x;"#;
    let lexer = Lexer::new(source);
    let mut arena = Arena::new();

    let mut parser = Parser::new(lexer, &mut arena);

    let scs = parser.parse();

    for sc in &scs {
        println!("{} {:?} =", sc.name, sc.args);
        println!("  -> {:?}", sc.body.kind);
    }

    for diag in parser.diagnostics() {
        eprintln!("parse error: {:?}", diag);
    }
    let vec = vec![1];
    drop(vec);
}

// enum Recursive<'ptr> {
//     None,
//     Next(ArenaPtr<'ptr, Recursive<'ptr>>),
// }

// struct Test<'arena, 'ptr>(&'arena Arena<Recursive<'ptr>>);

// impl<'a, 'ptr> Test<'a, 'ptr> {
//     fn new(arena: &'a Arena<Recursive<'ptr>>) -> Self {
//         Self(arena)
//     }

//     fn alloc(&self, value: Recursive<'ptr>) -> ArenaPtr<'_, Recursive<'ptr>> {
//         self.0.alloc(value).into()
//     }

//     fn do_sth(&self) {
//         let n = self.alloc(Recursive::None);
//         let next = Recursive::Next(n);
//         self.alloc(next);
//     }
// }

// fn test() {
//     let arena = Arena::new();
//     let test = Test::new(&arena);
// }
