//! Edge case tests for faxc-par

#[cfg(test)]
mod tests {
    use crate::{Ast, Item, Parser};
    use faxc_lex::{Lexer, Token};
    use faxc_util::Handler;

    fn parse_source(source: &str) -> (Ast, Handler) {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);
        let tokens: Vec<_> = std::iter::from_fn(|| Some(lexer.next_token()))
            .take_while(|t| *t != Token::Eof)
            .collect();
        let mut parser = Parser::new(tokens, &mut handler);
        let ast = parser.parse();
        (ast, handler)
    }

    // ==================== EDGE CASES ====================

    /// EDGE CASE: Empty source
    #[test]
    fn test_edge_empty_source() {
        let (ast, handler) = parse_source("");
        assert!(ast.is_empty());
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Whitespace only
    #[test]
    fn test_edge_whitespace_only() {
        let (ast, handler) = parse_source("   \n\t  \n  ");
        assert!(ast.is_empty());
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Single function
    #[test]
    fn test_edge_single_function() {
        let (ast, handler) = parse_source("fn main() { }");
        assert_eq!(ast.len(), 1);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Empty function body
    #[test]
    fn test_edge_empty_function_body() {
        let (ast, handler) = parse_source("fn empty() {}");
        assert_eq!(ast.len(), 1);
        if let Item::Fn(fn_item) = &ast[0] {
            assert!(fn_item.body.stmts.is_empty());
        } else {
            panic!("Expected function item");
        }
    }

    /// EDGE CASE: Function with no parameters
    #[test]
    fn test_edge_no_params() {
        let (ast, handler) = parse_source("fn no_params() { let x = 1; }");
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Function with single parameter
    #[test]
    fn test_edge_single_param() {
        let (ast, handler) = parse_source("fn one(x: i32) { }");
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Deeply nested blocks
    #[test]
    fn test_edge_nested_blocks() {
        let source = "fn main() { {{{{ let x = 1; }}}} }";
        let (ast, handler) = parse_source(source);
        assert_eq!(ast.len(), 1);
    }

    /// EDGE CASE: Multiple statements
    #[test]
    fn test_edge_multiple_stmts() {
        let source = "fn main() { let a = 1; let b = 2; let c = 3; }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: If without else
    #[test]
    fn test_edge_if_no_else() {
        let source = "fn main() { if true { let x = 1; } }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: If with else
    #[test]
    fn test_edge_if_else() {
        let source = "fn main() { if true { 1 } else { 2 } }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Nested if-else
    #[test]
    fn test_edge_nested_if_else() {
        let source = "fn main() { if true { if false { 1 } else { 2 } } else { 3 } }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: While loop
    #[test]
    fn test_edge_while_loop() {
        let source = "fn main() { while true { let x = 1; } }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: For loop
    #[test]
    #[ignore = "for loops not implemented"]
    fn test_edge_for_loop() {
        let source = "fn main() { for i in 0..10 { let x = i; } }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Loop with break
    #[test]
    fn test_edge_loop_break() {
        let source = "fn main() { loop { break; } }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Loop with continue
    #[test]
    fn test_edge_loop_continue() {
        let source = "fn main() { loop { continue; } }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Return statement
    #[test]
    fn test_edge_return() {
        let source = "fn main() { return 42; }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Return without value
    #[test]
    fn test_edge_return_unit() {
        let source = "fn main() { return; }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Binary expressions
    #[test]
    fn test_edge_binary_exprs() {
        let source = "fn main() { let x = 1 + 2 * 3 - 4 / 2; }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Comparison operators
    #[test]
    fn test_edge_comparisons() {
        let source = "fn main() { let x = 1 == 2; let y = 1 != 2; let z = 1 < 2; }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Logical operators
    #[test]
    fn test_edge_logical_ops() {
        let source = "fn main() { let x = true && false; let y = true || false; }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Unary operators
    #[test]
    fn test_edge_unary_ops() {
        let source = "fn main() { let x = -5; let y = !true; }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Struct definition
    #[test]
    fn test_edge_struct_def() {
        let source = "struct Point { x: i32, y: i32 }";
        let (ast, handler) = parse_source(source);
        assert_eq!(ast.len(), 1);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Empty struct
    #[test]
    fn test_edge_empty_struct() {
        let source = "struct Empty {}";
        let (ast, handler) = parse_source(source);
        assert_eq!(ast.len(), 1);
    }

    /// EDGE CASE: Enum definition
    #[test]
    fn test_edge_enum_def() {
        let source = "enum Color { Red, Green, Blue }";
        let (ast, handler) = parse_source(source);
        assert_eq!(ast.len(), 1);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Enum with variants
    #[test]
    fn test_edge_enum_variants() {
        let source = "enum Option { Some(i32), None }";
        let (ast, handler) = parse_source(source);
        assert_eq!(ast.len(), 1);
    }

    /// EDGE CASE: Match expression
    #[test]
    fn test_edge_match() {
        let source = "fn main() { match 1 { 0 => 1, _ => 2 } }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Match with multiple patterns
    #[test]
    #[ignore = "match or-patterns not implemented"]
    fn test_edge_match_patterns() {
        let source = "fn main() { match 1 { 0 | 1 => 1, _ => 2 } }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Function call
    #[test]
    fn test_edge_fn_call() {
        let source = "fn main() { foo(); bar(1, 2); }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Nested function calls
    #[test]
    fn test_edge_nested_fn_calls() {
        let source = "fn main() { foo(bar(baz(1))); }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: All literal types
    #[test]
    fn test_edge_all_literals() {
        let source =
            "fn main() { let a = 42; let b = 3.14; let c = \"hello\"; let d = true; let e = 'x'; }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    // ==================== ERROR CASES ====================

    /// ERROR CASE: Missing semicolon
    #[test]
    fn test_err_missing_semicolon() {
        let source = "fn main() { let x = 1 let y = 2 }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Missing closing brace
    #[test]
    fn test_err_missing_closing_brace() {
        let source = "fn main() { let x = 1;";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Missing opening brace
    #[test]
    fn test_err_missing_opening_brace() {
        let source = "fn main() let x = 1; }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Invalid token in function body
    #[test]
    fn test_err_invalid_token() {
        let source = "fn main() { @#$ }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Incomplete if statement
    #[test]
    fn test_err_incomplete_if() {
        let source = "fn main() { if true }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Incomplete while loop
    #[test]
    fn test_err_incomplete_while() {
        let source = "fn main() { while true }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Incomplete match
    #[test]
    fn test_err_incomplete_match() {
        let source = "fn main() { match x { }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Incomplete struct
    #[test]
    fn test_err_incomplete_struct() {
        let source = "struct Point { x: i32";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Incomplete enum
    #[test]
    fn test_err_incomplete_enum() {
        let source = "enum Color { Red";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Invalid function signature
    #[test]
    fn test_err_invalid_fn_sig() {
        let source = "fn main( { }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Missing function body
    #[test]
    fn test_err_missing_fn_body() {
        let source = "fn main()";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Invalid type annotation
    #[test]
    fn test_err_invalid_type() {
        let source = "fn main() { let x: @invalid = 1; }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Unbalanced parentheses
    #[test]
    fn test_err_unbalanced_parens() {
        let source = "fn main() { foo((1, 2); }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Unbalanced brackets
    #[test]
    fn test_err_unbalanced_brackets() {
        let source = "fn main() { let x = [1, 2; }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Unterminated string in code
    #[test]
    fn test_err_unterminated_string_in_code() {
        let source = "fn main() { let x = \"unterminated; }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Multiple errors
    #[test]
    fn test_err_multiple_errors() {
        let source = "fn main( { if true { let x = @ }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Empty match arm
    #[test]
    fn test_err_empty_match_arm() {
        let source = "fn main() { match 1 { 0 => } }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Invalid operator
    #[test]
    fn test_err_invalid_operator() {
        let source = "fn main() { let x = 1 @ 2; }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Missing condition in if
    #[test]
    fn test_err_missing_if_cond() {
        let source = "fn main() { if { 1 } }";
        let (_, handler) = parse_source(source);
        assert!(handler.has_errors());
    }

    /// ERROR CASE: Trailing comma in struct
    #[test]
    fn test_edge_trailing_comma_struct() {
        let source = "struct Point { x: i32, y: i32, }";
        let (ast, handler) = parse_source(source);
        // Should handle trailing comma gracefully
        assert_eq!(ast.len(), 1);
    }

    /// ERROR CASE: Trailing comma in enum
    #[test]
    fn test_edge_trailing_comma_enum() {
        let source = "enum Color { Red, Green, Blue, }";
        let (ast, handler) = parse_source(source);
        assert_eq!(ast.len(), 1);
    }

    /// EDGE CASE: Comments in code
    #[test]
    fn test_edge_comments() {
        let source = "fn main() { // comment\nlet x = 1; /* block */ }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }

    /// EDGE CASE: Complex expression
    #[test]
    fn test_edge_complex_expr() {
        let source = "fn main() { let x = (1 + 2) * (3 - 4) / (5 % 6); }";
        let (ast, handler) = parse_source(source);
        assert!(!handler.has_errors());
    }
}
