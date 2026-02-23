//! faxc-par - AST Node Definitions
//!
//! This module contains all AST node definitions used by the parser.

use faxc_util::{Span, Symbol};

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
    pub name: Symbol,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub ret_type: Option<Type>,
    pub body: Block,
    pub visibility: Visibility,
    pub span: Span,
    pub async_kw: bool,
    pub where_clause: Option<WhereClause>,
}

/// Generic parameter
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: Symbol,
    pub bounds: Vec<Type>,
}

/// Where clause constraint
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub bounds: Vec<WhereBound>,
}

/// A single where bound (e.g., `T: Trait1 + Trait2`)
#[derive(Debug, Clone)]
pub struct WhereBound {
    pub ty: Type,
    pub traits: Vec<Path>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: Symbol,
    pub ty: Type,
    pub mutable: bool,
}

/// Structure item
#[derive(Debug, Clone)]
pub struct StructItem {
    pub name: Symbol,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
    pub visibility: Visibility,
    pub span: Span,
    pub where_clause: Option<WhereClause>,
}

/// Field definition
#[derive(Debug, Clone)]
pub struct Field {
    pub name: Symbol,
    pub ty: Type,
    pub visibility: Visibility,
}

/// Enum item
#[derive(Debug, Clone)]
pub struct EnumItem {
    pub name: Symbol,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<Variant>,
    pub visibility: Visibility,
    pub span: Span,
    pub where_clause: Option<WhereClause>,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct Variant {
    pub name: Symbol,
    pub data: VariantData,
}

/// Variant data types
#[derive(Debug, Clone)]
pub enum VariantData {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<Field>),
}

/// Trait item
#[derive(Debug, Clone)]
pub struct TraitItem {
    pub name: Symbol,
    pub generics: Vec<GenericParam>,
    pub items: Vec<TraitMember>,
    pub supertraits: Vec<Type>,
    pub visibility: Visibility,
}

/// Trait member
#[derive(Debug, Clone)]
pub enum TraitMember {
    Method(FnSig),
    Type(Symbol, Vec<Type>),
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
    pub generics: Vec<GenericParam>,
    pub trait_ref: Option<Type>,
    pub self_ty: Type,
    pub items: Vec<ImplMember>,
    pub where_clause: Option<WhereClause>,
}

/// Implementation member
#[derive(Debug, Clone)]
pub enum ImplMember {
    Method(FnItem),
    Type(Symbol, Type),
    Const(Symbol, Type, Expr),
}

/// Use/import item
#[derive(Debug, Clone)]
pub struct UseItem {
    pub path: Path,
    pub alias: Option<Symbol>,
    pub is_glob: bool,
}

/// Constant item
#[derive(Debug, Clone)]
pub struct ConstItem {
    pub name: Symbol,
    pub ty: Type,
    pub value: Expr,
    pub visibility: Visibility,
    pub span: Span,
}

/// Static item
#[derive(Debug, Clone)]
pub struct StaticItem {
    pub name: Symbol,
    pub ty: Type,
    pub value: Expr,
    pub mutable: bool,
    pub visibility: Visibility,
    pub span: Span,
}

/// Visibility modifier
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Crate,
    Super,
    Restricted(Path),
}


/// Statement
#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Expr(Expr),
    Return(Option<Expr>),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Break(Option<Symbol>),
    Continue(Option<Symbol>),
    Item(Item),
}

/// Let statement
#[derive(Debug, Clone)]
pub struct LetStmt {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
    pub mutable: bool,
}

/// If statement
#[derive(Debug, Clone)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_clause: Option<Box<ElseClause>>,
}

/// Else clause
#[derive(Debug, Clone)]
pub enum ElseClause {
    Block(Block),
    If(IfStmt),
}

/// While loop
#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub cond: Expr,
    pub body: Block,
    pub label: Option<Symbol>,
}

/// For loop
#[derive(Debug, Clone)]
pub struct ForStmt {
    pub pattern: Pattern,
    pub iter: Expr,
    pub body: Block,
    pub label: Option<Symbol>,
}

/// Block expression
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub trailing: Option<Box<Expr>>,
    pub span: Span,
}


/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Path(Path),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    MethodCall(MethodCallExpr),
    Field(FieldExpr),
    Index(IndexExpr),
    Block(Block),
    If(IfExpr),
    Match(MatchExpr),
    Closure(ClosureExpr),
    Assign(AssignExpr),
    CompoundAssign(CompoundAssignExpr),
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>, Option<Symbol>),
    Continue(Option<Symbol>),
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    Range(RangeExpr),
    Cast(Box<Expr>, Type),
    Async(AsyncExpr),
    Await(Box<Expr>),
    StructLiteral(Box<StructLiteralExpr>),
    EnumVariant(Box<EnumVariantExpr>),
}

impl Expr {
    pub fn span(&self) -> Option<Span> {
        match self {
            Expr::Binary(b) => Some(b.span),
            Expr::Unary(u) => Some(u.span),
            Expr::Call(c) => Some(c.span),
            Expr::Field(f) => Some(f.span),
            Expr::Block(b) => Some(b.span),
            Expr::Literal(_)
            | Expr::Path(_)
            | Expr::MethodCall(_)
            | Expr::Index(_)
            | Expr::If(_)
            | Expr::Match(_)
            | Expr::Closure(_)
            | Expr::Assign(_)
            | Expr::CompoundAssign(_)
            | Expr::Return(_)
            | Expr::Break(_, _)
            | Expr::Continue(_)
            | Expr::Tuple(_)
            | Expr::Array(_)
            | Expr::Range(_)
            | Expr::Cast(_, _)
            | Expr::Async(_)
            | Expr::Await(_)
            | Expr::StructLiteral(_)
            | Expr::EnumVariant(_) => None,
        }
    }
}

/// Literal expression
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(Symbol),
    Char(char),
    Bool(bool),
    Unit,
}

/// Path expression
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

/// Path segment
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    pub ident: Symbol,
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
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
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
    Neg,
    Not,
    BitNot,
    Deref,
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
#[derive(Debug, Clone)]
pub struct StructLiteralExpr {
    pub path: Path,
    pub generics: Option<Vec<Type>>,
    pub fields: Vec<StructField>,
    pub base: Option<Expr>,
}

/// Field in a struct literal
#[derive(Debug, Clone)]
pub struct StructField {
    pub name: Symbol,
    pub expr: Expr,
    pub is_shorthand: bool,
}

/// Enum variant construction expression
#[derive(Debug, Clone)]
pub struct EnumVariantExpr {
    pub path: Path,
    pub variant: Symbol,
    pub generics: Option<Vec<Type>>,
    pub data: EnumVariantData,
}

/// Enum variant data types
#[derive(Debug, Clone)]
pub enum EnumVariantData {
    Unit,
    Tuple(Vec<Expr>),
    Struct(Vec<StructField>),
}


/// Pattern
#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Ident(Symbol, Mutability),
    Literal(Literal),
    Path(Path),
    Struct(Path, Vec<FieldPattern>),
    TupleStruct(Path, Vec<Pattern>),
    Tuple(Vec<Pattern>),
    Slice(Vec<Pattern>),
}

/// Field in struct pattern
#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub field: Symbol,
    pub pattern: Pattern,
}


/// Type expression
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unit,
    Never,
    Path(Path),
    Generic(Box<Type>, Vec<Type>),
    Reference(Box<Type>, Mutability),
    Pointer(Box<Type>, Mutability),
    Slice(Box<Type>),
    Array(Box<Type>, usize),
    Tuple(Vec<Type>),
    Fn(Vec<Type>, Box<Type>),
    TraitObject(Vec<Type>),
    ImplTrait(Vec<Type>),
    Inferred,
}

/// Mutability
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mutability {
    Mutable,
    Immutable,
}

/// Token with span wrapper
#[derive(Debug, Clone)]
pub struct TokenWithSpan {
    pub token: faxc_lex::Token,
    pub span: Span,
}

impl TokenWithSpan {
    pub fn new(token: faxc_lex::Token, span: Span) -> Self {
        Self { token, span }
    }
}
