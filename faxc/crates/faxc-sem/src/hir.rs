use crate::types::*;
use faxc_util::{Idx, Symbol, DefId};

/// HIR Item
#[derive(Debug, Clone)]
pub enum Item {
    Function(FnItem),
    Struct(StructItem),
    Enum(EnumItem),
    Trait(TraitItem),
    Impl(ImplItem),
}

/// HIR Function
#[derive(Debug, Clone)]
pub struct FnItem {
    pub def_id: DefId,
    pub name: Symbol,
    pub generics: GenericParams,
    pub params: Vec<Param>,
    pub ret_type: Type,
    pub body: Body,
    pub async_kw: bool,
}

/// Generic parameters
#[derive(Debug, Clone, Default)]
pub struct GenericParams {
    pub params: Vec<GenericParam>,
    pub where_clause: Vec<WherePredicate>,
}

/// Generic parameter
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub id: ParamId,
    pub name: Symbol,
    pub kind: GenericParamKind,
}

/// Kind of generic parameter
#[derive(Debug, Clone)]
pub enum GenericParamKind {
    Type { bounds: Vec<Type> },
    Lifetime,
    Const { ty: Type },
}

/// Where clause predicate
#[derive(Debug, Clone)]
pub struct WherePredicate {
    pub ty: Type,
    pub bounds: Vec<Type>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub pat: Pattern,
    pub ty: Type,
}

/// Function body
#[derive(Debug, Clone)]
pub struct Body {
    pub params: Vec<Pattern>,
    pub value: Expr,
}

/// HIR Struct
#[derive(Debug, Clone)]
pub struct StructItem {
    pub def_id: DefId,
    pub name: Symbol,
    pub generics: GenericParams,
    pub fields: Vec<FieldDef>,
}

/// Field definition
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: Symbol,
    pub ty: Type,
}

/// HIR Enum
#[derive(Debug, Clone)]
pub struct EnumItem {
    pub def_id: DefId,
    pub name: Symbol,
    pub generics: GenericParams,
    pub variants: Vec<VariantDef>,
}

/// Variant definition
#[derive(Debug, Clone)]
pub struct VariantDef {
    pub def_id: DefId,
    pub name: Symbol,
    pub data: VariantData,
}

/// Variant data
#[derive(Debug, Clone)]
pub enum VariantData {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<FieldDef>),
}

/// HIR Trait
#[derive(Debug, Clone)]
pub struct TraitItem {
    pub def_id: DefId,
    pub name: Symbol,
    pub generics: GenericParams,
    pub items: Vec<TraitItemKind>,
}

/// Trait item kind
#[derive(Debug, Clone)]
pub enum TraitItemKind {
    Method(FnSig),
    Type(Symbol, Vec<Type>),
    Const(Symbol, Type, Option<Expr>),
}

/// HIR Impl
#[derive(Debug, Clone)]
pub struct ImplItem {
    pub impl_id: DefId,
    pub generics: GenericParams,
    pub trait_ref: Option<TraitRef>,
    pub self_ty: Type,
    pub items: Vec<ImplItemKind>,
}

/// Trait reference
#[derive(Debug, Clone)]
pub struct TraitRef {
    pub def_id: DefId,
    pub args: Vec<Type>,
}

/// Impl item kind
#[derive(Debug, Clone)]
pub enum ImplItemKind {
    Method(FnItem),
    Type(Symbol, Type),
    Const(Symbol, Type, Expr),
}

/// Function signature (without body)
#[derive(Debug, Clone)]
pub struct FnSig {
    pub def_id: DefId,
    pub name: Symbol,
    pub generics: GenericParams,
    pub params: Vec<Param>,
    pub ret_type: Type,
}

/// HIR Expression
#[derive(Debug, Clone)]
pub enum Expr {
    Literal { lit: Literal, ty: Type },
    Var { def_id: DefId, ty: Type },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        ty: Type,
    },
    Unary { op: UnOp, expr: Box<Expr>, ty: Type },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        ty: Type,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: DefId,
        args: Vec<Expr>,
        ty: Type,
    },
    Field {
        object: Box<Expr>,
        field: DefId,
        ty: Type,
    },
    Block {
        stmts: Vec<Stmt>,
        expr: Option<Box<Expr>>,
        ty: Type,
    },
    If {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Option<Box<Expr>>,
        ty: Type,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<Arm>,
        ty: Type,
    },
    Assign { place: Box<Expr>, value: Box<Expr> },
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>, Option<LabelId>),
    Continue(Option<LabelId>),
    Async {
        body: Box<Expr>,
        ty: Type,
    },
    Await {
        expr: Box<Expr>,
        ty: Type,
    },
}

impl Expr {
    pub fn ty(&self) -> Type {
        match self {
            Expr::Literal { ty, .. } => ty.clone(),
            Expr::Var { ty, .. } => ty.clone(),
            Expr::Binary { ty, .. } => ty.clone(),
            Expr::Unary { ty, .. } => ty.clone(),
            Expr::Call { ty, .. } => ty.clone(),
            Expr::MethodCall { ty, .. } => ty.clone(),
            Expr::Field { ty, .. } => ty.clone(),
            Expr::Block { ty, .. } => ty.clone(),
            Expr::If { ty, .. } => ty.clone(),
            Expr::Match { ty, .. } => ty.clone(),
            Expr::Assign { .. } => Type::Unit,
            Expr::Return(_) => Type::Never,
            Expr::Break(_, _) => Type::Never,
            Expr::Continue(_) => Type::Never,
            Expr::Async { ty, .. } => ty.clone(),
            Expr::Await { ty, .. } => ty.clone(),
        }
    }
}

/// Literal
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(Symbol),
    Bool(bool),
    Char(char),
    Unit,
}

/// Binary operator
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
}

/// Unary operator
#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg, Not, Deref, Ref(bool),
}

/// Statement
#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        pat: Pattern,
        ty: Type,
        init: Option<Expr>,
    },
    Expr(Expr),
}

/// Pattern
#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Binding {
        name: Symbol,
        ty: Type,
        mutability: bool,
    },
    Path {
        def_id: DefId,
    },
    Struct {
        def_id: DefId,
        fields: Vec<FieldPattern>,
    },
    Tuple {
        pats: Vec<Pattern>,
    },
    Ref {
        pat: Box<Pattern>,
        mutability: bool,
    },
    Or(Vec<Pattern>),
}

/// Field in pattern
#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub field: DefId,
    pub pat: Pattern,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct Arm {
    pub pat: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// Label ID for loops
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelId(pub u32);
