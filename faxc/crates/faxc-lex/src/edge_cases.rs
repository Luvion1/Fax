//! Edge case tests for faxc-lex

#[cfg(test)]
mod tests {
    use crate::{Lexer, Token};
    use faxc_util::Handler;

    fn lex_all(source: &str) -> Vec<Token> {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            if token == Token::Eof { break; }
            tokens.push(token);
        }
        tokens
    }

    // ==================== EDGE CASES ====================

    #[test]
    fn test_edge_empty_source() {
        assert!(lex_all("").is_empty());
    }

    #[test]
    fn test_edge_single_char_ident() {
        let t = lex_all("x");
        assert_eq!(t[0], Token::Ident(faxc_util::Symbol::intern("x")));
    }

    #[test]
    fn test_edge_long_identifier() {
        let name = "a".repeat(10000);
        let t = lex_all(&format!("let {} = 1;", name));
        assert!(t.contains(&Token::Ident(faxc_util::Symbol::intern(&name))));
    }

    #[test]
    fn test_edge_keywords_not_idents() {
        let t = lex_all("fn let if");
        assert_eq!(t[0], Token::Fn);
        assert_eq!(t[1], Token::Let);
    }

    #[test]
    fn test_edge_hex_bounds() {
        let t = lex_all("0x0 0xFF");
        assert_eq!(t[0], Token::Number(0));
        assert_eq!(t[1], Token::Number(255));
    }

    #[test]
    fn test_edge_binary() {
        let t = lex_all("0b0 0b1010");
        assert_eq!(t[1], Token::Number(10));
    }

    #[test]
    fn test_edge_octal() {
        let t = lex_all("0o0 0o77");
        assert_eq!(t[1], Token::Number(63));
    }

    #[test]
    fn test_edge_empty_string() {
        let t = lex_all("\"\"");
        if let Token::String(s) = &t[0] {
            assert_eq!(s.as_str(), "");
        } else { panic!(); }
    }

    #[test]
    fn test_edge_all_operators() {
        let t = lex_all("+ - * / % == != < > <= >= && || !");
        assert!(t.contains(&Token::Plus));
        assert!(t.contains(&Token::EqEq));
    }

    #[test]
    fn test_edge_all_delimiters() {
        let t = lex_all("( ) { } [ ] , ; : . -> =>");
        assert!(t.contains(&Token::LParen));
        assert!(t.contains(&Token::Arrow));
    }

    #[test]
    fn test_edge_nested_delimiters() {
        let t = lex_all("((()))");
        assert_eq!(t.iter().filter(|x| **x == Token::LParen).count(), 3);
    }

    #[test]
    fn test_edge_case_sensitivity() {
        let t = lex_all("Fn fn");
        assert_eq!(t[0], Token::Ident(faxc_util::Symbol::intern("Fn")));
        assert_eq!(t[1], Token::Fn);
    }

    #[test]
    fn test_edge_bools() {
        let t = lex_all("true false");
        assert_eq!(t[0], Token::True);
        assert_eq!(t[1], Token::False);
    }

    #[test]
    fn test_edge_self_variants() {
        let t = lex_all("self Self");
        assert_eq!(t[0], Token::Self_);
        assert_eq!(t[1], Token::SelfUpper);
    }

    #[test]
    fn test_edge_dollar() {
        assert!(lex_all("$").contains(&Token::Dollar));
    }

    #[test]
    fn test_edge_at() {
        assert!(lex_all("@").contains(&Token::At));
    }

    #[test]
    fn test_edge_underscore() {
        assert!(lex_all("_").contains(&Token::Underscore));
    }

    #[test]
    fn test_edge_tilde() {
        assert!(lex_all("~").contains(&Token::Tilde));
    }

    #[test]
    fn test_edge_scientific() {
        let t = lex_all("1e10 1.5e-3");
        assert!(t.iter().all(|x| matches!(x, Token::Float(_))));
    }

    #[test]
    fn test_edge_max_int() {
        let t = lex_all("18446744073709551615");
        assert!(matches!(t[0], Token::Number(_)));
    }

    #[test]
    fn test_edge_all_keywords() {
        let t = lex_all("fn let if else match while for loop break continue return struct enum");
        assert!(t.contains(&Token::Fn));
        assert!(t.contains(&Token::Struct));
        assert!(t.contains(&Token::Enum));
    }

    // ==================== ERROR CASES ====================

    #[test]
    fn test_err_invalid_hex() {
        let mut h = Handler::new();
        let t = Lexer::new("0x", &mut h).next_token();
        assert!(matches!(t, Token::Number(_) | Token::Invalid(_)));
    }

    #[test]
    fn test_err_invalid_binary() {
        let mut h = Handler::new();
        let t = Lexer::new("0b", &mut h).next_token();
        assert!(matches!(t, Token::Number(_) | Token::Invalid(_)));
    }

    #[test]
    fn test_err_unterminated_string() {
        let mut h = Handler::new();
        let _ = Lexer::new("\"unterminated", &mut h).next_token();
        assert!(h.has_errors());
    }

    #[test]
    fn test_err_empty_char() {
        let mut h = Handler::new();
        let t = Lexer::new("''", &mut h).next_token();
        assert!(matches!(t, Token::Char(_) | Token::Invalid(_)));
    }

    #[test]
    fn test_err_unterminated_char() {
        let mut h = Handler::new();
        let _ = Lexer::new("'x", &mut h).next_token();
        assert!(h.has_errors());
    }

    #[test]
    fn test_err_invalid_chars() {
        let mut h = Handler::new();
        let mut lex = Lexer::new("@#$%", &mut h);
        while lex.next_token() != Token::Eof {}
        assert!(h.has_errors());
    }

    #[test]
    fn test_err_mixed_valid_invalid() {
        let mut h = Handler::new();
        let mut lex = Lexer::new("let x = # 1;", &mut h);
        while lex.next_token() != Token::Eof {}
        assert!(h.has_errors());
    }

    #[test]
    fn test_edge_raw_string() {
        let t = lex_all(r#"r"hello"world""#);
        if let Token::RawString(s) = &t[0] {
            assert!(s.as_str().contains('"'));
        } else { panic!(); }
    }

    #[test]
    fn test_edge_consec_ops() {
        assert!(lex_all("+++").len() >= 2);
    }

    #[test]
    fn test_edge_whitespace_variations() {
        let t = lex_all("let\tx\n=\n1");
        assert!(t.contains(&Token::Let));
        assert!(t.contains(&Token::Number(1)));
    }

    #[test]
    fn test_edge_leading_zeros() {
        assert!(!lex_all("007").is_empty());
    }
}