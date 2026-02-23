//! faxc-par - Parser (Syntactic Analyzer) for the Fax Programming Language
//!
//! This crate provides a complete recursive descent parser with Pratt parsing
//! for expressions. It transforms a stream of tokens into an Abstract Syntax
//! Tree (AST).
//!
//! # Overview
//!
//! The parser uses a combination of:
//! - **Recursive Descent** for statements and declarations
//! - **Pratt Parsing** (Top-Down Operator Precedence) for expressions
//! - **Panic Mode** error recovery for robust error handling
//!
//! # Example Usage
//!
//! ```
//! use faxc_util::Handler;
//! use faxc_lex::{Lexer, Token};
//! use faxc_par::{Parser, Ast};
//!
//! let source = "fn main() { println(\"Hello\"); }";
//! let mut handler = Handler::new();
//! let mut lexer = Lexer::new(source, &mut handler);
//!
//! // Collect tokens
//! let mut tokens = Vec::new();
//! loop {
//!     let token = lexer.next_token();
//!     if token == Token::Eof {
//!         break;
//!     }
//!     tokens.push(token);
//! }
//!
//! // Parse
//! let mut parser = Parser::new(tokens, &mut handler);
//! let ast = parser.parse();
//! ```
//!
//! # Error Recovery
//!
//! When encountering syntax errors, the parser:
//! 1. Reports a clear diagnostic with location
//! 2. Skips tokens to the next synchronization point
//! 3. Continues parsing to find additional errors
//!
//! Synchronization points include:
//! - Statement terminators (`;`)
//! - Block boundaries (`{`, `}`)
//! - Top-level item keywords (`fn`, `struct`, `enum`, etc.)

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

#[cfg(test)]
mod edge_cases;

use faxc_lex::Token;
use faxc_util::{Handler, Span, Symbol};

// ============================================================================
// AST NODE DEFINITIONS
// ============================================================================

/// AST root - a source file contains a list of items
pub type Ast = Vec<Item>;

/// Top-level item in a source file
#[derive(Debug, Clone)]
pub enum Item {
    /// Function definition
    Fn(FnItem),

    /// Structure definition
    Struct(StructItem),

    /// Enumeration definition
    Enum(EnumItem),

    /// Trait definition
    Trait(TraitItem),

    /// Implementation block
    Impl(ImplItem),

    /// Module import
    Use(UseItem),

    /// Constant definition
    Const(ConstItem),

    /// Static variable definition
    Static(StaticItem),
}

/// Function item
#[derive(Debug, Clone)]
pub struct FnItem {
    /// Function name
    pub name: Symbol,

    /// Generic parameters (if any)
    pub generics: Vec<GenericParam>,

    /// Function parameters
    pub params: Vec<Param>,

    /// Return type (None means unit type `()`)
    pub ret_type: Option<Type>,

    /// Function body
    pub body: Block,

    /// Visibility modifier
    pub visibility: Visibility,

    /// Source location
    pub span: Span,

    /// Async modifier
    pub async_kw: bool,

    /// Where clause constraints
    pub where_clause: Option<WhereClause>,
}

/// Generic parameter
#[derive(Debug, Clone)]
pub struct GenericParam {
    /// Parameter name (e.g., "T")
    pub name: Symbol,

    /// Trait bounds
    pub bounds: Vec<Type>,
}

/// Where clause constraint
#[derive(Debug, Clone)]
pub struct WhereClause {
    /// Bounds in the where clause
    pub bounds: Vec<WhereBound>,
}

/// A single where bound (e.g., `T: Trait1 + Trait2`)
#[derive(Debug, Clone)]
pub struct WhereBound {
    /// The type being constrained
    pub ty: Type,
    /// Trait bounds
    pub traits: Vec<Path>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    /// Parameter name
    pub name: Symbol,

    /// Parameter type
    pub ty: Type,

    /// Mutability
    pub mutable: bool,
}

/// Structure item
#[derive(Debug, Clone)]
pub struct StructItem {
    /// Struct name
    pub name: Symbol,

    /// Generic parameters
    pub generics: Vec<GenericParam>,

    /// Fields
    pub fields: Vec<Field>,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub span: Span,

    /// Where clause constraints
    pub where_clause: Option<WhereClause>,
}

/// Field definition
#[derive(Debug, Clone)]
pub struct Field {
    /// Field name
    pub name: Symbol,

    /// Field type
    pub ty: Type,

    /// Visibility
    pub visibility: Visibility,
}

/// Enum item
#[derive(Debug, Clone)]
pub struct EnumItem {
    /// Enum name
    pub name: Symbol,

    /// Generic parameters
    pub generics: Vec<GenericParam>,

    /// Variants
    pub variants: Vec<Variant>,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub span: Span,

    /// Where clause constraints
    pub where_clause: Option<WhereClause>,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct Variant {
    /// Variant name
    pub name: Symbol,

    /// Variant data
    pub data: VariantData,
}

/// Variant data types
#[derive(Debug, Clone)]
pub enum VariantData {
    /// Unit variant (e.g., `None`)
    Unit,

    /// Tuple variant (e.g., `Some(T)`)
    Tuple(Vec<Type>),

    /// Struct variant
    Struct(Vec<Field>),
}

/// Trait item
#[derive(Debug, Clone)]
pub struct TraitItem {
    /// Trait name
    pub name: Symbol,

    /// Generic parameters
    pub generics: Vec<GenericParam>,

    /// Trait items
    pub items: Vec<TraitMember>,

    /// Supertraits
    pub supertraits: Vec<Type>,

    /// Visibility
    pub visibility: Visibility,
}

/// Trait member
#[derive(Debug, Clone)]
pub enum TraitMember {
    /// Method signature
    Method(FnSig),

    /// Associated type
    Type(Symbol, Vec<Type>),

    /// Associated constant
    Const(Symbol, Type, Option<Expr>),
}

/// Function signature (without body)
#[derive(Debug, Clone)]
pub struct FnSig {
    pub name: Symbol,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub ret_type: Option<Type>,
}

/// Implementation item
#[derive(Debug, Clone)]
pub struct ImplItem {
    /// Generic parameters
    pub generics: Vec<GenericParam>,

    /// Trait being implemented
    pub trait_ref: Option<Type>,

    /// Type being implemented
    pub self_ty: Type,

    /// Items in the impl block
    pub items: Vec<ImplMember>,

    /// Where clause constraints
    pub where_clause: Option<WhereClause>,
}

/// Implementation member
#[derive(Debug, Clone)]
pub enum ImplMember {
    /// Method implementation
    Method(FnItem),

    /// Associated type definition
    Type(Symbol, Type),

    /// Associated constant
    Const(Symbol, Type, Expr),
}

/// Use/import item
#[derive(Debug, Clone)]
pub struct UseItem {
    /// Import path
    pub path: Path,

    /// Alias
    pub alias: Option<Symbol>,

    /// Glob import
    pub is_glob: bool,
}

/// Constant item
#[derive(Debug, Clone)]
pub struct ConstItem {
    /// Constant name
    pub name: Symbol,

    /// Constant type
    pub ty: Type,

    /// Constant value
    pub value: Expr,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub span: Span,
}

/// Static item
#[derive(Debug, Clone)]
pub struct StaticItem {
    /// Static name
    pub name: Symbol,

    /// Static type
    pub ty: Type,

    /// Static value
    pub value: Expr,

    /// Mutability
    pub mutable: bool,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub span: Span,
}

/// Visibility modifier
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    /// Public visibility
    Public,

    /// Private (default)
    Private,

    /// Public within crate
    Crate,

    /// Public within parent module
    Super,

    /// Public within path
    Restricted(Path),
}

// ============================================================================
// STATEMENTS
// ============================================================================

/// Statement
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Let binding
    Let(LetStmt),

    /// Expression statement
    Expr(Expr),

    /// Return statement
    Return(Option<Expr>),

    /// If statement
    If(IfStmt),

    /// While loop
    While(WhileStmt),

    /// For loop
    For(ForStmt),

    /// Break statement
    Break(Option<Symbol>),

    /// Continue statement
    Continue(Option<Symbol>),

    /// Item statement
    Item(Item),
}

/// Let statement
#[derive(Debug, Clone)]
pub struct LetStmt {
    /// Pattern being bound
    pub pattern: Pattern,

    /// Type annotation
    pub ty: Option<Type>,

    /// Initializer expression
    pub init: Option<Expr>,

    /// Mutability
    pub mutable: bool,
}

/// If statement
#[derive(Debug, Clone)]
pub struct IfStmt {
    /// Condition
    pub cond: Expr,

    /// Then block
    pub then_block: Block,

    /// Else clause
    pub else_clause: Option<Box<ElseClause>>,
}

/// Else clause
#[derive(Debug, Clone)]
pub enum ElseClause {
    /// Else block
    Block(Block),

    /// Else-if
    If(IfStmt),
}

/// While loop
#[derive(Debug, Clone)]
pub struct WhileStmt {
    /// Condition
    pub cond: Expr,

    /// Loop body
    pub body: Block,

    /// Label
    pub label: Option<Symbol>,
}

/// For loop
#[derive(Debug, Clone)]
pub struct ForStmt {
    /// Pattern binding
    pub pattern: Pattern,

    /// Iterator expression
    pub iter: Expr,

    /// Loop body
    pub body: Block,

    /// Label
    pub label: Option<Symbol>,
}

/// Block expression
#[derive(Debug, Clone)]
pub struct Block {
    /// Statements
    pub stmts: Vec<Stmt>,

    /// Trailing expression
    pub trailing: Option<Box<Expr>>,

    /// Source location
    pub span: Span,
}

// ============================================================================
// EXPRESSIONS
// ============================================================================

/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal value
    Literal(Literal),

    /// Variable/path reference
    Path(Path),

    /// Binary operation
    Binary(BinaryExpr),

    /// Unary operation
    Unary(UnaryExpr),

    /// Function call
    Call(CallExpr),

    /// Method call
    MethodCall(MethodCallExpr),

    /// Field access
    Field(FieldExpr),

    /// Index expression
    Index(IndexExpr),

    /// Block expression
    Block(Block),

    /// If expression
    If(IfExpr),

    /// Match expression
    Match(MatchExpr),

    /// Closure expression
    Closure(ClosureExpr),

    /// Assignment expression
    Assign(AssignExpr),

    /// Compound assignment
    CompoundAssign(CompoundAssignExpr),

    /// Return expression
    Return(Option<Box<Expr>>),

    /// Break expression
    Break(Option<Box<Expr>>, Option<Symbol>),

    /// Continue expression
    Continue(Option<Symbol>),

    /// Tuple expression
    Tuple(Vec<Expr>),

    /// Array expression
    Array(Vec<Expr>),

    /// Range expression
    Range(RangeExpr),

    /// Cast expression
    Cast(Box<Expr>, Type),

    /// Async block
    Async(AsyncExpr),

    /// Await expression
    Await(Box<Expr>),

    /// Struct literal expression
    /// Example: `Point { x: 1.0, y: 2.0 }`
    StructLiteral(Box<StructLiteralExpr>),

    /// Enum variant construction
    /// Example: `Option::Some(42)` or `Option::None`
    EnumVariant(Box<EnumVariantExpr>),
}

/// Literal expression
#[derive(Debug, Clone)]
pub enum Literal {
    /// Integer literal
    Int(i64),

    /// Float literal
    Float(f64),

    /// String literal
    String(Symbol),

    /// Character literal
    Char(char),

    /// Boolean literal
    Bool(bool),

    /// Unit literal
    Unit,
}

/// Path expression
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    /// Path segments
    pub segments: Vec<PathSegment>,
}

/// Path segment
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    /// Segment name
    pub ident: Symbol,

    /// Generic arguments
    pub args: Option<Vec<Type>>,
}

/// Binary expression
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    // Logical
    And,
    Or,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Unary expression
#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnOp,
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    /// Negation
    Neg,

    /// Logical not
    Not,

    /// Bitwise not
    BitNot,

    /// Dereference
    Deref,

    /// Reference
    Ref(bool),
}

/// Function call expression
#[derive(Debug, Clone)]
pub struct CallExpr {
    pub func: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span,
    pub generics: Option<Vec<Type>>,
}

/// Method call expression
#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub receiver: Box<Expr>,
    pub method: Symbol,
    pub args: Option<Vec<Type>>,
    pub call_args: Vec<Expr>,
}

/// Field access expression
#[derive(Debug, Clone)]
pub struct FieldExpr {
    pub object: Box<Expr>,
    pub field: Symbol,
    pub span: Span,
}

/// Index expression
#[derive(Debug, Clone)]
pub struct IndexExpr {
    pub object: Box<Expr>,
    pub index: Box<Expr>,
}

/// If expression
#[derive(Debug, Clone)]
pub struct IfExpr {
    pub cond: Box<Expr>,
    pub then_block: Block,
    pub else_block: Option<Box<Expr>>,
}

/// Match expression
#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// Closure expression
#[derive(Debug, Clone)]
pub struct ClosureExpr {
    pub params: Vec<Param>,
    pub ret_type: Option<Type>,
    pub body: Box<Expr>,
    pub move_kw: bool,
}

/// Async expression
#[derive(Debug, Clone)]
pub struct AsyncExpr {
    pub body: Block,
    pub move_kw: bool,
}

/// Assignment expression
#[derive(Debug, Clone)]
pub struct AssignExpr {
    pub place: Box<Expr>,
    pub value: Box<Expr>,
}

/// Compound assignment expression
#[derive(Debug, Clone)]
pub struct CompoundAssignExpr {
    pub place: Box<Expr>,
    pub op: BinOp,
    pub value: Box<Expr>,
}

/// Range expression
#[derive(Debug, Clone)]
pub struct RangeExpr {
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
    pub inclusive: bool,
}

/// Struct literal expression
/// Example: `Point { x: 1.0, y: 2.0 }` or `Point { x }` (shorthand)
#[derive(Debug, Clone)]
pub struct StructLiteralExpr {
    /// Struct type path
    pub path: Path,

    /// Generic arguments (turbofish syntax: `Struct::<T> { ... }`)
    pub generics: Option<Vec<Type>>,

    /// Fields in the struct literal
    pub fields: Vec<StructField>,

    /// Base struct for struct update syntax: `Struct { ..base }`
    pub base: Option<Expr>,
}

/// Field in a struct literal
#[derive(Debug, Clone)]
pub struct StructField {
    /// Field name
    pub name: Symbol,

    /// Field value expression
    pub expr: Expr,

    /// Whether this is a shorthand field (name only, no `: expr`)
    pub is_shorthand: bool,
}

/// Enum variant construction expression
/// Example: `Option::Some(42)`, `Option::None`, or `Result::Ok { value: 1 }`
#[derive(Debug, Clone)]
pub struct EnumVariantExpr {
    /// Enum type path
    pub path: Path,

    /// Variant name
    pub variant: Symbol,

    /// Generic arguments (turbofish syntax: `Enum::Variant::<T>(...)`)
    pub generics: Option<Vec<Type>>,

    /// Variant data
    pub data: EnumVariantData,
}

/// Enum variant data types
#[derive(Debug, Clone)]
pub enum EnumVariantData {
    /// Unit variant: `Option::None`
    Unit,

    /// Tuple variant: `Option::Some(42)`
    Tuple(Vec<Expr>),

    /// Struct variant: `Enum::Variant { field: expr }`
    Struct(Vec<StructField>),
}

impl Expr {
    /// Get the span of an expression if available
    pub fn span(&self) -> Option<Span> {
        match self {
            Expr::Binary(b) => Some(b.span),
            Expr::Unary(u) => Some(u.span),
            Expr::Call(c) => Some(c.span),
            Expr::Field(f) => Some(f.span),
            Expr::Block(b) => Some(b.span),
            Expr::Literal(_) => None,
            Expr::Path(_) => None,
            Expr::MethodCall(_) => None,
            Expr::Index(_) => None,
            Expr::If(_) => None,
            Expr::Match(_) => None,
            Expr::Closure(_) => None,
            Expr::Assign(_) => None,
            Expr::CompoundAssign(_) => None,
            Expr::Return(_) => None,
            Expr::Break(_, _) => None,
            Expr::Continue(_) => None,
            Expr::Tuple(_) => None,
            Expr::Array(_) => None,
            Expr::Range(_) => None,
            Expr::Cast(_, _) => None,
            Expr::Async(_) => None,
            Expr::Await(_) => None,
            Expr::StructLiteral(_) => None,
            Expr::EnumVariant(_) => None,
        }
    }
}

// ============================================================================
// PATTERNS
// ============================================================================

/// Pattern
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Wildcard pattern
    Wildcard,

    /// Identifier pattern
    Ident(Symbol, Mutability),

    /// Literal pattern
    Literal(Literal),

    /// Path pattern
    Path(Path),

    /// Struct pattern
    Struct(Path, Vec<FieldPattern>),

    /// Tuple struct pattern
    TupleStruct(Path, Vec<Pattern>),

    /// Tuple pattern
    Tuple(Vec<Pattern>),

    /// Array/slice pattern
    Slice(Vec<Pattern>),
}

/// Field in struct pattern
#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub field: Symbol,
    pub pattern: Pattern,
}

// ============================================================================
// TYPES
// ============================================================================

/// Type expression
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Unit type
    Unit,

    /// Never type
    Never,

    /// Path type
    Path(Path),

    /// Generic type
    Generic(Box<Type>, Vec<Type>),

    /// Reference type
    Reference(Box<Type>, Mutability),

    /// Pointer type
    Pointer(Box<Type>, Mutability),

    /// Slice type
    Slice(Box<Type>),

    /// Array type
    Array(Box<Type>, usize),

    /// Tuple type
    Tuple(Vec<Type>),

    /// Function type
    Fn(Vec<Type>, Box<Type>),

    /// Trait object type
    TraitObject(Vec<Type>),

    /// Impl trait type
    ImplTrait(Vec<Type>),

    /// Inferred type
    Inferred,
}

/// Mutability
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mutability {
    Mutable,
    Immutable,
}

/// A token with its source location span
#[derive(Debug, Clone)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Span,
}

impl TokenWithSpan {
    fn new(token: Token, span: Span) -> Self {
        Self { token, span }
    }
}

// ============================================================================
// PARSER
// ============================================================================

/// Recursive descent parser with Pratt parsing for expressions
///
/// The parser uses LL(2) lookahead for disambiguation and implements
/// panic-mode error recovery for robust error handling.
pub struct Parser<'a> {
    /// Token stream with spans
    tokens: Vec<TokenWithSpan>,

    /// Current position in token stream
    position: usize,

    /// Error handler
    handler: &'a mut Handler,

    /// Source code (for span calculation)
    source: &'a str,
}

impl<'a> Parser<'a> {
    /// Create a new parser from tokens
    ///
    /// # Arguments
    ///
    /// * `tokens` - Vector of tokens from the lexer
    /// * `handler` - Diagnostic handler for error reporting
    ///
    /// # Example
    ///
    /// ```
    /// use faxc_util::Handler;
    /// use faxc_lex::{Lexer, Token};
    /// use faxc_par::Parser;
    ///
    /// let source = "let x = 42;";
    /// let mut handler = Handler::new();
    /// let mut lexer = Lexer::new(source, &mut handler);
    ///
    /// let mut tokens = Vec::new();
    /// let mut start = 0;
    /// loop {
    ///     let token = lexer.next_token();
    ///     if token == Token::Eof { break; }
    ///     tokens.push(faxc_par::TokenWithSpan::new(
    ///         token,
    ///         Span::DUMMY, // In production, calculate actual span
    ///     ));
    /// }
    ///
    /// let mut parser = Parser::from_tokens(tokens, &mut handler, source);
    /// let ast = parser.parse();
    /// ```
    pub fn from_tokens(
        tokens: Vec<TokenWithSpan>,
        handler: &'a mut Handler,
        source: &'a str,
    ) -> Self {
        Self {
            tokens,
            position: 0,
            handler,
            source,
        }
    }

    /// Create a new parser from raw tokens (without spans)
    ///
    /// This is a convenience method that creates dummy spans.
    /// For production use, prefer `from_tokens` with proper spans.
    pub fn new(tokens: Vec<Token>, handler: &'a mut Handler) -> Self {
        let tokens_with_span: Vec<TokenWithSpan> = tokens
            .into_iter()
            .map(|t| TokenWithSpan::new(t, Span::DUMMY))
            .collect();
        Self {
            tokens: tokens_with_span,
            position: 0,
            handler,
            source: "",
        }
    }

    /// Parse a complete source file
    ///
    /// # Returns
    ///
    /// A vector of top-level items (the AST)
    ///
    /// # Example
    ///
    /// ```
    /// # use faxc_util::Handler;
    /// # use faxc_lex::{Lexer, Token};
    /// # use faxc_par::{Parser, TokenWithSpan};
    /// let source = "fn main() { }";
    /// let mut handler = Handler::new();
    /// let mut lexer = Lexer::new(source, &mut handler);
    ///
    /// let mut tokens = Vec::new();
    /// loop {
    ///     let token = lexer.next_token();
    ///     if token == Token::Eof { break; }
    ///     tokens.push(TokenWithSpan::new(token, Span::DUMMY));
    /// }
    ///
    /// let mut parser = Parser::from_tokens(tokens, &mut handler, source);
    /// let ast = parser.parse();
    /// ```
    pub fn parse(&mut self) -> Ast {
        let mut items = Vec::new();

        while !self.is_at_end() {
            match self.parse_item() {
                Some(item) => items.push(item),
                None => {
                    // Error recovery: skip to sync point
                    self.recover_to_sync_point();
                },
            }
        }

        items
    }

    // ========================================================================
    // ITEM PARSING
    // ========================================================================

    /// Parse a single top-level item
    fn parse_item(&mut self) -> Option<Item> {
        let visibility = self.parse_visibility();

        // Check for async before fn
        let async_kw = self.match_token(Token::Async);

        match self.current_token() {
            Token::Fn => self.parse_fn_item(visibility, async_kw),
            Token::Struct => self.parse_struct_item(visibility),
            Token::Enum => self.parse_enum_item(visibility),
            Token::Trait => self.parse_trait_item(visibility),
            Token::Impl => self.parse_impl_item(),
            Token::Use => self.parse_use_item(),
            Token::Mod => self.parse_mod_item(visibility),
            Token::Const => self.parse_const_item(visibility),
            Token::Static => self.parse_static_item(visibility),
            _ => {
                self.error(
                    "expected item: fn, struct, enum, trait, impl, use, mod, const, or static",
                );
                None
            },
        }
    }

    /// Parse visibility modifier
    fn parse_visibility(&mut self) -> Visibility {
        if !self.match_token(Token::Pub) {
            return Visibility::Private;
        }

        // Check for restricted visibility: pub(crate), pub(super), pub(in path)
        if self.match_token(Token::LParen) {
            let vis = match self.current_token() {
                Token::Crate => {
                    self.advance();
                    Visibility::Crate
                },
                Token::Super => {
                    self.advance();
                    Visibility::Super
                },
                Token::Ident(sym) if sym.as_str() == "in" => {
                    self.advance();
                    let path = self.parse_path();
                    Visibility::Restricted(path)
                },
                _ => {
                    self.error("expected 'crate', 'super', or 'in' in visibility");
                    Visibility::Public
                },
            };
            self.expect(Token::RParen);
            vis
        } else {
            Visibility::Public
        }
    }

    /// Parse function item
    fn parse_fn_item(&mut self, visibility: Visibility, async_kw: bool) -> Option<Item> {
        let span_start = self.current_span();

        self.expect(Token::Fn)?;

        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        let params = self.parse_params()?;
        let where_clause = self.parse_where_clause();
        let ret_type = self.parse_return_type();
        let body = self.parse_block()?;

        let span = self.span_from_start(span_start);

        Some(Item::Fn(FnItem {
            name,
            generics,
            params,
            ret_type,
            body,
            visibility,
            span,
            async_kw,
            where_clause,
        }))
    }

    /// Parse generic parameters
    fn parse_generics(&mut self) -> Vec<GenericParam> {
        if !self.match_token(Token::Lt) {
            return Vec::new();
        }

        let mut params = Vec::new();

        while !self.is_at_end() && self.current_token() != Token::Gt {
            let name = match self.parse_ident() {
                Some(n) => n,
                None => break,
            };

            let mut bounds = Vec::new();
            if self.match_token(Token::Colon) {
                // Parse trait bounds (simplified)
                loop {
                    if let Some(ty) = self.parse_type() {
                        bounds.push(ty);
                    }
                    if !self.match_token(Token::Plus) {
                        break;
                    }
                }
            }

            params.push(GenericParam { name, bounds });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::Gt);
        params
    }

    /// Parse where clause
    fn parse_where_clause(&mut self) -> Option<WhereClause> {
        if !self.match_token(Token::Where) {
            return None;
        }

        let mut bounds = Vec::new();

        loop {
            let ty = self.parse_type()?;

            self.expect(Token::Colon)?;

            let mut traits = Vec::new();
            loop {
                let path = self.parse_path();
                traits.push(path);

                if !self.match_token(Token::Plus) {
                    break;
                }
            }

            bounds.push(WhereBound { ty, traits });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        Some(WhereClause { bounds })
    }

    /// Parse function parameters
    fn parse_params(&mut self) -> Option<Vec<Param>> {
        self.expect(Token::LParen)?;

        let mut params = Vec::new();

        if !self.match_token(Token::RParen) {
            loop {
                let mutable = self.match_token(Token::Mut);
                let name = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;

                params.push(Param { name, ty, mutable });

                if !self.match_token(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::RParen)?;
        }

        Some(params)
    }

    /// Parse return type
    fn parse_return_type(&mut self) -> Option<Type> {
        if !self.match_token(Token::Arrow) {
            return None;
        }
        self.parse_type()
    }

    /// Parse struct item
    fn parse_struct_item(&mut self, visibility: Visibility) -> Option<Item> {
        let span_start = self.current_span();

        self.expect(Token::Struct)?;

        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        let where_clause = self.parse_where_clause();

        self.expect(Token::LBrace)?;

        let mut fields = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBrace {
            let field_vis = self.parse_visibility();
            let field_name = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let field_ty = self.parse_type()?;

            fields.push(Field {
                name: field_name,
                ty: field_ty,
                visibility: field_vis,
            });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        let span = self.span_from_start(span_start);

        Some(Item::Struct(StructItem {
            name,
            generics,
            fields,
            visibility,
            span,
            where_clause,
        }))
    }

    /// Parse enum item
    fn parse_enum_item(&mut self, visibility: Visibility) -> Option<Item> {
        let span_start = self.current_span();

        self.expect(Token::Enum)?;

        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        let where_clause = self.parse_where_clause();

        self.expect(Token::LBrace)?;

        let mut variants = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBrace {
            let variant_name = self.parse_ident()?;

            let data = if self.match_token(Token::LParen) {
                // Tuple variant
                let mut types = Vec::new();
                while !self.is_at_end() && self.current_token() != Token::RParen {
                    if let Some(ty) = self.parse_type() {
                        types.push(ty);
                    }
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RParen);
                VariantData::Tuple(types)
            } else if self.match_token(Token::LBrace) {
                // Struct variant
                let mut fields = Vec::new();
                while !self.is_at_end() && self.current_token() != Token::RBrace {
                    let field_name = self.parse_ident()?;
                    self.expect(Token::Colon)?;
                    let field_ty = self.parse_type()?;
                    fields.push(Field {
                        name: field_name,
                        ty: field_ty,
                        visibility: Visibility::Private,
                    });
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RBrace);
                VariantData::Struct(fields)
            } else {
                VariantData::Unit
            };

            variants.push(Variant {
                name: variant_name,
                data,
            });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        let span = self.span_from_start(span_start);

        Some(Item::Enum(EnumItem {
            name,
            generics,
            variants,
            visibility,
            span,
            where_clause,
        }))
    }

    /// Parse trait item
    fn parse_trait_item(&mut self, visibility: Visibility) -> Option<Item> {
        let _span_start = self.current_span();

        self.expect(Token::Trait)?;

        let name = self.parse_ident()?;
        let generics = self.parse_generics();

        // Parse supertraits (simplified)
        let mut supertraits = Vec::new();
        if self.match_token(Token::Colon) {
            loop {
                if let Some(ty) = self.parse_type() {
                    supertraits.push(ty);
                }
                if !self.match_token(Token::Plus) {
                    break;
                }
            }
        }

        self.expect(Token::LBrace)?;

        let mut items = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBrace {
            // Parse trait members (simplified - just method signatures for now)
            if self.current_token() == Token::Fn {
                if let Some(sig) = self.parse_fn_sig() {
                    items.push(TraitMember::Method(sig));
                }
            } else {
                self.recover_to_stmt_sync();
            }

            if self.match_token(Token::Semicolon) {
                continue;
            }
        }

        self.expect(Token::RBrace)?;

        Some(Item::Trait(TraitItem {
            name,
            generics,
            items,
            supertraits,
            visibility,
        }))
    }

    /// Parse function signature (for traits)
    fn parse_fn_sig(&mut self) -> Option<FnSig> {
        self.expect(Token::Fn)?;

        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        let params = self.parse_params()?;
        let ret_type = self.parse_return_type();

        // Consume semicolon if present
        self.match_token(Token::Semicolon);

        Some(FnSig {
            name,
            generics,
            params,
            ret_type,
        })
    }

    /// Parse impl item
    fn parse_impl_item(&mut self) -> Option<Item> {
        let _span_start = self.current_span();

        self.expect(Token::Impl)?;

        let generics = self.parse_generics();
        let where_clause = self.parse_where_clause();

        // Check for trait impl: impl Trait for Type
        let trait_ref = if self.current_token() != Token::For {
            let ty = self.parse_type()?;
            if self.match_token(Token::For) {
                Some(ty)
            } else {
                // Inherent impl
                let self_ty = ty;
                self.expect(Token::LBrace)?;

                let mut items = Vec::new();
                while !self.is_at_end() && self.current_token() != Token::RBrace {
                    if self.current_token() == Token::Fn {
                        if let Some(item) = self.parse_item() {
                            if let Item::Fn(fn_item) = item {
                                items.push(ImplMember::Method(fn_item));
                            }
                        }
                    } else {
                        self.recover_to_stmt_sync();
                    }
                }
                self.expect(Token::RBrace)?;

                return Some(Item::Impl(ImplItem {
                    generics,
                    trait_ref: None,
                    self_ty,
                    items,
                    where_clause,
                }));
            }
        } else {
            None
        };

        self.expect(Token::For)?;
        let self_ty = self.parse_type()?;

        self.expect(Token::LBrace)?;

        let mut items = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBrace {
            if self.current_token() == Token::Fn {
                if let Some(item) = self.parse_item() {
                    if let Item::Fn(fn_item) = item {
                        items.push(ImplMember::Method(fn_item));
                    }
                }
            } else {
                self.recover_to_stmt_sync();
            }
        }
        self.expect(Token::RBrace)?;

        Some(Item::Impl(ImplItem {
            generics,
            trait_ref,
            self_ty,
            items,
            where_clause,
        }))
    }

    /// Parse use item
    fn parse_use_item(&mut self) -> Option<Item> {
        let _span_start = self.current_span();

        self.expect(Token::Use)?;

        let path = self.parse_path();
        let mut alias = None;
        let mut is_glob = false;

        if self.match_token(Token::As) {
            alias = self.parse_ident();
        }

        if self.match_token(Token::Star) {
            is_glob = true;
        }

        self.expect(Token::Semicolon)?;

        Some(Item::Use(UseItem {
            path,
            alias,
            is_glob,
        }))
    }

    /// Parse mod item
    fn parse_mod_item(&mut self, _visibility: Visibility) -> Option<Item> {
        self.expect(Token::Mod)?;
        let _name = self.parse_ident()?;

        if self.match_token(Token::Semicolon) {
            // External module
            return None;
        }

        if self.match_token(Token::LBrace) {
            // Inline module - parse items inside
            let mut items = Vec::new();
            while !self.is_at_end() && self.current_token() != Token::RBrace {
                if let Some(item) = self.parse_item() {
                    items.push(item);
                } else {
                    self.recover_to_sync_point();
                }
            }
            self.expect(Token::RBrace)?;
        }

        // For now, return None for mod items (they're handled differently)
        None
    }

    /// Parse const item
    fn parse_const_item(&mut self, visibility: Visibility) -> Option<Item> {
        let span_start = self.current_span();

        self.expect(Token::Const)?;
        let name = self.parse_ident()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(Token::Semicolon)?;

        let span = self.span_from_start(span_start);

        Some(Item::Const(ConstItem {
            name,
            ty,
            value,
            visibility,
            span,
        }))
    }

    /// Parse static item
    fn parse_static_item(&mut self, visibility: Visibility) -> Option<Item> {
        let span_start = self.current_span();

        self.expect(Token::Static)?;
        let mutable = self.match_token(Token::Mut);
        let name = self.parse_ident()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(Token::Semicolon)?;

        let span = self.span_from_start(span_start);

        Some(Item::Static(StaticItem {
            name,
            ty,
            value,
            mutable,
            visibility,
            span,
        }))
    }

    // ========================================================================
    // STATEMENT PARSING
    // ========================================================================

    /// Parse a statement
    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.current_token() {
            Token::Let => self.parse_let_stmt(),
            Token::If => self.parse_if_stmt(),
            Token::While => self.parse_while_stmt(),
            Token::For => self.parse_for_stmt(),
            Token::Return => self.parse_return_stmt(),
            Token::Break => self.parse_break_stmt(),
            Token::Continue => self.parse_continue_stmt(),
            Token::LBrace => {
                // Block statement
                let block = self.parse_block()?;
                Some(Stmt::Expr(Expr::Block(block)))
            },
            _ => {
                // Try expression statement
                let expr = self.parse_expr()?;

                // Check for assignment
                if self.match_token(Token::Eq) {
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon);
                    return Some(Stmt::Expr(Expr::Assign(AssignExpr {
                        place: Box::new(expr),
                        value: Box::new(value),
                    })));
                }

                // Check for compound assignment
                if let Some(op) = self.parse_compound_assign_op() {
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon);
                    return Some(Stmt::Expr(Expr::CompoundAssign(CompoundAssignExpr {
                        place: Box::new(expr),
                        op,
                        value: Box::new(value),
                    })));
                }

                // Regular expression statement
                if self.match_token(Token::Semicolon) {
                    Some(Stmt::Expr(expr))
                } else if self.is_at_end() || self.current_token() == Token::RBrace {
                    // Trailing expression in block
                    Some(Stmt::Expr(expr))
                } else {
                    self.expect(Token::Semicolon);
                    Some(Stmt::Expr(expr))
                }
            },
        }
    }

    /// Parse compound assignment operator
    fn parse_compound_assign_op(&mut self) -> Option<BinOp> {
        match self.current_token() {
            Token::PlusEq => {
                self.advance();
                Some(BinOp::Add)
            },
            Token::MinusEq => {
                self.advance();
                Some(BinOp::Sub)
            },
            Token::StarEq => {
                self.advance();
                Some(BinOp::Mul)
            },
            Token::SlashEq => {
                self.advance();
                Some(BinOp::Div)
            },
            Token::PercentEq => {
                self.advance();
                Some(BinOp::Mod)
            },
            Token::AmpersandEq => {
                self.advance();
                Some(BinOp::BitAnd)
            },
            Token::PipeEq => {
                self.advance();
                Some(BinOp::BitOr)
            },
            Token::CaretEq => {
                self.advance();
                Some(BinOp::BitXor)
            },
            Token::ShlEq => {
                self.advance();
                Some(BinOp::Shl)
            },
            Token::ShrEq => {
                self.advance();
                Some(BinOp::Shr)
            },
            _ => None,
        }
    }

    /// Parse let statement
    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        let _span_start = self.current_span();

        self.expect(Token::Let)?;

        let mutable = self.match_token(Token::Mut);
        let pattern = self.parse_pattern()?;

        let ty = if self.match_token(Token::Colon) {
            self.parse_type()
        } else {
            None
        };

        let init = if self.match_token(Token::Eq) {
            self.parse_expr()
        } else {
            None
        };

        self.expect(Token::Semicolon)?;

        Some(Stmt::Let(LetStmt {
            pattern,
            ty,
            init,
            mutable,
        }))
    }

    /// Parse if statement
    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        let _span_start = self.current_span();

        self.expect(Token::If)?;

        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_clause = if self.match_token(Token::Else) {
            if self.match_token(Token::If) {
                // else if - parse nested if and extract IfStmt
                if let Some(Stmt::If(if_stmt)) = self.parse_if_stmt() {
                    Some(Box::new(ElseClause::If(if_stmt)))
                } else {
                    None
                }
            } else {
                // else block
                let block = self.parse_block()?;
                Some(Box::new(ElseClause::Block(block)))
            }
        } else {
            None
        };

        Some(Stmt::If(IfStmt {
            cond,
            then_block,
            else_clause,
        }))
    }

    /// Parse while statement
    fn parse_while_stmt(&mut self) -> Option<Stmt> {
        self.expect(Token::While)?;

        let cond = self.parse_expr()?;
        let body = self.parse_block()?;

        Some(Stmt::While(WhileStmt {
            cond,
            body,
            label: None,
        }))
    }

    /// Parse for statement
    fn parse_for_stmt(&mut self) -> Option<Stmt> {
        self.expect(Token::For)?;

        let pattern = self.parse_pattern()?;

        // Check for 'in' keyword (handled as identifier in lexer)
        let is_in = match self.current_token() {
            Token::Ident(sym) => sym.as_str() == "in",
            _ => false,
        };
        if !is_in {
            self.error("expected 'in' after pattern in for loop");
            return None;
        }
        self.advance();

        let iter = self.parse_expr()?;
        let body = self.parse_block()?;

        Some(Stmt::For(ForStmt {
            pattern,
            iter,
            body,
            label: None,
        }))
    }

    /// Parse return statement
    fn parse_return_stmt(&mut self) -> Option<Stmt> {
        self.expect(Token::Return)?;

        let expr = if self.current_token() != Token::Semicolon
            && self.current_token() != Token::RBrace
            && !self.is_at_end()
        {
            self.parse_expr()
        } else {
            None
        };

        self.expect(Token::Semicolon)?;

        Some(Stmt::Return(expr))
    }

    /// Parse break statement
    fn parse_break_stmt(&mut self) -> Option<Stmt> {
        self.expect(Token::Break)?;

        let label = if let Token::Ident(_sym) = self.current_token() {
            // Check if it's a label (not an expression)
            // For simplicity, we don't support break with value yet
            None
        } else {
            None
        };

        self.expect(Token::Semicolon)?;

        Some(Stmt::Break(label))
    }

    /// Parse continue statement
    fn parse_continue_stmt(&mut self) -> Option<Stmt> {
        self.expect(Token::Continue)?;

        let label = None;

        self.expect(Token::Semicolon)?;

        Some(Stmt::Continue(label))
    }

    // ========================================================================
    // EXPRESSION PARSING (PRATT PARSER)
    // ========================================================================

    /// Parse expression using Pratt parsing
    ///
    /// This implements operator precedence parsing with the following
    /// precedence levels (lowest to highest):
    /// 1. `||` (logical or)
    /// 2. `&&` (logical and)
    /// 3. `==`, `!=`, `<`, `<=`, `>`, `>=` (comparison)
    /// 4. `|` (bitwise or)
    /// 5. `^` (bitwise xor)
    /// 6. `&` (bitwise and)
    /// 7. `<<`, `>>` (shift)
    /// 8. `+`, `-` (arithmetic)
    /// 9. `*`, `/`, `%` (multiplicative)
    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_expr_with_min_bp(0)
    }

    /// Parse expression with minimum binding power (Pratt parser)
    fn parse_expr_with_min_bp(&mut self, min_bp: u8) -> Option<Expr> {
        // Parse prefix (atom or prefix operator)
        let mut lhs = self.parse_prefix()?;

        // Parse infix operators with sufficient binding power
        loop {
            let (_lbp, rbp) = match self.infix_binding_power() {
                Some(bp) if bp.0 >= min_bp => bp,
                _ => break,
            };

            let op_token = self.current_token();
            let op_span = self.current_span();

            // Special handling for range expression
            if op_token == Token::DotDot || op_token == Token::DotDotEq {
                self.advance();
                let inclusive = op_token == Token::DotDotEq;

                // Parse end expression (optional)
                let end = if matches!(
                    self.current_token(),
                    Token::Semicolon
                        | Token::RParen
                        | Token::RBrace
                        | Token::RBracket
                        | Token::Comma
                        | Token::FatArrow
                        | Token::Eof
                ) {
                    None
                } else {
                    Some(Box::new(self.parse_expr_with_min_bp(rbp)?))
                };

                lhs = Expr::Range(RangeExpr {
                    start: Some(Box::new(lhs)),
                    end,
                    inclusive,
                });
                continue;
            }

            // Special handling for cast expression (takes a type, not an expression)
            if op_token == Token::As {
                self.advance();
                let cast_type = self.parse_type()?;
                lhs = Expr::Cast(Box::new(lhs), cast_type);
                continue;
            }

            self.advance();

            let rhs = self.parse_expr_with_min_bp(rbp)?;

            let op = self.token_to_binop(op_token)?;
            let span = self.span_from_start(op_span);

            lhs = Expr::Binary(BinaryExpr {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
                span,
            });
        }

        // Handle prefix range: ..end or ..=end (when min_bp is 0)
        if min_bp == 0 {
            if let Token::DotDot | Token::DotDotEq = self.current_token() {
                let op_token = self.current_token();
                self.advance();
                let inclusive = op_token == Token::DotDotEq;

                let end = if matches!(
                    self.current_token(),
                    Token::Semicolon
                        | Token::RParen
                        | Token::RBrace
                        | Token::RBracket
                        | Token::Comma
                        | Token::FatArrow
                        | Token::Eof
                ) {
                    None
                } else {
                    Some(Box::new(self.parse_expr()?))
                };

                return Some(Expr::Range(RangeExpr {
                    start: None,
                    end,
                    inclusive,
                }));
            }
        }

        Some(lhs)
    }

    /// Parse prefix expression (atom or prefix operator)
    fn parse_prefix(&mut self) -> Option<Expr> {
        let span_start = self.current_span();

        match self.current_token() {
            // Prefix operators
            Token::Minus => {
                self.advance();
                let expr = self.parse_prefix()?;
                let span = self.span_from_start(span_start);
                Some(Expr::Unary(UnaryExpr {
                    op: UnOp::Neg,
                    expr: Box::new(expr),
                    span,
                }))
            },
            Token::Bang => {
                self.advance();
                let expr = self.parse_prefix()?;
                let span = self.span_from_start(span_start);
                Some(Expr::Unary(UnaryExpr {
                    op: UnOp::Not,
                    expr: Box::new(expr),
                    span,
                }))
            },
            Token::Tilde => {
                self.advance();
                let expr = self.parse_prefix()?;
                let span = self.span_from_start(span_start);
                Some(Expr::Unary(UnaryExpr {
                    op: UnOp::BitNot,
                    expr: Box::new(expr),
                    span,
                }))
            },
            Token::Star => {
                // Prefix * is dereference
                self.advance();
                let expr = self.parse_prefix()?;
                let span = self.span_from_start(span_start);
                Some(Expr::Unary(UnaryExpr {
                    op: UnOp::Deref,
                    expr: Box::new(expr),
                    span,
                }))
            },
            Token::Ampersand => {
                self.advance();
                let mutable = self.match_token(Token::Mut);
                let expr = self.parse_prefix()?;
                let span = self.span_from_start(span_start);
                Some(Expr::Unary(UnaryExpr {
                    op: UnOp::Ref(mutable),
                    expr: Box::new(expr),
                    span,
                }))
            },

            // Literals
            Token::Number(n) => {
                self.advance();
                Some(Expr::Literal(Literal::Int(n as i64)))
            },
            Token::Float(n) => {
                self.advance();
                Some(Expr::Literal(Literal::Float(n)))
            },
            Token::String(s) => {
                self.advance();
                Some(Expr::Literal(Literal::String(s)))
            },
            Token::Char(c) => {
                self.advance();
                Some(Expr::Literal(Literal::Char(c)))
            },
            Token::True => {
                self.advance();
                Some(Expr::Literal(Literal::Bool(true)))
            },
            Token::False => {
                self.advance();
                Some(Expr::Literal(Literal::Bool(false)))
            },

            // Identifiers and paths
            Token::Ident(_) | Token::Self_ | Token::SelfUpper | Token::Super | Token::Crate => {
                self.parse_path_expr()
            },

            // Parenthesized expressions, tuples, closures
            Token::LParen => self.parse_paren_or_tuple_or_closure(),

            // Block, struct literal, or closure
            Token::LBrace => self.parse_block_expr(),

            // Array literal
            Token::LBracket => self.parse_array_expr(),

            // Control flow expressions
            Token::If => self.parse_if_expr(),
            Token::Match => self.parse_match_expr(),
            Token::While => self.parse_while_expr(),
            Token::For => self.parse_for_expr(),
            Token::Loop => self.parse_loop_expr(),

            // Async expression
            Token::Async => self.parse_async_expr(),

            // Return expression (in expression context)
            Token::Return => {
                self.advance();
                let expr = if self.current_token() != Token::Semicolon
                    && self.current_token() != Token::RBrace
                    && !self.is_at_end()
                {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                Some(Expr::Return(expr))
            },

            // Break expression
            Token::Break => {
                self.advance();
                self.expect(Token::Semicolon);
                Some(Expr::Break(None, None))
            },

            // Continue expression
            Token::Continue => {
                self.advance();
                self.expect(Token::Semicolon);
                Some(Expr::Continue(None))
            },

            // Closure with `fn` syntax
            Token::Fn => {
                self.advance();
                self.parse_closure_body()
            },

            // Closure with pipe syntax: |x| x + 1
            Token::Pipe => self.parse_closure_pipe(),

            _ => {
                self.error("expected expression");
                None
            },
        }
    }

    /// Parse closure with pipe syntax: |params| body
    fn parse_closure_pipe(&mut self) -> Option<Expr> {
        let span_start = self.current_span();

        // Parse parameters between pipes
        let params = self.parse_closure_params()?;
        self.expect(Token::Pipe)?;

        // Parse closure body (can be expression or block)
        let body = if self.current_token() == Token::LBrace {
            let block = self.parse_block()?;
            Expr::Block(block)
        } else {
            self.parse_expr()?
        };

        Some(Expr::Closure(ClosureExpr {
            params,
            ret_type: None,
            body: Box::new(body),
            move_kw: false,
        }))
    }

    /// Parse path or function call
    ///
    /// Handles:
    /// - Simple path: `foo`, `foo::bar`
    /// - Function call: `foo()`, `foo(a, b)`, `foo::<T>(a, b)`
    /// - Method call: `obj.method()`, `obj.method::<T>(a)`
    /// - Field access: `obj.field`
    /// - Struct literal: `Struct { field: expr }`, `Struct { field }` (shorthand)
    /// - Enum variant: `Enum::Variant`, `Enum::Variant(args)`, `Enum::Variant { field }`
    fn parse_path_expr(&mut self) -> Option<Expr> {
        let span_start = self.current_span();
        let path = self.parse_path();

        // Extract generics from the last path segment (turbofish syntax)
        let path_generics = path.segments.last().and_then(|s| s.args.clone());

        // Check for enum variant construction: Enum::Variant
        if self.match_token(Token::ColonColon) {
            // This is Enum::Variant syntax
            if let Some(variant) = self.parse_ident() {
                // Check for turbofish: Enum::Variant::<T>
                let generics = if self.current_token() == Token::ColonColon
                    && self.peek_token() == Token::Lt
                {
                    self.advance(); // consume ::
                    self.advance(); // consume <
                    let mut types = Vec::new();
                    while !self.is_at_end() && self.current_token() != Token::Gt {
                        if let Some(ty) = self.parse_type() {
                            types.push(ty);
                        }
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::Gt);
                    Some(types)
                } else {
                    None
                };

                // Check for tuple variant: Enum::Variant(args)
                if self.match_token(Token::LParen) {
                    let mut args = Vec::new();
                    while !self.is_at_end() && self.current_token() != Token::RParen {
                        if let Some(arg) = self.parse_expr() {
                            args.push(arg);
                        }
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;

                    return Some(Expr::EnumVariant(Box::new(EnumVariantExpr {
                        path,
                        variant,
                        generics,
                        data: EnumVariantData::Tuple(args),
                    })));
                }

                // Check for struct variant: Enum::Variant { fields }
                if self.match_token(Token::LBrace) {
                    let fields = self.parse_struct_fields()?;
                    return Some(Expr::EnumVariant(Box::new(EnumVariantExpr {
                        path,
                        variant,
                        generics,
                        data: EnumVariantData::Struct(fields),
                    })));
                }

                // Unit variant: Enum::Variant
                return Some(Expr::EnumVariant(Box::new(EnumVariantExpr {
                    path,
                    variant,
                    generics,
                    data: EnumVariantData::Unit,
                })));
            }
        }

        // Check for struct literal: Struct { fields }
        // But not if this looks like match arms (contains =>) or a block expression
        if self.current_token() == Token::LBrace {
            // Peek ahead to check if content has => (match arms) or looks like block
            if !self.looks_like_match_arms() && !self.looks_like_block() {
                self.advance(); // consume LBrace
                                // Use generics from path (turbofish already parsed by parse_path)
                let generics = path_generics.clone();

                let fields = self.parse_struct_fields()?;

                // Check for struct update syntax: Struct { ..base }
                let base = if self.match_token(Token::DotDot) {
                    self.parse_expr()
                } else {
                    None
                };

                return Some(Expr::StructLiteral(Box::new(StructLiteralExpr {
                    path,
                    generics,
                    fields,
                    base,
                })));
            }
        }

        // Check for function call
        if self.match_token(Token::LParen) {
            // Use generics from path (turbofish already parsed by parse_path)
            let generics = path_generics.clone();

            let mut args = Vec::new();
            while !self.is_at_end() && self.current_token() != Token::RParen {
                if let Some(arg) = self.parse_expr() {
                    args.push(arg);
                }
                if !self.match_token(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::RParen)?;

            let span = self.span_from_start(span_start);
            return Some(Expr::Call(CallExpr {
                func: Box::new(Expr::Path(path)),
                args,
                span,
                generics,
            }));
        }

        // Check for method call or field access
        if self.match_token(Token::Dot) {
            if let Some(field) = self.parse_ident() {
                // Check for turbofish: method::<T>()
                let generics = if self.current_token() == Token::ColonColon
                    && self.peek_token() == Token::Lt
                {
                    self.advance(); // consume ::
                    self.advance(); // consume <
                    let mut types = Vec::new();
                    while !self.is_at_end() && self.current_token() != Token::Gt {
                        if let Some(ty) = self.parse_type() {
                            types.push(ty);
                        }
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::Gt);
                    Some(types)
                } else {
                    None
                };

                // Parse call arguments
                let mut call_args = Vec::new();
                if self.match_token(Token::LParen) {
                    while !self.is_at_end() && self.current_token() != Token::RParen {
                        if let Some(arg) = self.parse_expr() {
                            call_args.push(arg);
                        }
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;

                    // This is a method call
                    return Some(Expr::MethodCall(MethodCallExpr {
                        receiver: Box::new(Expr::Path(path)),
                        method: field,
                        args: generics,
                        call_args,
                    }));
                }

                // This is a field access
                return Some(Expr::Field(FieldExpr {
                    object: Box::new(Expr::Path(path)),
                    field,
                    span: self.span_from_start(span_start),
                }));
            }
        }

        Some(Expr::Path(path))
    }

    /// Parse parenthesized expression, tuple, or closure
    fn parse_paren_or_tuple_or_closure(&mut self) -> Option<Expr> {
        let span_start = self.current_span();

        self.expect(Token::LParen)?;

        // Check for empty tuple (unit)
        if self.match_token(Token::RParen) {
            return Some(Expr::Literal(Literal::Unit));
        }

        // Check for closure: |params| body
        if self.match_token(Token::Pipe) {
            let params = self.parse_closure_params()?;
            self.expect(Token::Pipe)?;

            // Parse closure body (can be expression or block)
            let body = if self.current_token() == Token::LBrace {
                let block = self.parse_block()?;
                Expr::Block(block)
            } else {
                self.parse_expr()?
            };

            return Some(Expr::Closure(ClosureExpr {
                params,
                ret_type: None,
                body: Box::new(body),
                move_kw: false,
            }));
        }

        // Parse expressions for tuple or parenthesized expression
        let mut exprs = Vec::new();
        loop {
            if let Some(expr) = self.parse_expr() {
                exprs.push(expr);
            }
            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RParen)?;

        // Single expression in parens vs tuple
        if exprs.len() == 1 {
            Some(exprs.into_iter().next().unwrap())
        } else {
            Some(Expr::Tuple(exprs))
        }
    }

    /// Parse closure parameters (between pipes)
    fn parse_closure_params(&mut self) -> Option<Vec<Param>> {
        let mut params = Vec::new();

        while !self.is_at_end() && self.current_token() != Token::Pipe {
            let mutable = self.match_token(Token::Mut);
            let name = self.parse_ident()?;

            let ty = if self.match_token(Token::Colon) {
                self.parse_type()?
            } else {
                Type::Inferred
            };

            params.push(Param { name, ty, mutable });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        Some(params)
    }

    /// Parse struct literal fields: `{ field: expr, field2 }` (shorthand)
    ///
    /// Handles:
    /// - Named fields: `field: expr`
    /// - Shorthand: `field` (same as `field: field`)
    /// - Base struct: `..base` (struct update syntax)
    fn parse_struct_fields(&mut self) -> Option<Vec<StructField>> {
        let mut fields = Vec::new();

        self.expect(Token::LBrace)?;

        while !self.is_at_end() && self.current_token() != Token::RBrace {
            // Check for base struct: ..base
            if self.match_token(Token::DotDot) {
                // Base struct is handled separately in parse_path_expr
                break;
            }

            let field_name = self.parse_ident()?;

            // Check for shorthand: just the field name (no colon)
            if self.current_token() != Token::Colon {
                // Shorthand: field name is used as both name and value
                fields.push(StructField {
                    name: field_name,
                    expr: Expr::Path(Path {
                        segments: vec![PathSegment {
                            ident: field_name,
                            args: None,
                        }],
                    }),
                    is_shorthand: true,
                });
            } else {
                // Named field: field: expr
                self.expect(Token::Colon)?;
                let expr = self.parse_expr()?;
                fields.push(StructField {
                    name: field_name,
                    expr,
                    is_shorthand: false,
                });
            }

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBrace)?;
        Some(fields)
    }

    /// Parse closure with `fn` syntax: `fn(x: i32) -> i32 { x + 1 }`
    fn parse_closure_body(&mut self) -> Option<Expr> {
        let params = self.parse_params()?;
        let ret_type = self.parse_return_type();

        let body = if self.current_token() == Token::LBrace {
            let block = self.parse_block()?;
            Expr::Block(block)
        } else {
            self.parse_expr()?
        };

        Some(Expr::Closure(ClosureExpr {
            params,
            ret_type,
            body: Box::new(body),
            move_kw: false,
        }))
    }

    /// Parse block expression
    fn parse_block_expr(&mut self) -> Option<Expr> {
        let block = self.parse_block()?;
        Some(Expr::Block(block))
    }

    /// Parse block
    fn parse_block(&mut self) -> Option<Block> {
        let span_start = self.current_span();

        self.expect(Token::LBrace)?;

        let mut stmts = Vec::new();
        let mut trailing = None;

        while !self.is_at_end() && self.current_token() != Token::RBrace {
            if let Some(stmt) = self.parse_stmt() {
                // Check if this is an expression statement that could be trailing
                if let Stmt::Expr(_) = stmt {
                    if self.current_token() == Token::RBrace || self.is_at_end() {
                        // This is a trailing expression
                        if let Stmt::Expr(expr) = stmt {
                            trailing = Some(Box::new(expr));
                        }
                        break;
                    }
                }
                stmts.push(stmt);
            } else {
                self.recover_to_stmt_sync();
            }
        }

        self.expect(Token::RBrace)?;

        let span = self.span_from_start(span_start);

        Some(Block {
            stmts,
            trailing,
            span,
        })
    }

    /// Check if current position could be a trailing expression
    fn is_trailing_expr(&mut self) -> bool {
        // If next token after potential expr would be RBrace or EOF
        matches!(
            self.current_token(),
            Token::If
                | Token::Match
                | Token::While
                | Token::For
                | Token::Loop
                | Token::LBrace
                | Token::LParen
                | Token::LBracket
                | Token::Fn
                | Token::Async
                | Token::Return
                | Token::Break
                | Token::Continue
                | Token::Ident(_)
                | Token::Self_
                | Token::SelfUpper
                | Token::Super
                | Token::Crate
                | Token::Number(_)
                | Token::Float(_)
                | Token::String(_)
                | Token::True
                | Token::False
                | Token::Minus
                | Token::Bang
                | Token::Tilde
                | Token::Ampersand
        )
    }

    /// Parse if expression
    fn parse_if_expr(&mut self) -> Option<Expr> {
        let span_start = self.current_span();

        self.expect(Token::If)?;

        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_block = if self.match_token(Token::Else) {
            if self.match_token(Token::If) {
                // Nested if-else as else block
                let inner_if = self.parse_if_expr()?;
                Some(Box::new(inner_if))
            } else {
                let block = self.parse_block()?;
                Some(Box::new(Expr::Block(block)))
            }
        } else {
            None
        };

        Some(Expr::If(IfExpr {
            cond: Box::new(cond),
            then_block,
            else_block,
        }))
    }

    /// Parse match expression
    fn parse_match_expr(&mut self) -> Option<Expr> {
        let span_start = self.current_span();

        self.expect(Token::Match)?;

        let scrutinee = self.parse_expr()?;

        self.expect(Token::LBrace)?;

        let mut arms = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBrace {
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(Token::If) {
                self.parse_expr()
            } else {
                None
            };

            self.expect(Token::FatArrow)?;

            // Parse match arm body as expression (blocks are handled by expression parser)
            let body = self.parse_expr()?;

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        Some(Expr::Match(MatchExpr {
            scrutinee: Box::new(scrutinee),
            arms,
        }))
    }

    /// Parse while expression (as expression form)
    fn parse_while_expr(&mut self) -> Option<Expr> {
        // For now, treat while as statement-only
        // Could be extended to return unit value
        self.parse_while_stmt()?;
        None // This is a statement, not an expression
    }

    /// Parse for expression
    fn parse_for_expr(&mut self) -> Option<Expr> {
        self.parse_for_stmt()?;
        None
    }

    /// Parse loop expression
    fn parse_loop_expr(&mut self) -> Option<Expr> {
        self.expect(Token::Loop)?;
        let body = self.parse_block()?;

        // Loop expression returns never type conceptually
        Some(Expr::Block(body))
    }

    /// Parse async expression
    fn parse_async_expr(&mut self) -> Option<Expr> {
        let _span_start = self.current_span();

        self.expect(Token::Async)?;

        let move_kw = self.match_token(Token::Mut); // Simplified: treating 'mut' as 'move'

        let body = self.parse_block()?;

        Some(Expr::Async(AsyncExpr { body, move_kw }))
    }

    /// Parse array expression
    fn parse_array_expr(&mut self) -> Option<Expr> {
        let _span_start = self.current_span();

        self.expect(Token::LBracket)?;

        let mut elements = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBracket {
            if let Some(expr) = self.parse_expr() {
                elements.push(expr);
            }
            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBracket)?;

        Some(Expr::Array(elements))
    }

    // ========================================================================
    // PATTERN PARSING
    // ========================================================================

    /// Parse pattern
    fn parse_pattern(&mut self) -> Option<Pattern> {
        match self.current_token() {
            Token::Underscore => {
                self.advance();
                Some(Pattern::Wildcard)
            },
            Token::Ident(name) => {
                self.advance();
                let _mutable = false; // Could check for 'mut' prefix
                Some(Pattern::Ident(name, Mutability::Immutable))
            },
            Token::Number(n) => {
                self.advance();
                Some(Pattern::Literal(Literal::Int(n as i64)))
            },
            Token::True => {
                self.advance();
                Some(Pattern::Literal(Literal::Bool(true)))
            },
            Token::False => {
                self.advance();
                Some(Pattern::Literal(Literal::Bool(false)))
            },
            Token::String(s) => {
                self.advance();
                Some(Pattern::Literal(Literal::String(s)))
            },
            Token::Char(c) => {
                self.advance();
                Some(Pattern::Literal(Literal::Char(c)))
            },
            Token::LParen => {
                self.advance();

                if self.match_token(Token::RParen) {
                    return Some(Pattern::Tuple(Vec::new()));
                }

                let mut patterns = Vec::new();
                loop {
                    if let Some(pat) = self.parse_pattern() {
                        patterns.push(pat);
                    }
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RParen)?;
                Some(Pattern::Tuple(patterns))
            },
            Token::Self_ | Token::SelfUpper => {
                // Path pattern (could be enum variant)
                let path = self.parse_path();

                // Check for tuple struct pattern
                if self.match_token(Token::LParen) {
                    let mut patterns = Vec::new();
                    while !self.is_at_end() && self.current_token() != Token::RParen {
                        if let Some(pat) = self.parse_pattern() {
                            patterns.push(pat);
                        }
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;
                    Some(Pattern::TupleStruct(path, patterns))
                } else {
                    Some(Pattern::Path(path))
                }
            },
            _ => {
                self.error("expected pattern");
                None
            },
        }
    }

    // ========================================================================
    // TYPE PARSING
    // ========================================================================

    /// Parse type expression
    fn parse_type(&mut self) -> Option<Type> {
        match self.current_token() {
            Token::Ident(name) => {
                self.advance();
                let path = Path {
                    segments: vec![PathSegment {
                        ident: name,
                        args: None,
                    }],
                };

                // Check for generic arguments
                if self.match_token(Token::Lt) {
                    let mut args = Vec::new();
                    while !self.is_at_end() && self.current_token() != Token::Gt {
                        if let Some(ty) = self.parse_type() {
                            args.push(ty);
                        }
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::Gt)?;
                    return Some(Type::Generic(Box::new(Type::Path(path)), args));
                }

                Some(Type::Path(path))
            },
            Token::LParen => {
                self.advance();

                if self.match_token(Token::RParen) {
                    return Some(Type::Unit);
                }

                let mut types = Vec::new();
                loop {
                    if let Some(ty) = self.parse_type() {
                        types.push(ty);
                    }
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RParen)?;

                if types.len() == 1 {
                    Some(types.into_iter().next().unwrap())
                } else {
                    Some(Type::Tuple(types))
                }
            },
            Token::Ampersand => {
                self.advance();
                let mutable = self.match_token(Token::Mut);
                let ty = self.parse_type()?;
                Some(Type::Reference(
                    Box::new(ty),
                    if mutable {
                        Mutability::Mutable
                    } else {
                        Mutability::Immutable
                    },
                ))
            },
            Token::LBracket => {
                self.advance();
                let ty = self.parse_type()?;

                if self.match_token(Token::Semicolon) {
                    // Array type: [T; N]
                    // For now, we just parse the type and ignore size
                    let _size = self.parse_expr();
                    self.expect(Token::RBracket)?;
                    Some(Type::Array(Box::new(ty), 0)) // Size would need evaluation
                } else {
                    self.expect(Token::RBracket)?;
                    Some(Type::Slice(Box::new(ty)))
                }
            },
            Token::Star => {
                self.advance();
                let mutable = self.match_token(Token::Mut);
                let ty = self.parse_type()?;
                Some(Type::Pointer(
                    Box::new(ty),
                    if mutable {
                        Mutability::Mutable
                    } else {
                        Mutability::Immutable
                    },
                ))
            },
            Token::Fn => {
                self.advance();
                self.expect(Token::LParen)?;

                let mut param_types = Vec::new();
                while !self.is_at_end() && self.current_token() != Token::RParen {
                    if let Some(ty) = self.parse_type() {
                        param_types.push(ty);
                    }
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RParen)?;

                let ret_type = if self.match_token(Token::Arrow) {
                    self.parse_type()?
                } else {
                    Type::Unit
                };

                Some(Type::Fn(param_types, Box::new(ret_type)))
            },
            Token::Dyn => {
                self.advance();
                let mut traits = Vec::new();
                loop {
                    let path = self.parse_path();
                    traits.push(Type::Path(path));

                    if !self.match_token(Token::Plus) {
                        break;
                    }
                }
                Some(Type::TraitObject(traits))
            },
            Token::Underscore => {
                self.advance();
                Some(Type::Inferred)
            },
            _ => {
                self.error("expected type");
                None
            },
        }
    }

    // ========================================================================
    // PATH PARSING
    // ========================================================================

    /// Parse path (e.g., `std::io::Result`)
    fn parse_path(&mut self) -> Path {
        let mut segments = Vec::new();

        loop {
            let ident = match self.current_token() {
                Token::Ident(sym) => {
                    self.advance();
                    sym
                },
                Token::Self_ => {
                    self.advance();
                    Symbol::intern("self")
                },
                Token::SelfUpper => {
                    self.advance();
                    Symbol::intern("Self")
                },
                Token::Super => {
                    self.advance();
                    Symbol::intern("super")
                },
                Token::Crate => {
                    self.advance();
                    Symbol::intern("crate")
                },
                _ => break,
            };

            // Check for generic arguments (turbofish: ::<T>)
            let args =
                if self.current_token() == Token::ColonColon && self.peek_token() == Token::Lt {
                    self.advance(); // consume ::
                    self.advance(); // consume <
                    let mut types = Vec::new();
                    while !self.is_at_end() && self.current_token() != Token::Gt {
                        if let Some(ty) = self.parse_type() {
                            types.push(ty);
                        }
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                    self.expect(Token::Gt);
                    Some(types)
                } else {
                    None
                };

            segments.push(PathSegment { ident, args });

            // Check for path continuation (::)
            if !self.match_token(Token::ColonColon) {
                break;
            }

            // After ::, we expect another identifier. If not, break.
            if !matches!(
                self.current_token(),
                Token::Ident(_) | Token::Self_ | Token::SelfUpper | Token::Super | Token::Crate
            ) {
                break;
            }
        }

        Path { segments }
    }

    /// Parse identifier
    fn parse_ident(&mut self) -> Option<Symbol> {
        let sym = match self.current_token() {
            Token::Ident(s) => {
                self.advance();
                s
            },
            Token::Self_ => {
                self.advance();
                Symbol::intern("self")
            },
            Token::SelfUpper => {
                self.advance();
                Symbol::intern("Self")
            },
            Token::Super => {
                self.advance();
                Symbol::intern("super")
            },
            Token::Crate => {
                self.advance();
                Symbol::intern("crate")
            },
            _ => {
                self.error("expected identifier");
                return None;
            },
        };
        Some(sym)
    }

    // ========================================================================
    // TOKEN NAVIGATION
    // ========================================================================

    /// Get current token
    fn current_token(&self) -> Token {
        self.tokens
            .get(self.position)
            .map(|t| t.token.clone())
            .unwrap_or(Token::Eof)
    }

    /// Get current token with span
    fn current_token_with_span(&self) -> Option<&TokenWithSpan> {
        self.tokens.get(self.position)
    }

    /// Get current span
    fn current_span(&self) -> Span {
        self.tokens
            .get(self.position)
            .map(|t| t.span)
            .unwrap_or(Span::DUMMY)
    }

    /// Create span from start position to current
    fn span_from_start(&self, start: Span) -> Span {
        let current = self.current_span();
        Span {
            start: start.start,
            end: current.end,
            line: start.line,
            column: start.column,
            file_id: start.file_id,
        }
    }

    /// Check if at end of tokens
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    /// Peek at next token
    fn peek_token(&self) -> Token {
        self.tokens
            .get(self.position + 1)
            .map(|t| t.token.clone())
            .unwrap_or(Token::Eof)
    }

    /// Peek at current token without consuming
    fn peek_token_raw(&self) -> Token {
        self.tokens
            .get(self.position)
            .map(|t| t.token.clone())
            .unwrap_or(Token::Eof)
    }

    /// Advance to next token
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    /// Match and consume token
    fn match_token(&mut self, expected: Token) -> bool {
        if self.current_token() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expect specific token
    fn expect(&mut self, expected: Token) -> Option<()> {
        if self.current_token() == expected {
            self.advance();
            Some(())
        } else {
            self.error(format!(
                "expected '{}', found '{}'",
                expected,
                self.current_token()
            ));
            None
        }
    }

    // ========================================================================
    // OPERATOR PRECEDENCE
    // ========================================================================

    /// Get infix operator binding power
    ///
    /// Returns (left_binding_power, right_binding_power)
    /// Higher numbers = tighter binding
    ///
    /// Precedence (low to high):
    /// 1. || (logical or)
    /// 2. && (logical and)
    /// 3. ==, !=, <, <=, >, >= (comparison)
    /// 4. | (bitwise or)
    /// 5. ^ (bitwise xor)
    /// 6. & (bitwise and)
    /// 7. <<, >> (shift)
    /// 8. +, - (arithmetic)
    /// 9. *, /, % (multiplicative)
    /// 10. as (cast - highest precedence)
    fn infix_binding_power(&self) -> Option<(u8, u8)> {
        match self.current_token() {
            // Range (lowest precedence for infix)
            Token::DotDot | Token::DotDotEq => Some((1, 2)),

            // Assignment
            Token::Eq => Some((1, 2)), // Right associative

            // Logical or
            Token::OrOr => Some((3, 4)),

            // Logical and
            Token::AndAnd => Some((5, 6)),

            // Comparison
            Token::EqEq | Token::NotEq => Some((7, 8)),
            Token::Lt | Token::Gt | Token::LtEq | Token::GtEq => Some((9, 10)),

            // Bitwise or
            Token::Pipe => Some((11, 12)),

            // Bitwise xor
            Token::Caret => Some((13, 14)),

            // Bitwise and
            Token::Ampersand => Some((15, 16)),

            // Shift
            Token::Shl | Token::Shr => Some((17, 18)),

            // Arithmetic
            Token::Plus | Token::Minus => Some((19, 20)),

            // Multiplicative
            Token::Star | Token::Slash | Token::Percent => Some((21, 22)),

            // Cast (highest precedence, right associative)
            Token::As => Some((23, 24)),

            _ => None,
        }
    }

    /// Convert token to binary operator
    fn token_to_binop(&self, token: Token) -> Option<BinOp> {
        match token {
            Token::Plus => Some(BinOp::Add),
            Token::Minus => Some(BinOp::Sub),
            Token::Star => Some(BinOp::Mul),
            Token::Slash => Some(BinOp::Div),
            Token::Percent => Some(BinOp::Mod),

            Token::EqEq => Some(BinOp::Eq),
            Token::NotEq => Some(BinOp::Ne),
            Token::Lt => Some(BinOp::Lt),
            Token::Gt => Some(BinOp::Gt),
            Token::LtEq => Some(BinOp::Le),
            Token::GtEq => Some(BinOp::Ge),

            Token::AndAnd => Some(BinOp::And),
            Token::OrOr => Some(BinOp::Or),

            Token::Ampersand => Some(BinOp::BitAnd),
            Token::Pipe => Some(BinOp::BitOr),
            Token::Caret => Some(BinOp::BitXor),

            Token::Shl => Some(BinOp::Shl),
            Token::Shr => Some(BinOp::Shr),

            _ => None,
        }
    }

    // ========================================================================
    // ERROR HANDLING
    // ========================================================================

    /// Check if current position looks like match arms (contains =>)
    /// Used to distinguish match arms from struct fields
    fn looks_like_match_arms(&self) -> bool {
        // Peek ahead up to 10 tokens to check for =>
        let mut depth = 0;
        let mut pos = self.position;
        let max_depth = 10;

        while depth < max_depth && pos < self.tokens.len() {
            let token = &self.tokens[pos];
            match token.token {
                Token::FatArrow => return true,
                Token::LBrace | Token::LParen | Token::LBracket => depth += 1,
                Token::RBrace | Token::RParen | Token::RBracket => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                },
                Token::Comma => {
                    if depth == 0 {
                        return false;
                    }
                },
                _ => {},
            }
            pos += 1;
        }
        false
    }

    /// Check if current position looks like a block (expression context) vs struct fields
    /// Returns true if it looks like a block (starts with expression, not ident:)
    fn looks_like_block(&self) -> bool {
        // Peek at the next non-whitespace token after {
        let mut pos = self.position;

        // Skip LBrace
        if pos < self.tokens.len() && self.tokens[pos].token == Token::LBrace {
            pos += 1;
        }

        // Check first token inside braces
        if pos < self.tokens.len() {
            let token = &self.tokens[pos];
            match token.token {
                // If starts with ident followed by :, it's struct field
                Token::Ident(_) => {
                    // Check if next token is :
                    if pos + 1 < self.tokens.len() {
                        return self.tokens[pos + 1].token != Token::Colon;
                    }
                    return true;
                },
                // If starts with other expression tokens, it's a block
                Token::Number(_)
                | Token::Float(_)
                | Token::String(_)
                | Token::True
                | Token::False
                | Token::Minus
                | Token::Bang
                | Token::Tilde
                | Token::Ampersand
                | Token::Star
                | Token::LParen
                | Token::LBrace
                | Token::If
                | Token::Match
                | Token::While
                | Token::For
                | Token::Loop
                | Token::Return
                | Token::Break
                | Token::Continue => return true,
                _ => {},
            }
        }
        false
    }

    /// Check if current position is at a likely item start
    /// This helps with error recovery by identifying valid item beginnings
    pub fn is_at_item_start(&self) -> bool {
        match self.current_token() {
            Token::Fn
            | Token::Struct
            | Token::Enum
            | Token::Trait
            | Token::Impl
            | Token::Use
            | Token::Mod
            | Token::Const
            | Token::Pub
            | Token::MacroRules => true,
            _ => false,
        }
    }

    /// Check if current position is at a statement start
    pub fn is_at_stmt_start(&self) -> bool {
        match self.current_token() {
            Token::Let
            | Token::If
            | Token::While
            | Token::For
            | Token::Loop
            | Token::Match
            | Token::Return
            | Token::Break
            | Token::Continue
            | Token::LBrace
            | Token::Semicolon => true,
            Token::Ident(_) => {
                // Could be a function call or other expression
                !self.is_at_item_start()
            },
            _ => false,
        }
    }

    /// Report an error
    fn error(&mut self, message: impl Into<String>) {
        let span = self.current_span();
        self.handler.error(message, span);
    }

    /// Report an error with expected token info
    fn error_expected(&mut self, expected: &str) {
        let found = self.current_token().to_string();
        self.error(format!("expected {}, found {}", expected, found));
    }

    /// Report an error with context
    fn error_at(&mut self, message: impl Into<String>, span: Span) {
        self.handler.error(message, span);
    }

    /// Recover to synchronization point
    ///
    /// Skip tokens until we reach a point where parsing can resume.
    /// Sync points include:
    /// - Statement terminators (;)
    /// - Block boundaries ({, })
    /// - Top-level item keywords
    fn recover_to_sync_point(&mut self) {
        loop {
            match self.current_token() {
                Token::Eof => break,
                Token::Fn
                | Token::Struct
                | Token::Enum
                | Token::Trait
                | Token::Impl
                | Token::Use
                | Token::Mod
                | Token::Const
                | Token::Pub => break,
                Token::Semicolon => {
                    self.advance();
                    break;
                },
                _ => {
                    self.advance();
                },
            }
        }
    }

    /// Recover to statement synchronization point
    fn recover_to_stmt_sync(&mut self) {
        loop {
            match self.current_token() {
                Token::Eof | Token::RBrace => break,
                Token::Semicolon => {
                    self.advance();
                    break;
                },
                Token::Let
                | Token::If
                | Token::While
                | Token::For
                | Token::Return
                | Token::Break
                | Token::Continue => break,
                _ => {
                    self.advance();
                },
            }
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use faxc_lex::Lexer;

    /// Helper to parse source and return AST
    fn parse_source(source: &str) -> (Ast, Handler) {
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
        let ast = parser.parse();

        (ast, handler)
    }

    /// Helper to parse a single expression
    fn parse_expr_source(source: &str) -> (Option<Expr>, Handler) {
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
        let expr = parser.parse_expr();

        (expr, handler)
    }

    // ========================================================================
    // EXPRESSION TESTS
    // ========================================================================

    #[test]
    fn test_parse_literal_int() {
        let (expr, handler) = parse_expr_source("42");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Literal(Literal::Int(42)))));
    }

    #[test]
    fn test_parse_literal_float() {
        let (expr, handler) = parse_expr_source("3.14");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Literal(Literal::Float(f))) if (f - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_parse_literal_string() {
        let (expr, handler) = parse_expr_source("\"hello\"");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Literal(Literal::String(s))) if s.as_str() == "hello"));
    }

    #[test]
    fn test_parse_literal_bool() {
        let (expr, handler) = parse_expr_source("true");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Literal(Literal::Bool(true)))));

        let (expr, handler) = parse_expr_source("false");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Literal(Literal::Bool(false)))));
    }

    #[test]
    fn test_parse_variable() {
        let (expr, handler) = parse_expr_source("x");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Path(_))));
    }

    #[test]
    fn test_parse_binary_arithmetic() {
        let (expr, handler) = parse_expr_source("1 + 2");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Binary(b)) if b.op == BinOp::Add));
    }

    #[test]
    fn test_parse_binary_precedence() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let (expr, handler) = parse_expr_source("1 + 2 * 3");
        assert!(!handler.has_errors());

        if let Some(Expr::Binary(b)) = expr {
            assert_eq!(b.op, BinOp::Add);
            // Right side should be multiplication
            assert!(matches!(*b.right, Expr::Binary(ref rb) if rb.op == BinOp::Mul));
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_parse_binary_logical() {
        let (expr, handler) = parse_expr_source("a && b || c");
        assert!(!handler.has_errors());

        // Should parse as (a && b) || c due to precedence
        if let Some(Expr::Binary(b)) = expr {
            assert_eq!(b.op, BinOp::Or);
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_parse_binary_comparison() {
        let (expr, handler) = parse_expr_source("a < b && c > d");
        assert!(!handler.has_errors());

        // Should parse as (a < b) && (c > d)
        if let Some(Expr::Binary(b)) = expr {
            assert_eq!(b.op, BinOp::And);
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_parse_unary_negation() {
        let (expr, handler) = parse_expr_source("-x");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Unary(u)) if u.op == UnOp::Neg));
    }

    #[test]
    fn test_parse_unary_not() {
        let (expr, handler) = parse_expr_source("!flag");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Unary(u)) if u.op == UnOp::Not));
    }

    #[test]
    fn test_parse_unary_bitwise_not() {
        let (expr, handler) = parse_expr_source("~mask");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Unary(u)) if u.op == UnOp::BitNot));
    }

    #[test]
    fn test_parse_parenthesized() {
        let (expr, handler) = parse_expr_source("(1 + 2) * 3");
        assert!(!handler.has_errors());

        if let Some(Expr::Binary(b)) = expr {
            assert_eq!(b.op, BinOp::Mul);
            // Left side should be the parenthesized addition
            assert!(matches!(*b.left, Expr::Binary(ref lb) if lb.op == BinOp::Add));
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_parse_tuple() {
        let (expr, handler) = parse_expr_source("(1, 2, 3)");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Tuple(v)) if v.len() == 3));
    }

    #[test]
    fn test_parse_array() {
        let (expr, handler) = parse_expr_source("[1, 2, 3]");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Array(v)) if v.len() == 3));
    }

    #[test]
    fn test_parse_function_call() {
        let (expr, handler) = parse_expr_source("foo(1, 2)");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Call(c)) if c.args.len() == 2));
    }

    #[test]
    fn test_parse_method_call() {
        let (expr, handler) = parse_expr_source("obj.method(1, 2)");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::MethodCall(m)) if m.call_args.len() == 2));
    }

    #[test]
    fn test_parse_field_access() {
        let (expr, handler) = parse_expr_source("obj.field");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Field(f))));
    }

    #[test]
    fn test_parse_if_expression() {
        let (expr, handler) = parse_expr_source("if x > 0 { x } else { -x }");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::If(_))));
    }

    #[test]
    fn test_parse_match_expression() {
        let source = "match x { 0 => \"zero\", _ => \"other\" }";
        let (expr, handler) = parse_expr_source(source);
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Match(m)) if m.arms.len() == 2));
    }

    #[test]
    fn test_parse_block() {
        let (expr, handler) = parse_expr_source("{ let x = 1; x + 1 }");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Block(b))));
    }

    #[test]
    fn test_parse_closure() {
        let (expr, handler) = parse_expr_source("|x: i32| x + 1");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Closure(c))));
    }

    #[test]
    fn test_parse_closure_fn_syntax() {
        let (expr, handler) = parse_expr_source("fn(x: i32) -> i32 { x + 1 }");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Closure(c))));
    }

    // ========================================================================
    // STATEMENT TESTS
    // ========================================================================

    #[test]
    fn test_parse_let_statement() {
        let (ast, handler) = parse_source("fn foo() { let x = 42; }");
        assert!(!handler.has_errors());
        assert_eq!(ast.len(), 1);
    }

    #[test]
    fn test_parse_let_mutable() {
        let (ast, handler) = parse_source("fn foo() { let mut x = 42; }");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_parse_let_with_type() {
        let (ast, handler) = parse_source("fn foo() { let x: i32 = 42; }");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_parse_return_statement() {
        let (ast, handler) = parse_source("fn foo() { return 42; }");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_parse_if_statement() {
        let (ast, handler) = parse_source("fn foo() { if x > 0 { println(x); } }");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_parse_if_else_statement() {
        let (ast, handler) =
            parse_source("fn foo() { if x > 0 { println(x); } else { println(0); } }");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_parse_while_statement() {
        let (ast, handler) = parse_source("fn foo() { while i < 10 { i = i + 1; } }");
        assert!(!handler.has_errors());
    }

    // ========================================================================
    // ITEM TESTS
    // ========================================================================

    #[test]
    fn test_parse_function() {
        let (ast, handler) = parse_source("fn main() { println(\"Hello\"); }");
        assert!(!handler.has_errors());
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0], Item::Fn(_)));
    }

    #[test]
    fn test_parse_function_with_params() {
        let (ast, handler) = parse_source("fn add(a: i32, b: i32) -> i32 { a + b }");
        assert!(!handler.has_errors());

        if let Item::Fn(fn_item) = &ast[0] {
            assert_eq!(fn_item.params.len(), 2);
            assert!(fn_item.ret_type.is_some());
        } else {
            panic!("Expected function item");
        }
    }

    #[test]
    fn test_parse_function_async() {
        let (ast, handler) = parse_source("async fn fetch() -> str { \"data\" }");
        assert!(!handler.has_errors());

        if let Item::Fn(fn_item) = &ast[0] {
            assert!(fn_item.async_kw);
        } else {
            panic!("Expected function item");
        }
    }

    #[test]
    fn test_parse_struct() {
        let (ast, handler) = parse_source("struct Point { x: f64, y: f64 }");
        assert!(!handler.has_errors());
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0], Item::Struct(_)));

        if let Item::Struct(struct_item) = &ast[0] {
            assert_eq!(struct_item.fields.len(), 2);
        }
    }

    #[test]
    fn test_parse_enum() {
        let (ast, handler) = parse_source("enum Option { Some(i32), None }");
        assert!(!handler.has_errors());
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0], Item::Enum(_)));

        if let Item::Enum(enum_item) = &ast[0] {
            assert_eq!(enum_item.variants.len(), 2);
        }
    }

    #[test]
    fn test_parse_enum_struct_variant() {
        let (ast, handler) = parse_source("enum Result { Ok { value: i32 }, Err { msg: str } }");
        assert!(!handler.has_errors());

        if let Item::Enum(enum_item) = &ast[0] {
            assert_eq!(enum_item.variants.len(), 2);
            assert!(matches!(enum_item.variants[0].data, VariantData::Struct(_)));
        }
    }

    #[test]
    fn test_parse_use_statement() {
        let (ast, handler) = parse_source("use std::io::Read;");
        if handler.has_errors() {
            eprintln!(
                "Errors in test_parse_use_statement: {} errors",
                handler.error_count()
            );
            for diag in handler.diagnostics() {
                eprintln!("  - {} at {:?}", diag.message, diag.span);
            }
        }
        assert!(!handler.has_errors());
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0], Item::Use(_)));
    }

    #[test]
    fn test_parse_use_with_alias() {
        let (ast, handler) = parse_source("use std::io::Read as IoRead;");
        assert!(!handler.has_errors());

        if let Item::Use(use_item) = &ast[0] {
            assert!(use_item.alias.is_some());
        }
    }

    #[test]
    fn test_parse_pub_function() {
        let (ast, handler) = parse_source("pub fn main() { }");
        assert!(!handler.has_errors());

        if let Item::Fn(fn_item) = &ast[0] {
            assert_eq!(fn_item.visibility, Visibility::Public);
        }
    }

    #[test]
    fn test_parse_pub_crate_function() {
        let (ast, handler) = parse_source("pub(crate) fn helper() { }");
        assert!(!handler.has_errors());

        if let Item::Fn(fn_item) = &ast[0] {
            assert_eq!(fn_item.visibility, Visibility::Crate);
        }
    }

    // ========================================================================
    // ERROR RECOVERY TESTS
    // ========================================================================

    #[test]
    fn test_error_recovery_missing_semicolon() {
        let (ast, handler) = parse_source("fn foo() { let x = 1 let y = 2; }");
        // Should recover and continue parsing
        assert!(handler.has_errors());
    }

    #[test]
    fn test_error_recovery_missing_brace() {
        let (ast, handler) = parse_source("fn foo( { }");
        assert!(handler.has_errors());
    }

    #[test]
    fn test_error_recovery_invalid_token() {
        let (ast, handler) = parse_source("fn foo() { @invalid x = 1; }");
        // Should recover and continue
        assert!(handler.has_errors());
    }

    #[test]
    fn test_error_recovery_multiple_errors() {
        let (ast, handler) = parse_source("fn foo( { let x = ; let y = 2; }");
        // Should report multiple errors
        assert!(handler.has_errors());
        assert!(handler.error_count() >= 1);
    }

    // ========================================================================
    // EDGE CASE TESTS
    // ========================================================================

    #[test]
    fn test_empty_source() {
        let (ast, handler) = parse_source("");
        assert!(!handler.has_errors());
        assert!(ast.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let (ast, handler) = parse_source("   \n\t  \n  ");
        assert!(!handler.has_errors());
        assert!(ast.is_empty());
    }

    #[test]
    fn test_complex_expression() {
        let source = "if a > b && c < d || e == f { x + y * z } else { -w }";
        let (expr, handler) = parse_expr_source(source);
        if handler.has_errors() {
            eprintln!(
                "Errors in test_complex_expression: {}",
                handler.error_count()
            );
            for diag in handler.diagnostics() {
                eprintln!("  - {}", diag.message);
            }
        }
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::If(_))));
    }

    #[test]
    fn test_nested_match() {
        let source = "match x { n if n > 0 => match y { 0 => \"zero\", _ => \"other\" }, _ => \"negative\" }";
        let (expr, handler) = parse_expr_source(source);
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Match(_))));
    }

    #[test]
    fn test_chained_method_calls() {
        let (expr, handler) = parse_expr_source("obj.method1().method2().method3()");
        assert!(!handler.has_errors());
    }

    #[test]
    fn test_generic_function_call() {
        let (expr, handler) = parse_expr_source("foo::<i32, str>(1, \"hello\")");
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Call(_))));
    }

    #[test]
    fn test_reference_and_deref() {
        let (expr, handler) = parse_expr_source("&mut *ptr");
        if handler.has_errors() {
            eprintln!("Errors: {}", handler.error_count());
            for diag in handler.diagnostics() {
                eprintln!("  - {}", diag.message);
            }
        }
        assert!(!handler.has_errors());
        assert!(matches!(expr, Some(Expr::Unary(u)) if matches!(u.op, UnOp::Ref(true))));
    }
}
