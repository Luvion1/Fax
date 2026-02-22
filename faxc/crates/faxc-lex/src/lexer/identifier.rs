//! Identifier and keyword lexing.
//!
//! This module handles lexing of identifiers and keywords.

use crate::token::{keyword_from_ident, Token};
use crate::unicode::is_ascii_ident_continue;
use crate::Lexer;
use faxc_util::Symbol;

impl<'a> Lexer<'a> {
    /// Lexes an identifier or keyword.
    ///
    /// Identifiers start with a letter or underscore, followed by
    /// alphanumeric characters or underscores. After reading the identifier,
    /// checks if it matches a reserved keyword.
    ///
    /// # Returns
    ///
    /// Either a keyword token (e.g., `Token::Let`) or `Token::Ident(symbol)`
    pub fn lex_identifier(&mut self) -> Token {
        while is_ascii_ident_continue(self.cursor.current_char()) {
            self.cursor.advance();
        }

        let text = self.cursor.slice_from(self.token_start);

        keyword_from_ident(text).unwrap_or_else(|| Token::Ident(Symbol::intern(text)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;
    use faxc_util::Handler;

    fn lex_ident(source: &str) -> Token {
        let mut handler = Handler::new();
        let mut lexer = crate::Lexer::new(source, &mut handler);
        lexer.lex_identifier()
    }

    #[test]
    fn test_simple_identifier() {
        let token = lex_ident("foo");
        assert_eq!(token, Token::Ident(Symbol::intern("foo")));
    }

    #[test]
    fn test_identifier_with_underscore() {
        let token = lex_ident("foo_bar_123");
        assert_eq!(token, Token::Ident(Symbol::intern("foo_bar_123")));
    }

    #[test]
    fn test_keyword_let() {
        let token = lex_ident("let");
        assert_eq!(token, Token::Let);
    }

    #[test]
    fn test_keyword_fn() {
        let token = lex_ident("fn");
        assert_eq!(token, Token::Fn);
    }

    #[test]
    fn test_keyword_if() {
        let token = lex_ident("if");
        assert_eq!(token, Token::If);
    }

    #[test]
    fn test_keyword_else() {
        let token = lex_ident("else");
        assert_eq!(token, Token::Else);
    }

    #[test]
    fn test_keyword_while() {
        let token = lex_ident("while");
        assert_eq!(token, Token::While);
    }

    #[test]
    fn test_keyword_match() {
        let token = lex_ident("match");
        assert_eq!(token, Token::Match);
    }

    #[test]
    fn test_keyword_struct() {
        let token = lex_ident("struct");
        assert_eq!(token, Token::Struct);
    }

    #[test]
    fn test_keyword_enum() {
        let token = lex_ident("enum");
        assert_eq!(token, Token::Enum);
    }

    #[test]
    fn test_keyword_trait() {
        let token = lex_ident("trait");
        assert_eq!(token, Token::Trait);
    }

    #[test]
    fn test_keyword_impl() {
        let token = lex_ident("impl");
        assert_eq!(token, Token::Impl);
    }

    #[test]
    fn test_keyword_true() {
        let token = lex_ident("true");
        assert_eq!(token, Token::True);
    }

    #[test]
    fn test_keyword_false() {
        let token = lex_ident("false");
        assert_eq!(token, Token::False);
    }

    #[test]
    fn test_keyword_async() {
        let token = lex_ident("async");
        assert_eq!(token, Token::Async);
    }

    #[test]
    fn test_keyword_await() {
        let token = lex_ident("await");
        assert_eq!(token, Token::Await);
    }

    #[test]
    fn test_keyword_return() {
        let token = lex_ident("return");
        assert_eq!(token, Token::Return);
    }

    #[test]
    fn test_keyword_break() {
        let token = lex_ident("break");
        assert_eq!(token, Token::Break);
    }

    #[test]
    fn test_keyword_continue() {
        let token = lex_ident("continue");
        assert_eq!(token, Token::Continue);
    }

    #[test]
    fn test_keyword_for() {
        let token = lex_ident("for");
        assert_eq!(token, Token::For);
    }

    #[test]
    fn test_keyword_loop() {
        let token = lex_ident("loop");
        assert_eq!(token, Token::Loop);
    }

    #[test]
    fn test_keyword_mut() {
        let token = lex_ident("mut");
        assert_eq!(token, Token::Mut);
    }

    #[test]
    fn test_keyword_pub() {
        let token = lex_ident("pub");
        assert_eq!(token, Token::Pub);
    }

    #[test]
    fn test_keyword_const() {
        let token = lex_ident("const");
        assert_eq!(token, Token::Const);
    }

    #[test]
    fn test_keyword_static() {
        let token = lex_ident("static");
        assert_eq!(token, Token::Static);
    }

    #[test]
    fn test_keyword_unsafe() {
        let token = lex_ident("unsafe");
        assert_eq!(token, Token::Unsafe);
    }

    #[test]
    fn test_keyword_ref() {
        let token = lex_ident("ref");
        assert_eq!(token, Token::Ref);
    }

    #[test]
    fn test_keyword_self() {
        let token = lex_ident("self");
        assert_eq!(token, Token::Self_);
    }

    #[test]
    fn test_keyword_Self() {
        let token = lex_ident("Self");
        assert_eq!(token, Token::SelfUpper);
    }

    #[test]
    fn test_keyword_mod() {
        let token = lex_ident("mod");
        assert_eq!(token, Token::Mod);
    }

    #[test]
    fn test_keyword_use() {
        let token = lex_ident("use");
        assert_eq!(token, Token::Use);
    }

    #[test]
    fn test_keyword_as() {
        let token = lex_ident("as");
        assert_eq!(token, Token::As);
    }

    #[test]
    fn test_keyword_super() {
        let token = lex_ident("super");
        assert_eq!(token, Token::Super);
    }

    #[test]
    fn test_keyword_crate() {
        let token = lex_ident("crate");
        assert_eq!(token, Token::Crate);
    }

    #[test]
    fn test_keyword_type() {
        let token = lex_ident("type");
        assert_eq!(token, Token::Type);
    }

    #[test]
    fn test_keyword_where() {
        let token = lex_ident("where");
        assert_eq!(token, Token::Where);
    }

    #[test]
    fn test_keyword_dyn() {
        let token = lex_ident("dyn");
        assert_eq!(token, Token::Dyn);
    }

    #[test]
    fn test_keyword_macro_rules() {
        let token = lex_ident("macro_rules");
        assert_eq!(token, Token::MacroRules);
    }
}
