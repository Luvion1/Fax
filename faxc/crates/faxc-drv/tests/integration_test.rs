use faxc_drv::{Config, Session, EmitType};
use std::path::PathBuf;

#[test]
fn test_compile_if_else_logic() {
    let source_code = r#"
        fn main() {
            if true {
                let x = 1;
            }
        }
    "#;

    let mut config = Config::default();
    config.emit = EmitType::Lir;

    let mut session = Session::new(config).expect("Failed to create session");
    session.sources.add(PathBuf::from("logic.fax"), source_code.to_string());

    let source = &session.sources.iter().next().unwrap().1.content;
    let mut lexer = faxc_lex::Lexer::new(source, &mut session.diagnostics);
    let tokens: Vec<_> = std::iter::from_fn(|| Some(lexer.next_token()))
        .take_while(|t| *t != faxc_lex::Token::Eof)
        .collect();
    let mut parser = faxc_par::Parser::new(tokens, &mut session.diagnostics);
    let ast = parser.parse();

    println!("Generated AST: {:#?}", ast);

    let result = session.compile().expect("Compilation failed up to LIR");

    let mut found_cmp = false;
    let mut found_jcc = false;
    let mut found_jmp = false;

    for (_, lir_fn) in &result.lirs {
        println!("LIR Instructions for {}:", lir_fn.name.as_str());
        for instr in &lir_fn.instructions {
            println!("  {:?}", instr);
            match instr {
                faxc_lir::Instruction::Cmp { .. } => found_cmp = true,
                faxc_lir::Instruction::Jcc { .. } => found_jcc = true,
                faxc_lir::Instruction::Jmp { .. } => found_jmp = true,
                _ => {}
            }
        }
    }

    assert!(found_cmp, "LIR should contain Cmp instruction");
    assert!(found_jcc, "LIR should contain Jcc instruction");
    assert!(found_jmp, "LIR should contain Jmp instruction");
}
