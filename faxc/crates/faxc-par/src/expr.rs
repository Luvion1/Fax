//! Expression parsing using Pratt Parsing (Top-Down Operator Precedence)
//!
//! This module provides the core Pratt parsing algorithm and comprehensive
//! tests for expression parsing in the Fax programming language.
//!
//! # Operator Precedence (lowest to highest)
//!
//! | Level | Operators | Associativity |
//! |-------|-----------|---------------|
//! | 1 | `||` | Left |
//! | 2 | `&&` | Left |
//! | 3 | `==`, `!=`, `<`, `<=`, `>`, `>=` | Left |
//! | 4 | `|` | Left |
//! | 5 | `^` | Left |
//! | 6 | `&` | Left |
//! | 7 | `<<`, `>>` | Left |
//! | 8 | `+`, `-` | Left |
//! | 9 | `*`, `/`, `%` | Left |
//!
//! # Example
//!
//! ```
//! // a + b * c parses as a + (b * c) because * has higher precedence
//! // a - b - c parses as (a - b) - c because - is left-associative
//! ```

use crate::{BinOp, Expr};
use faxc_lex::Token;
use faxc_util::Handler;

use crate::{Parser, TokenWithSpan};

/// Binding power levels for Pratt parsing
/// Higher numbers = tighter binding (higher precedence)
#[doc(hidden)]
pub mod bp {
    /// Minimum binding power (start of expression)
    pub const MIN: u8 = 0;

    /// Logical OR: ||
    pub const LOGICAL_OR: u8 = 2;

    /// Logical AND: &&
    pub const LOGICAL_AND: u8 = 4;

    /// Comparison: ==, !=, <, <=, >, >=
    pub const COMPARISON: u8 = 6;

    /// Bitwise OR: |
    pub const BITWISE_OR: u8 = 8;

    /// Bitwise XOR: ^
    pub const BITWISE_XOR: u8 = 10;

    /// Bitwise AND: &
    pub const BITWISE_AND: u8 = 12;

    /// Shift: <<, >>
    pub const SHIFT: u8 = 14;

    /// Additive: +, -
    pub const ADDITIVE: u8 = 16;

    /// Multiplicative: *, /, %
    pub const MULTIPLICATIVE: u8 = 18;

    /// Cast: as (tighter than multiplicative)
    pub const CAST: u8 = 22;

    /// Maximum binding power (for prefix operators)
    pub const MAX: u8 = 24;
}

impl<'a> Parser<'a> {
    // =========================================================================
    // MAIN EXPRESSION ENTRY POINTS (Public API)
    // =========================================================================

    /// Main expression entry point
    ///
    /// Parses a complete expression using Pratt parsing algorithm.
    /// This is the primary method for parsing expressions in statement context.
    ///
    /// # Returns
    ///
    /// `Ok(Expr)` if parsing succeeds, `Err(&'static str)` on failure
    ///
    /// # Example
    ///
    /// ```
    /// let expr = parser.parse_expression()?;
    /// ```
    pub fn parse_expression(&mut self) -> Result<Expr, &'static str> {
        // Delegate to existing implementation
        self.parse_expr().ok_or("failed to parse expression")
    }

    /// Parse expression with minimum binding power (Pratt parser core)
    ///
    /// This is the heart of the Pratt parsing algorithm. It parses an expression
    /// while respecting operator precedence. The `min_bp` parameter controls
    /// which operators will be parsed - only operators with left binding power
    /// >= min_bp will be consumed.
    ///
    /// # Algorithm
    ///
    /// 1. Parse a prefix expression (atom or unary) as the left-hand side
    /// 2. While the current operator has sufficient binding power:
    ///    - Get the operator's binding powers (left, right)
    ///    - Parse the right-hand side with right_bp as the new minimum
    ///    - Combine into a binary expression
    ///    - Continue with the result as the new left-hand side
    ///
    /// # Associativity
    ///
    /// - Left-associative: right_bp = left_bp + 1 (e.g., `a - b - c` = `(a - b) - c`)
    /// - Right-associative: right_bp = left_bp (e.g., `a = b = c` = `a = (b = c)`)
    ///
    /// # Arguments
    ///
    /// * `min_bp` - Minimum binding power for operators to consume
    ///
    /// # Returns
    ///
    /// `Ok(Expr)` - The parsed expression
    /// `Err(&'static str)` - Error message on failure
    pub fn parse_expression_bp(&mut self, min_bp: u8) -> Result<Expr, &'static str> {
        // Delegate to existing implementation
        self.parse_expr_with_min_bp(min_bp).ok_or("failed to parse expression")
    }

    /// Get binding powers for Pratt parsing
    ///
    /// Returns (left_binding_power, right_binding_power) for the current token.
    /// Higher numbers = tighter binding (higher precedence).
    ///
    /// For left-associative operators: right_bp = left_bp + 1
    /// For right-associative operators: right_bp = left_bp
    ///
    /// # Precedence Levels (lowest to highest)
    ///
    /// | Level | Operators | Left BP | Right BP |
    /// |-------|-----------|---------|----------|
    /// | 1 | `||` | 2 | 3 |
    /// | 2 | `&&` | 4 | 5 |
    /// | 3 | `==`, `!=`, `<`, `<=`, `>`, `>=` | 6 | 7 |
    /// | 4 | `|` | 8 | 9 |
    /// | 5 | `^` | 10 | 11 |
    /// | 6 | `&` | 12 | 13 |
    /// | 7 | `<<`, `>>` | 14 | 15 |
    /// | 8 | `+`, `-` | 16 | 17 |
    /// | 9 | `*`, `/`, `%` | 18 | 19 |
    pub fn binding_power(&self) -> Option<(u8, u8)> {
        // Delegate to existing implementation
        self.infix_binding_power()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BinaryExpr, Block, CallExpr, ClosureExpr, EnumVariantData, FieldExpr, IfExpr, IndexExpr,
        Literal, MatchArm, MatchExpr, MethodCallExpr, Param, Path, Stmt, StructField, Type, UnOp,
        UnaryExpr,
    };
    use faxc_lex::Lexer;
    use faxc_util::Span;

    /// Helper to parse a single expression
    fn parse_expr_source(source: &str) -> (Result<Expr, &'static str>, Handler) {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new(source, &mut handler);

        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            if token == Token::Eof {
                break;
            }
            tokens.push(TokenWithSpan::new(token, Span::DUMMY));
        }

        let mut parser = Parser::from_tokens(tokens, &mut handler, source);
        let expr = parser.parse_expression();

        (expr, handler)
    }

    /// Helper to check expression type
    fn assert_is_binary(expr: &Expr, op: BinOp) {
        match expr {
            Expr::Binary(b) => assert_eq!(b.op, op, "Expected operator {:?}", op),
            _ => panic!("Expected Binary expression, got {:?}", expr),
        }
    }

    fn assert_is_unary(expr: &Expr, op: UnOp) {
        match expr {
            Expr::Unary(u) => assert_eq!(u.op, op, "Expected unary operator {:?}", op),
            _ => panic!("Expected Unary expression, got {:?}", expr),
        }
    }

    // =========================================================================
    // LITERAL TESTS
    // =========================================================================

    #[test]
    fn test_parse_int_literal() {
        let (expr, handler) = parse_expr_source("42");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Literal(Literal::Int(42)))));
    }

    #[test]
    fn test_parse_float_literal() {
        let (expr, handler) = parse_expr_source("3.14");
        assert!(!handler.has_errors());
        if let Ok(Expr::Literal(Literal::Float(f))) = expr {
            assert!((f - 3.14).abs() < 0.001);
        } else {
            panic!("Expected float literal");
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let (expr, handler) = parse_expr_source("\"hello world\"");
        assert!(!handler.has_errors());
        if let Ok(Expr::Literal(Literal::String(s))) = expr {
            assert_eq!(s.as_str(), "hello world");
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_parse_bool_literal() {
        let (expr, handler) = parse_expr_source("true");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Literal(Literal::Bool(true)))));

        let (expr, handler) = parse_expr_source("false");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Literal(Literal::Bool(false)))));
    }

    #[test]
    fn test_parse_unit_literal() {
        let (expr, handler) = parse_expr_source("()");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Literal(Literal::Unit))));
    }

    // =========================================================================
    // UNARY OPERATOR TESTS
    // =========================================================================

    #[test]
    fn test_parse_unary_negation() {
        let (expr, handler) = parse_expr_source("-x");
        assert!(!handler.has_errors());
        assert_is_unary(&expr.unwrap(), UnOp::Neg);
    }

    #[test]
    fn test_parse_unary_not() {
        let (expr, handler) = parse_expr_source("!flag");
        assert!(!handler.has_errors());
        assert_is_unary(&expr.unwrap(), UnOp::Not);
    }

    #[test]
    fn test_parse_unary_bitwise_not() {
        let (expr, handler) = parse_expr_source("~mask");
        assert!(!handler.has_errors());
        assert_is_unary(&expr.unwrap(), UnOp::BitNot);
    }

    #[test]
    fn test_parse_unary_reference() {
        let (expr, handler) = parse_expr_source("&value");
        assert!(!handler.has_errors());
        assert_is_unary(&expr.unwrap(), UnOp::Ref(false));
    }

    #[test]
    fn test_parse_unary_reference_mut() {
        let (expr, handler) = parse_expr_source("&mut value");
        assert!(!handler.has_errors());
        assert_is_unary(&expr.unwrap(), UnOp::Ref(true));
    }

    #[test]
    fn test_parse_chained_unary() {
        let (expr, handler) = parse_expr_source("-!~x");
        assert!(!handler.has_errors());
        // Should parse as -(!(~x))
        let expr = expr.unwrap();
        assert_is_unary(&expr, UnOp::Neg);
    }

    // =========================================================================
    // BINARY OPERATOR PRECEDENCE TESTS
    // =========================================================================

    #[test]
    fn test_precedence_mul_add() {
        // a + b * c should parse as a + (b * c)
        let (expr, handler) = parse_expr_source("a + b * c");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        assert_is_binary(&expr, BinOp::Add);

        if let Expr::Binary(b) = &expr {
            // Right side should be multiplication
            assert_is_binary(&b.right, BinOp::Mul);
        }
    }

    #[test]
    fn test_precedence_multiple_levels() {
        // a + b * c - d / e should parse as (a + (b * c)) - (d / e)
        let (expr, handler) = parse_expr_source("a + b * c - d / e");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        // Top level should be subtraction (left-associative)
        assert_is_binary(&expr, BinOp::Sub);

        if let Expr::Binary(b) = &expr {
            // Left side: a + (b * c)
            assert_is_binary(&b.left, BinOp::Add);
            // Right side: d / e
            assert_is_binary(&b.right, BinOp::Div);
        }
    }

    #[test]
    fn test_precedence_logical_vs_comparison() {
        // a && b == c || d should parse as (a && (b == c)) || d
        let (expr, handler) = parse_expr_source("a && b == c || d");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        // Top level should be OR
        assert_is_binary(&expr, BinOp::Or);
    }

    #[test]
    fn test_precedence_bitwise_hierarchy() {
        // a | b ^ c & d should parse as a | (b ^ (c & d))
        let (expr, handler) = parse_expr_source("a | b ^ c & d");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        // Top level should be bitwise OR
        assert_is_binary(&expr, BinOp::BitOr);

        if let Expr::Binary(b) = &expr {
            // Right side: b ^ (c & d)
            assert_is_binary(&b.right, BinOp::BitXor);
        }
    }

    #[test]
    fn test_precedence_shift_vs_additive() {
        // a + b << c - d should parse as (a + b) << (c - d)
        let (expr, handler) = parse_expr_source("a + b << c - d");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        // Top level should be shift
        assert_is_binary(&expr, BinOp::Shl);
    }

    // =========================================================================
    // ASSOCIATIVITY TESTS
    // =========================================================================

    #[test]
    fn test_associativity_subtraction() {
        // a - b - c should parse as (a - b) - c (left-associative)
        let (expr, handler) = parse_expr_source("a - b - c");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        assert_is_binary(&expr, BinOp::Sub);

        if let Expr::Binary(b) = &expr {
            // Left side should be subtraction
            assert_is_binary(&b.left, BinOp::Sub);
        }
    }

    #[test]
    fn test_associativity_division() {
        // a / b / c should parse as (a / b) / c (left-associative)
        let (expr, handler) = parse_expr_source("a / b / c");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        assert_is_binary(&expr, BinOp::Div);

        if let Expr::Binary(b) = &expr {
            assert_is_binary(&b.left, BinOp::Div);
        }
    }

    #[test]
    fn test_associativity_logical_and() {
        // a && b && c should parse as (a && b) && c
        let (expr, handler) = parse_expr_source("a && b && c");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        assert_is_binary(&expr, BinOp::And);

        if let Expr::Binary(b) = &expr {
            assert_is_binary(&b.left, BinOp::And);
        }
    }

    #[test]
    fn test_associativity_comparison_chain() {
        // a == b == c should parse as (a == b) == c
        let (expr, handler) = parse_expr_source("a == b == c");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        assert_is_binary(&expr, BinOp::Eq);

        if let Expr::Binary(b) = &expr {
            assert_is_binary(&b.left, BinOp::Eq);
        }
    }

    // =========================================================================
    // PARENTHESIZED EXPRESSION TESTS
    // =========================================================================

    #[test]
    fn test_paren_override_precedence() {
        // (a + b) * c should parse as (a + b) * c
        let (expr, handler) = parse_expr_source("(a + b) * c");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        assert_is_binary(&expr, BinOp::Mul);

        if let Expr::Binary(b) = &expr {
            // Left side should be addition
            assert_is_binary(&b.left, BinOp::Add);
        }
    }

    #[test]
    fn test_nested_parens() {
        // ((a + b) * (c - d))
        let (expr, handler) = parse_expr_source("(a + b) * (c - d)");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        assert_is_binary(&expr, BinOp::Mul);
    }

    // =========================================================================
    // FUNCTION CALL TESTS
    // =========================================================================

    #[test]
    fn test_function_call_no_args() {
        let (expr, handler) = parse_expr_source("foo()");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Call(c)) if c.args.is_empty()));
    }

    #[test]
    fn test_function_call_with_args() {
        let (expr, handler) = parse_expr_source("foo(a, b, c)");
        assert!(!handler.has_errors());
        if let Ok(Expr::Call(c)) = &expr {
            assert_eq!(c.args.len(), 3);
        } else {
            panic!("Expected call expression");
        }
    }

    #[test]
    fn test_function_call_in_expression() {
        // foo(a + b, c * d)
        let (expr, handler) = parse_expr_source("foo(a + b, c * d)");
        assert!(!handler.has_errors());

        if let Ok(Expr::Call(c)) = &expr {
            assert_eq!(c.args.len(), 2);
            assert_is_binary(&c.args[0], BinOp::Add);
            assert_is_binary(&c.args[1], BinOp::Mul);
        } else {
            panic!("Expected call expression");
        }
    }

    // =========================================================================
    // FIELD ACCESS AND INDEXING TESTS
    // =========================================================================

    #[test]
    fn test_field_access() {
        let (expr, handler) = parse_expr_source("obj.field");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Field(_))));
    }

    #[test]
    fn test_tuple_index() {
        let (expr, handler) = parse_expr_source("tuple.0");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Field(_))));
    }

    #[test]
    fn test_array_index() {
        let (expr, handler) = parse_expr_source("arr[0]");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Index(_))));
    }

    #[test]
    fn test_chained_access() {
        // obj.field[0].nested
        let (expr, handler) = parse_expr_source("obj.field[0].nested");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Field(_))));
    }

    // =========================================================================
    // METHOD CALL TESTS
    // =========================================================================

    #[test]
    fn test_method_call_no_args() {
        let (expr, handler) = parse_expr_source("obj.method()");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::MethodCall(m)) if m.call_args.is_empty()));
    }

    #[test]
    fn test_method_call_with_args() {
        let (expr, handler) = parse_expr_source("obj.method(a, b)");
        assert!(!handler.has_errors());
        if let Ok(Expr::MethodCall(m)) = &expr {
            assert_eq!(m.call_args.len(), 2);
        } else {
            panic!("Expected method call");
        }
    }

    #[test]
    fn test_method_call_turbofish() {
        let (expr, handler) = parse_expr_source("obj.method::<i32>(a)");
        assert!(!handler.has_errors());
        if let Ok(Expr::MethodCall(m)) = &expr {
            assert!(m.args.is_some());
            assert_eq!(m.args.as_ref().unwrap().len(), 1);
        } else {
            panic!("Expected method call");
        }
    }

    #[test]
    fn test_chained_method_calls() {
        let (expr, handler) = parse_expr_source("obj.method1().method2()");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::MethodCall(_))));
    }

    // =========================================================================
    // IF EXPRESSION TESTS
    // =========================================================================

    #[test]
    fn test_if_expression() {
        let (expr, handler) = parse_expr_source("if cond { a }");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::If(_))));
    }

    #[test]
    fn test_if_else_expression() {
        let (expr, handler) = parse_expr_source("if cond { a } else { b }");
        assert!(!handler.has_errors());
        if let Ok(Expr::If(i)) = &expr {
            assert!(i.else_block.is_some());
        } else {
            panic!("Expected if expression");
        }
    }

    #[test]
    fn test_if_else_if_expression() {
        let (expr, handler) = parse_expr_source("if a { 1 } else if b { 2 } else { 3 }");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::If(_))));
    }

    #[test]
    fn test_if_as_expression_value() {
        // let x = if cond { 1 } else { 2 };
        let (expr, handler) = parse_expr_source("if x > 0 { x } else { -x }");
        assert!(!handler.has_errors());
    }

    // =========================================================================
    // MATCH EXPRESSION TESTS
    // =========================================================================

    #[test]
    fn test_match_expression() {
        let (expr, handler) = parse_expr_source("match x { 0 => \"zero\", _ => \"other\" }");
        assert!(!handler.has_errors());
        if let Ok(Expr::Match(m)) = &expr {
            assert_eq!(m.arms.len(), 2);
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_match_with_guard() {
        let (expr, handler) =
            parse_expr_source("match x { n if n > 0 => \"positive\", _ => \"other\" }");
        assert!(!handler.has_errors());
        if let Ok(Expr::Match(m)) = &expr {
            assert!(m.arms[0].guard.is_some());
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_match_with_block_body() {
        let (expr, handler) =
            parse_expr_source("match x { 0 => { println(\"zero\"); }, _ => {} }");
        assert!(!handler.has_errors());
    }

    // =========================================================================
    // LAMBDA/CLOSURE TESTS
    // =========================================================================

    #[test]
    fn test_closure_pipe_syntax() {
        let (expr, handler) = parse_expr_source("|x: i32| x + 1");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Closure(_))));
    }

    #[test]
    fn test_closure_fn_syntax() {
        let (expr, handler) = parse_expr_source("fn(x: i32) -> i32 { x + 1 }");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Closure(_))));
    }

    #[test]
    fn test_closure_no_params() {
        let (expr, handler) = parse_expr_source("|| 42");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Closure(_))));
    }

    #[test]
    fn test_closure_with_block() {
        let (expr, handler) = parse_expr_source("|x| { let y = x + 1; y }");
        assert!(!handler.has_errors());
    }

    // =========================================================================
    // BLOCK EXPRESSION TESTS
    // =========================================================================

    #[test]
    fn test_block_expression() {
        let (expr, handler) = parse_expr_source("{ let x = 1; x + 1 }");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Block(_))));
    }

    #[test]
    fn test_block_with_trailing_expr() {
        let (expr, handler) = parse_expr_source("{ 1 + 2 }");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_nested_block() {
        let (expr, handler) = parse_expr_source("{ { 1 } }");
        assert!(!handler.has_errors());
    }

    // =========================================================================
    // ARRAY LITERAL TESTS
    // =========================================================================

    #[test]
    fn test_array_literal() {
        let (expr, handler) = parse_expr_source("[1, 2, 3]");
        assert!(!handler.has_errors());
        if let Ok(Expr::Array(arr)) = &expr {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_array_empty() {
        let (expr, handler) = parse_expr_source("[]");
        assert!(!handler.has_errors());
        if let Ok(Expr::Array(arr)) = &expr {
            assert!(arr.is_empty());
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_array_with_expressions() {
        let (expr, handler) = parse_expr_source("[a + b, c * d]");
        assert!(!handler.has_errors());
        if let Ok(Expr::Array(arr)) = &expr {
            assert_eq!(arr.len(), 2);
            assert_is_binary(&arr[0], BinOp::Add);
            assert_is_binary(&arr[1], BinOp::Mul);
        } else {
            panic!("Expected array");
        }
    }

    // =========================================================================
    // TUPLE TESTS
    // =========================================================================

    #[test]
    fn test_tuple_literal() {
        let (expr, handler) = parse_expr_source("(1, 2, 3)");
        assert!(!handler.has_errors());
        if let Ok(Expr::Tuple(t)) = &expr {
            assert_eq!(t.len(), 3);
        } else {
            panic!("Expected tuple");
        }
    }

    #[test]
    fn test_tuple_single_with_comma() {
        let (expr, handler) = parse_expr_source("(1,)");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Tuple(_))));
    }

    // =========================================================================
    // COMPLEX EXPRESSION TESTS
    // =========================================================================

    #[test]
    fn test_complex_arithmetic() {
        // a * b + c * d - e / f
        let (expr, handler) = parse_expr_source("a * b + c * d - e / f");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_complex_logical() {
        // (a && b) || (c && d)
        let (expr, handler) = parse_expr_source("(a && b) || (c && d)");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_mixed_operators() {
        // a + b * c == d && e || f
        let (expr, handler) = parse_expr_source("a + b * c == d && e || f");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_deeply_nested() {
        // ((((a))))
        let (expr, handler) = parse_expr_source("((((a))))");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_function_with_complex_args() {
        // foo(a + b * c, if x { y } else { z })
        let (expr, handler) = parse_expr_source("foo(a + b * c, if x { y } else { z })");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_match_in_if() {
        // if match x { 0 => true, _ => false } { a } else { b }
        let (expr, handler) =
            parse_expr_source("if match x { 0 => true, _ => false } { a } else { b }");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_closure_in_call() {
        // map(fn(x) { x + 1 })
        let (expr, handler) = parse_expr_source("map(fn(x: i32) -> i32 { x + 1 })");
        assert!(!handler.has_errors());
    }

    // =========================================================================
    // ERROR CASE TESTS
    // =========================================================================

    #[test]
    fn test_error_missing_operand_binary() {
        // a + (missing operand)
        let (expr, handler) = parse_expr_source("a +");
        assert!(handler.has_errors());
    }

    #[test]
    fn test_error_missing_operand_unary() {
        let (expr, handler) = parse_expr_source("-");
        assert!(handler.has_errors());
    }

    #[test]
    fn test_error_invalid_token() {
        let (expr, handler) = parse_expr_source("@invalid");
        assert!(handler.has_errors());
    }

    #[test]
    fn test_error_unmatched_paren() {
        let (expr, handler) = parse_expr_source("(a + b");
        assert!(handler.has_errors());
    }

    #[test]
    fn test_error_unmatched_brace() {
        let (expr, handler) = parse_expr_source("if x { a");
        assert!(handler.has_errors());
    }

    #[test]
    fn test_error_unmatched_bracket() {
        let (expr, handler) = parse_expr_source("arr[0");
        assert!(handler.has_errors());
    }

    #[test]
    fn test_error_double_operator() {
        let (expr, handler) = parse_expr_source("a ++ b");
        assert!(handler.has_errors());
    }

    // =========================================================================
    // SPEC EXAMPLES
    // =========================================================================

    #[test]
    fn test_spec_fibonacci_expression() {
        // fib(n - 1) + fib(n - 2)
        let (expr, handler) = parse_expr_source("fib(n - 1) + fib(n - 2)");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        assert_is_binary(&expr, BinOp::Add);
    }

    #[test]
    fn test_spec_max_expression() {
        // if a > b { a } else { b }
        let (expr, handler) = parse_expr_source("if a > b { a } else { b }");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_spec_complex_condition() {
        // a > b && c < d || e == f
        let (expr, handler) = parse_expr_source("a > b && c < d || e == f");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_spec_bitwise_example() {
        // 5 & 3, 5 | 3, 5 ^ 3, 4 << 1, 8 >> 1
        let tests = vec![
            ("5 & 3", BinOp::BitAnd),
            ("5 | 3", BinOp::BitOr),
            ("5 ^ 3", BinOp::BitXor),
            ("4 << 1", BinOp::Shl),
            ("8 >> 1", BinOp::Shr),
        ];

        for (source, expected_op) in tests {
            let (expr, handler) = parse_expr_source(source);
            assert!(!handler.has_errors(), "Failed for: {}", source);
            assert_is_binary(&expr.unwrap(), expected_op);
        }
    }

    #[test]
    fn test_spec_unary_precedence() {
        // -5 + 3 should parse as (-5) + 3
        let (expr, handler) = parse_expr_source("-5 + 3");
        assert!(!handler.has_errors());

        let expr = expr.unwrap();
        assert_is_binary(&expr, BinOp::Add);

        if let Expr::Binary(b) = &expr {
            assert_is_unary(&b.left, UnOp::Neg);
        }
    }

    // =========================================================================
    // BINDING POWER TESTS
    // =========================================================================

    #[test]
    fn test_binding_power_logical_or() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("||", &mut handler);
        let tokens: Vec<TokenWithSpan> = std::iter::from_fn(|| {
            let token = lexer.next_token();
            if token == Token::Eof {
                None
            } else {
                Some(TokenWithSpan::new(token, Span::DUMMY))
            }
        })
        .collect();

        let parser = Parser::from_tokens(tokens, &mut handler, "||");
        let bp = parser.binding_power();
        assert_eq!(bp, Some((bp::LOGICAL_OR, bp::LOGICAL_OR + 1)));
    }

    #[test]
    fn test_binding_power_logical_and() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("&&", &mut handler);
        let tokens: Vec<TokenWithSpan> = std::iter::from_fn(|| {
            let token = lexer.next_token();
            if token == Token::Eof {
                None
            } else {
                Some(TokenWithSpan::new(token, Span::DUMMY))
            }
        })
        .collect();

        let parser = Parser::from_tokens(tokens, &mut handler, "&&");
        let bp = parser.binding_power();
        assert_eq!(bp, Some((bp::LOGICAL_AND, bp::LOGICAL_AND + 1)));
    }

    #[test]
    fn test_binding_power_comparison() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("==", &mut handler);
        let tokens: Vec<TokenWithSpan> = std::iter::from_fn(|| {
            let token = lexer.next_token();
            if token == Token::Eof {
                None
            } else {
                Some(TokenWithSpan::new(token, Span::DUMMY))
            }
        })
        .collect();

        let parser = Parser::from_tokens(tokens, &mut handler, "==");
        let bp = parser.binding_power();
        assert_eq!(bp, Some((bp::COMPARISON, bp::COMPARISON + 1)));
    }

    #[test]
    fn test_binding_power_multiplicative() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("*", &mut handler);
        let tokens: Vec<TokenWithSpan> = std::iter::from_fn(|| {
            let token = lexer.next_token();
            if token == Token::Eof {
                None
            } else {
                Some(TokenWithSpan::new(token, Span::DUMMY))
            }
        })
        .collect();

        let parser = Parser::from_tokens(tokens, &mut handler, "*");
        let bp = parser.binding_power();
        assert_eq!(bp, Some((bp::MULTIPLICATIVE, bp::MULTIPLICATIVE + 1)));
    }

    #[test]
    fn test_binding_power_not_operator() {
        let mut handler = Handler::new();
        let mut lexer = Lexer::new("ident", &mut handler);
        let tokens: Vec<TokenWithSpan> = std::iter::from_fn(|| {
            let token = lexer.next_token();
            if token == Token::Eof {
                None
            } else {
                Some(TokenWithSpan::new(token, Span::DUMMY))
            }
        })
        .collect();

        let parser = Parser::from_tokens(tokens, &mut handler, "ident");
        let bp = parser.binding_power();
        assert_eq!(bp, None);
    }

    // =========================================================================
    // CHARACTER LITERAL TESTS
    // =========================================================================

    #[test]
    fn test_char_literal() {
        let (expr, handler) = parse_expr_source("'a'");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Literal(Literal::Char('a')))));
    }

    #[test]
    fn test_char_literal_escape() {
        let (expr, handler) = parse_expr_source("'\\n'");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Literal(Literal::Char('\n')))));
    }

    #[test]
    fn test_char_literal_unicode() {
        let (expr, handler) = parse_expr_source("'\\u{1F600}'");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Literal(Literal::Char('😀')))));
    }

    // =========================================================================
    // CAST EXPRESSION TESTS
    // =========================================================================

    #[test]
    fn test_cast_expression() {
        let (expr, handler) = parse_expr_source("x as i32");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Cast(_, _))));
    }

    #[test]
    fn test_cast_expression_precedence() {
        // Cast binds tighter than arithmetic: (x as i32) + 1
        let (expr, handler) = parse_expr_source("x as i32 + 1");
        assert!(!handler.has_errors());
        
        let expr = expr.unwrap();
        // Top level should be addition
        assert!(matches!(expr, Expr::Binary(b) if b.op == BinOp::Add));
        
        if let Expr::Binary(b) = &expr {
            // Left side should be cast
            assert!(matches!(b.left, Expr::Cast(_, _)));
        }
    }

    #[test]
    fn test_cast_expression_chain() {
        let (expr, handler) = parse_expr_source("x as i32 as i64");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::Cast(_, _))));
    }

    #[test]
    fn test_cast_with_multiplication() {
        // x as i32 * 2 should parse as (x as i32) * 2
        let (expr, handler) = parse_expr_source("x as i32 * 2");
        assert!(!handler.has_errors());
        
        let expr = expr.unwrap();
        assert!(matches!(expr, Expr::Binary(b) if b.op == BinOp::Mul));
    }

    // =========================================================================
    // STRUCT LITERAL TESTS
    // =========================================================================

    #[test]
    fn test_struct_literal() {
        let (expr, handler) = parse_expr_source("Point { x: 1.0, y: 2.0 }");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Ok(Expr::StructLiteral(_))));
    }

    #[test]
    fn test_struct_literal_shorthand() {
        let (expr, handler) = parse_expr_source("Point { x, y }");
        assert!(!handler.has_errors());
        
        if let Ok(Expr::StructLiteral(s)) = &expr {
            assert_eq!(s.fields.len(), 2);
            assert!(s.fields[0].is_shorthand);
            assert!(s.fields[1].is_shorthand);
        } else {
            panic!("Expected struct literal");
        }
    }

    #[test]
    fn test_struct_literal_mixed() {
        let (expr, handler) = parse_expr_source("Point { x, y: 2.0 }");
        assert!(!handler.has_errors());
        
        if let Ok(Expr::StructLiteral(s)) = &expr {
            assert_eq!(s.fields.len(), 2);
            assert!(s.fields[0].is_shorthand);
            assert!(!s.fields[1].is_shorthand);
        } else {
            panic!("Expected struct literal");
        }
    }

    #[test]
    fn test_struct_literal_with_base() {
        let (expr, handler) = parse_expr_source("Point { x: 1.0, ..base }");
        assert!(!handler.has_errors());
        
        if let Ok(Expr::StructLiteral(s)) = &expr {
            assert!(s.base.is_some());
        } else {
            panic!("Expected struct literal");
        }
    }

    #[test]
    fn test_struct_literal_turbofish() {
        let (expr, handler) = parse_expr_source("Vec::<i32> { data: 1 }");
        assert!(!handler.has_errors());
        
        if let Ok(Expr::StructLiteral(s)) = &expr {
            assert!(s.generics.is_some());
        } else {
            panic!("Expected struct literal");
        }
    }

    #[test]
    fn test_struct_literal_empty() {
        let (expr, handler) = parse_expr_source("Empty {}");
        assert!(!handler.has_errors());
        
        if let Ok(Expr::StructLiteral(s)) = &expr {
            assert!(s.fields.is_empty());
        } else {
            panic!("Expected struct literal");
        }
    }

    // =========================================================================
    // ENUM VARIANT CONSTRUCTION TESTS
    // =========================================================================

    #[test]
    fn test_enum_variant_unit() {
        let (expr, handler) = parse_expr_source("Option::None");
        assert!(!handler.has_errors());
        
        if let Ok(Expr::EnumVariant(e)) = &expr {
            assert_eq!(e.variant.as_str(), "None");
            assert!(matches!(e.data, EnumVariantData::Unit));
        } else {
            panic!("Expected enum variant");
        }
    }

    #[test]
    fn test_enum_variant_tuple() {
        let (expr, handler) = parse_expr_source("Option::Some(42)");
        assert!(!handler.has_errors());
        
        if let Ok(Expr::EnumVariant(e)) = &expr {
            assert_eq!(e.variant.as_str(), "Some");
            if let EnumVariantData::Tuple(args) = &e.data {
                assert_eq!(args.len(), 1);
            } else {
                panic!("Expected tuple variant");
            }
        } else {
            panic!("Expected enum variant");
        }
    }

    #[test]
    fn test_enum_variant_tuple_multiple_args() {
        let (expr, handler) = parse_expr_source("Result::Ok(value, extra)");
        assert!(!handler.has_errors());
        
        if let Ok(Expr::EnumVariant(e)) = &expr {
            if let EnumVariantData::Tuple(args) = &e.data {
                assert_eq!(args.len(), 2);
            } else {
                panic!("Expected tuple variant");
            }
        } else {
            panic!("Expected enum variant");
        }
    }

    #[test]
    fn test_enum_variant_struct() {
        let (expr, handler) = parse_expr_source("Message::Click { x: 1, y: 2 }");
        assert!(!handler.has_errors());
        
        if let Ok(Expr::EnumVariant(e)) = &expr {
            assert_eq!(e.variant.as_str(), "Click");
            if let EnumVariantData::Struct(fields) = &e.data {
                assert_eq!(fields.len(), 2);
            } else {
                panic!("Expected struct variant");
            }
        } else {
            panic!("Expected enum variant");
        }
    }

    #[test]
    fn test_enum_variant_turbofish() {
        let (expr, handler) = parse_expr_source("Option::Some::<i32>(42)");
        assert!(!handler.has_errors());
        
        if let Ok(Expr::EnumVariant(e)) = &expr {
            assert!(e.generics.is_some());
            assert_eq!(e.generics.as_ref().unwrap().len(), 1);
        } else {
            panic!("Expected enum variant");
        }
    }

    #[test]
    fn test_enum_variant_in_expression() {
        let (expr, handler) = parse_expr_source("match x { None => 0, Some(n) => n }");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_enum_variant_nested() {
        let (expr, handler) = parse_expr_source("Outer::Inner::Variant(1)");
        assert!(!handler.has_errors());
    }
}
