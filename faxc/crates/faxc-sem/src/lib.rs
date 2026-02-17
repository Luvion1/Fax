//! faxc-sem - Semantic Analyzer & Type Checker
//!
//! ============================================================================
//! SEMANTIC ANALYSIS THEORY
//! ============================================================================
//!
//! Semantic analysis is the phase where we check the "meaning" of the program,
//! not just its syntax. It answers questions like:
//! - Is this variable defined?
//! - Are these types compatible?
//! - Is this code reachable?
//! - Are ownership rules followed?
//!
//! PHASES OF SEMANTIC ANALYSIS:
//! ----------------------------
//!
//! 1. NAME RESOLUTION
//!    - Match identifiers to their definitions
//!    - Build scope tree
//!    - Check for undefined names
//!
//! 2. TYPE CHECKING
//!    - Infer types of expressions
//!    - Check type compatibility
//!    - Validate generic instantiations
//!
//! 3. BORROW CHECKING
//!    - Verify ownership rules
//!    - Check lifetimes
//!    - Ensure memory safety
//!
//! 4. CONTROL FLOW ANALYSIS
//!    - Check all paths return in non-void functions
//!    - Detect unreachable code
//!    - Verify break/continue are in loops
//!
//! 5. MISCELLANEOUS CHECKS
//!    - Visibility rules
//!    - Trait implementations
//!    - Pattern exhaustiveness
//!
//! ============================================================================
//! NAME RESOLUTION
//! ============================================================================
//!
//! PROBLEM:
//! Given an identifier, find its definition.
//!
//! Example:
//! ```
//! fn main() {
//!     let x = 5;
//!     println(x);  // x refers to the let binding above
//! }
//! ```
//!
//! SCOPE:
//! A scope is a region of code where a name is valid.
//!
//! Scope Types:
//! - Module scope (top-level)
//! - Function scope
//! - Block scope
//! - Loop scope (for loop variables)
//! - Pattern scope (match arm bindings)
//!
//! LEXICAL SCOPING:
//! Names are resolved by looking outward through nested scopes.
//!
//! ```
//! let x = 1;           // Scope A
//! {
//!     let y = 2;       // Scope B (inside A)
//!     {
//!         let z = 3;   // Scope C (inside B)
//!         x + y + z    // Can see x (A), y (B), z (C)
//!     }
//!     x + y + z        // ERROR: z not in scope
//! }
//! ```
//!
//! SHADOWING:
//! Inner scope can declare same name as outer scope.
//!
//! ```
//! let x = 1;
//! {
//!     let x = 2;   // Shadows outer x
//!     println(x);  // Prints 2
//! }
//! println(x);      // Prints 1
//! ```
//!
//! RESOLUTION ALGORITHM:
//! --------------------
//!
//! ```
//! resolve(name, current_scope):
//!   for scope in current_scope.chain():
//!       if scope.contains(name):
//!           return scope.get(name)
//!   return ERROR("undefined name")
//! ```
//!
//! RIB STRUCTURE:
//! --------------
//! A "rib" is a data structure representing one scope level.
//!
//! ```
//! Rib {
//!   bindings: Map<Name, DefId>,
//!   parent: Option<Rib>,
//!   kind: RibKind,  // Normal, Fn, Module, etc.
//! }
//! ```
//!
//! Scope Chain:
//! ```
//! Rib(module) -> Rib(fn main) -> Rib(block) -> Rib(if block)
//! ```
//!
//! ============================================================================
//! TYPE SYSTEM
//! ============================================================================
//!
//! TYPE HIERARCHY:
//! --------------
//!
//! Primitive Types:
//! - int, float, bool, string, unit ()
//!
//! Composite Types:
//! - Tuple: (T1, T2, T3)
//! - Array: [T; N]
//! - Slice: [T]
//! - Function: fn(A, B) -> C
//!
//! User-Defined Types:
//! - Structs
//! - Enums
//! - Traits
//! - Type aliases
//!
//! Reference Types:
//! - Shared reference: &T
//! - Mutable reference: &mut T
//! - Raw pointer: *const T, *mut T
//!
//! TYPE EQUALITY:
//! --------------
//!
//! Structural Equality:
//! Types are equal if they have same structure.
//! (int, bool) == (int, bool)  // true
//!
//! Nominal Equality:
//! Types are equal only if they are the same named type.
//! struct Point1 { x: int, y: int }
//! struct Point2 { x: int, y: int }
//! Point1 != Point2  // Different types!
//!
//! TYPE COMPATIBILITY:
//! -------------------
//!
//! Subtyping (<:):
//! T1 <: T2 means T1 is a subtype of T2 (can be used where T2 expected).
//!
//! Examples:
//! - &mut T <: &T  (mutable reference is subtype of immutable)
//! - ! <: T  (never type is subtype of all types)
//!
//! Coercion:
//! Automatic type conversion.
//!
//! Examples:
//! - &T can coerce to &U if T: Deref<Target=U>
//! - &mut T can coerce to &U
//! - T can coerce to U if T implements trait U (unsized coercion)
//!
//! ============================================================================
//! TYPE INFERENCE (HINDLEY-MILNER)
//! ============================================================================
//!
//! Basic Idea:
//! 1. Assign type variable to every expression
//! 2. Generate constraints from AST
//! 3. Solve constraints via unification
//!
//! EXAMPLE:
//! ```
//! let x = 5;      // x: ?T1
//! let y = x + 3;  // y: ?T2
//! ```
//!
//! Constraints:
//! - From `5`: ?T1 = int
//! - From `x + 3`: ?T1 = int, ?T2 = int
//!
//! UNIFICATION:
//! ------------
//! Solve constraint T1 = T2:
//!
//! ```
//! unify(T1, T2):
//!   if T1 == T2: return Ok
//!   
//!   if T1 is variable:
//!       bind T1 := T2
//!       return Ok
//!       
//!   if T2 is variable:
//!       bind T2 := T1
//!       return Ok
//!       
//!   if T1 and T2 are concrete:
//!       if same constructor:
//!           unify each component
//!       else:
//!           return TypeError
//! ```
//!
//! TYPE VARIABLE:
//! --------------
//! Represents unknown type during inference.
//!
//! ```
//! enum Type {
//!   Concrete(ConcreteType),
//!   Variable(TypeVarId),
//! }
//! ```
//!
//! Type variables have a substitution (mapping to actual types).
//!
//! ============================================================================
//! HIR (HIGH-LEVEL IR)
//! ============================================================================
//!
//! HIR is AST after name resolution and type checking.
//!
//! Differences from AST:
//! - All identifiers -> DefId (resolved)
//! - All expressions -> Typed
//! - All types -> Resolved
//! - All scopes -> Explicit
//!
//! HIR enables:
//! - Accurate analysis
//! - Optimization
//! - Code generation
//!
//! ============================================================================
//! ERROR REPORTING
//! ============================================================================
//!
//! Good semantic errors should:
//! 1. Explain what went wrong
//! 2. Show where it went wrong
//! 3. Suggest how to fix it
//! 4. Avoid cascading errors
//!
//! Example:
//! ```
//! error[E0308]: mismatched types
//!   --> main.fax:3:14
//!    |
//!  3 |     let x: int = "hello";
//!    |              ^^^   ^^^^^^^ expected `int`, found `string`
//!    |
//! help: you might have meant to use a number
//!    |
//!  3 |     let x: int = 42;
//!    |              ^^^^^
//! ```

use faxc_par::ast;
use faxc_util::{Handler, Idx, IndexVec, Span, Symbol};
use std::collections::HashMap;

// ============================================================================
// TYPE REPRESENTATION
// ============================================================================

/// A type in the type system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Error type (for recovery)
    Error,

    /// Unit type ()
    Unit,

    /// Never type (!) - diverges
    Never,

    /// Primitive integer type
    Int,

    /// Primitive float type
    Float,

    /// Boolean type
    Bool,

    /// String type (GC-managed)
    String,

    /// Named type (struct, enum, etc.)
    ///
    /// Resolved to a definition ID
    Adt(DefId),

    /// Type parameter
    ///
    /// Example: The T in `fn foo<T>(x: T)`
    Param(ParamId),

    /// Reference type
    ///
    /// bool indicates mutability (true = &mut, false = &)
    Ref(Box<Type>, bool),

    /// Tuple type
    Tuple(Vec<Type>),

    /// Array type [T; N]
    Array(Box<Type>, usize),

    /// Slice type [T]
    Slice(Box<Type>),

    /// Function type fn(A, B) -> C
    Fn(Vec<Type>, Box<Type>),

    /// Future type - represents an async computation
    ///
    /// Future<T> is the type of an async block or async function return
    Future(Box<Type>),

    /// Type variable (for inference)
    Infer(InferId),
}

/// Definition ID - unique identifier for definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(pub u32);

impl Idx for DefId {
    fn from_usize(idx: usize) -> Self {
        DefId(idx as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Type parameter ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParamId(pub u32);

impl Idx for ParamId {
    fn from_usize(idx: usize) -> Self {
        ParamId(idx as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Type inference variable ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InferId(pub u32);

impl Idx for InferId {
    fn from_usize(idx: usize) -> Self {
        InferId(idx as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

// ============================================================================
// HIR DEFINITIONS
// ============================================================================

/// HIR Item (fully resolved item)
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

// ============================================================================
// HIR EXPRESSIONS
// ============================================================================

/// HIR Expression (typed and resolved)
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal expression
    Literal { lit: Literal, ty: Type },

    /// Variable reference
    Var { def_id: DefId, ty: Type },

    /// Binary operation
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        ty: Type,
    },

    /// Unary operation
    Unary { op: UnOp, expr: Box<Expr>, ty: Type },

    /// Function call
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        ty: Type,
    },

    /// Method call
    MethodCall {
        receiver: Box<Expr>,
        method: DefId,
        args: Vec<Expr>,
        ty: Type,
    },

    /// Field access
    Field {
        object: Box<Expr>,
        field: DefId,
        ty: Type,
    },

    /// Block expression
    Block {
        stmts: Vec<Stmt>,
        expr: Option<Box<Expr>>,
        ty: Type,
    },

    /// If expression
    If {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Option<Box<Expr>>,
        ty: Type,
    },

    /// Match expression
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<Arm>,
        ty: Type,
    },

    /// Assignment
    Assign { place: Box<Expr>, value: Box<Expr> },

    /// Return expression
    Return(Option<Box<Expr>>),

    /// Break expression
    Break(Option<Box<Expr>>, Option<LabelId>),

    /// Continue expression
    Continue(Option<LabelId>),

    /// Async block expression
    Async {
        body: Box<Expr>,
        ty: Type, // Future<inner_type>
    },

    /// Await expression
    Await {
        expr: Box<Expr>,
        ty: Type, // The type the future resolves to
    },
}

/// Literal
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(Symbol),
    Bool(bool),
    Unit,
}

/// Binary operator
#[derive(Debug, Clone, Copy)]
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
}

/// Unary operator
#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
    Deref,
    Ref(bool),
}

/// Statement
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Let binding
    Let {
        pat: Pattern,
        ty: Type,
        init: Option<Expr>,
    },

    /// Expression statement
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
#[derive(Debug, Clone, Copy)]
pub struct LabelId(pub u32);

// ============================================================================
// TYPE CONTEXT
// ============================================================================

/// Type context - stores all type information
#[derive(Debug, Default)]
pub struct TypeContext {
    /// Type of each definition
    def_types: HashMap<DefId, Type>,

    /// Type of each expression
    expr_types: HashMap<ExprId, Type>,

    /// Inference variable substitutions
    substitutions: IndexVec<InferId, Option<Type>>,

    /// Constraints to solve
    constraints: Vec<Constraint>,
}

/// Expression ID
#[derive(Debug, Clone, Copy)]
pub struct ExprId(pub u32);

/// Type constraint
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Equality: T1 = T2
    Eq(Type, Type),

    /// Trait bound: T: Trait
    Trait(Type, DefId),
}

impl TypeContext {
    /// Get type of a definition
    pub fn type_of_def(&self, def_id: DefId) -> Option<&Type> {
        self.def_types.get(&def_id)
    }

    /// Set type of a definition
    pub fn set_def_type(&mut self, def_id: DefId, ty: Type) {
        self.def_types.insert(def_id, ty);
    }

    /// Add equality constraint
    pub fn add_eq_constraint(&mut self, t1: Type, t2: Type) {
        self.constraints.push(Constraint::Eq(t1, t2));
    }

    /// Create new inference variable
    pub fn new_infer_var(&mut self) -> InferId {
        self.substitutions.push(None)
    }

    /// Substitute type variables with their solutions
    pub fn substitute(&self, ty: &Type) -> Type {
        match ty {
            Type::Infer(id) => match self.substitutions.get(*id) {
                Some(Some(t)) => self.substitute(t),
                _ => ty.clone(),
            },
            Type::Tuple(tys) => Type::Tuple(tys.iter().map(|t| self.substitute(t)).collect()),
            Type::Ref(t, m) => Type::Ref(Box::new(self.substitute(t)), *m),
            Type::Array(t, n) => Type::Array(Box::new(self.substitute(t)), *n),
            Type::Fn(params, ret) => Type::Fn(
                params.iter().map(|p| self.substitute(p)).collect(),
                Box::new(self.substitute(ret)),
            ),
            _ => ty.clone(),
        }
    }
}

// ============================================================================
// SCOPE & RESOLUTION
// ============================================================================

/// Scope tree for name resolution
#[derive(Debug)]
pub struct ScopeTree {
    /// All ribs (scopes)
    ribs: IndexVec<RibId, Rib>,

    /// Current rib stack
    current_rib: RibId,
}

/// Rib ID
#[derive(Debug, Clone, Copy)]
pub struct RibId(pub u32);

impl Idx for RibId {
    fn from_usize(idx: usize) -> Self {
        RibId(idx as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// A single scope (rib)
#[derive(Debug)]
pub struct Rib {
    /// Bindings in this scope
    bindings: HashMap<Symbol, DefId>,

    /// Parent rib
    parent: Option<RibId>,

    /// Kind of rib
    kind: RibKind,
}

/// Kind of rib
#[derive(Debug, Clone, Copy)]
pub enum RibKind {
    Module,
    Function,
    Block,
    Loop(Option<LabelId>),
}

impl ScopeTree {
    /// Create new scope tree
    pub fn new() -> Self {
        let mut ribs = IndexVec::new();
        let root = ribs.push(Rib {
            bindings: HashMap::new(),
            parent: None,
            kind: RibKind::Module,
        });

        Self {
            ribs,
            current_rib: root,
        }
    }

    /// Enter new scope
    pub fn enter_scope(&mut self, kind: RibKind) -> RibId {
        let new_rib = self.ribs.push(Rib {
            bindings: HashMap::new(),
            parent: Some(self.current_rib),
            kind,
        });
        self.current_rib = new_rib;
        new_rib
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.ribs[self.current_rib].parent {
            self.current_rib = parent;
        }
    }

    /// Add binding to current scope
    pub fn add_binding(&mut self, name: Symbol, def_id: DefId) {
        self.ribs[self.current_rib].bindings.insert(name, def_id);
    }

    /// Resolve name to definition
    pub fn resolve(&self, name: Symbol) -> Option<DefId> {
        let mut rib_id = self.current_rib;

        loop {
            let rib = &self.ribs[rib_id];

            if let Some(&def_id) = rib.bindings.get(&name) {
                return Some(def_id);
            }

            match rib.parent {
                Some(parent) => rib_id = parent,
                None => return None,
            }
        }
    }
}

// ============================================================================
// SEMANTIC ANALYZER
// ============================================================================

/// Main semantic analyzer
pub struct SemanticAnalyzer<'a> {
    /// Type context
    type_context: &'a mut TypeContext,

    /// Scope tree
    scope_tree: ScopeTree,

    /// Current function return type (for return checking)
    current_ret_type: Option<Type>,

    /// Loop stack (for break/continue checking)
    loop_stack: Vec<(Option<LabelId>, Type)>,

    /// Error handler
    handler: &'a mut Handler,
}

impl<'a> SemanticAnalyzer<'a> {
    /// Create new analyzer
    pub fn new(type_context: &'a mut TypeContext, handler: &'a mut Handler) -> Self {
        Self {
            type_context,
            scope_tree: ScopeTree::new(),
            current_ret_type: None,
            loop_stack: Vec::new(),
            handler,
        }
    }

    /// Analyze AST items and produce HIR
    pub fn analyze_items(&mut self, items: Vec<ast::Item>) -> Vec<Item> {
        // First pass: collect all item names
        self.collect_items(&items);

        // Second pass: resolve and type check
        items
            .into_iter()
            .filter_map(|item| self.analyze_item(item))
            .collect()
    }

    /// Collect item names (first pass)
    fn collect_items(&mut self, items: &[ast::Item]) {
        // Add all items to module scope
        // This enables forward references
        unimplemented!("Item collection not implemented")
    }

    /// Analyze single item
    fn analyze_item(&mut self, item: ast::Item) -> Option<Item> {
        match item {
            ast::Item::Fn(fn_item) => self.analyze_fn_item(fn_item).map(Item::Function),
            ast::Item::Struct(struct_item) => {
                self.analyze_struct_item(struct_item).map(Item::Struct)
            }
            ast::Item::Enum(enum_item) => self.analyze_enum_item(enum_item).map(Item::Enum),
            ast::Item::Impl(impl_item) => self.analyze_impl_item(impl_item).map(Item::Impl),
            ast::Item::Trait(trait_item) => self.analyze_trait_item(trait_item).map(Item::Trait),
            _ => unimplemented!(),
        }
    }

    /// Analyze function item
    fn analyze_fn_item(&mut self, item: ast::FnItem) -> Option<FnItem> {
        // 1. Resolve function name
        // 2. Create function scope
        // 3. Add generic parameters to scope
        // 4. Resolve parameter types
        // 5. Add parameters to scope
        // 6. Resolve return type
        // 7. Analyze function body
        // 8. Check return type matches
        unimplemented!("Function analysis not implemented")
    }

    /// Analyze struct item
    fn analyze_struct_item(&mut self, item: ast::StructItem) -> Option<StructItem> {
        unimplemented!("Struct analysis not implemented")
    }

    /// Analyze enum item
    fn analyze_enum_item(&mut self, item: ast::EnumItem) -> Option<EnumItem> {
        unimplemented!("Enum analysis not implemented")
    }

    /// Analyze impl item
    fn analyze_impl_item(&mut self, item: ast::ImplItem) -> Option<ImplItem> {
        unimplemented!("Impl analysis not implemented")
    }

    /// Analyze trait item
    fn analyze_trait_item(&mut self, item: ast::TraitItem) -> Option<TraitItem> {
        unimplemented!("Trait analysis not implemented")
    }

    /// Analyze expression
    fn analyze_expr(&mut self, expr: ast::Expr) -> Option<Expr> {
        match expr {
            ast::Expr::Literal(lit) => self.analyze_literal(lit),
            ast::Expr::Path(path) => self.analyze_path(path),
            ast::Expr::Binary(bin) => self.analyze_binary(bin),
            ast::Expr::Unary(un) => self.analyze_unary(un),
            ast::Expr::Call(call) => self.analyze_call(call),
            ast::Expr::Block(block) => self.analyze_block(block),
            ast::Expr::If(if_expr) => self.analyze_if(if_expr),
            ast::Expr::Return(ret) => self.analyze_return(ret),
            _ => unimplemented!(),
        }
    }

    /// Analyze literal
    fn analyze_literal(&mut self, lit: ast::Literal) -> Option<Expr> {
        let (lit_kind, ty) = match lit {
            ast::Literal::Int(n) => (Literal::Int(n), Type::Int),
            ast::Literal::Float(f) => (Literal::Float(f), Type::Float),
            ast::Literal::String(s) => (Literal::String(s), Type::String),
            ast::Literal::Bool(b) => (Literal::Bool(b), Type::Bool),
        };

        Some(Expr::Literal { lit: lit_kind, ty })
    }

    /// Analyze path expression
    fn analyze_path(&mut self, path: ast::Path) -> Option<Expr> {
        // Resolve path to definition
        let name = path.segments.first()?;
        let def_id = self.scope_tree.resolve(name.ident)?;

        // Get type of definition
        let ty = self.type_context.type_of_def(def_id)?.clone();

        Some(Expr::Var { def_id, ty })
    }

    /// Analyze binary expression
    fn analyze_binary(&mut self, expr: ast::BinaryExpr) -> Option<Expr> {
        let left = self.analyze_expr(*expr.left)?;
        let right = self.analyze_expr(*expr.right)?;

        // Get types
        let left_ty = self.expr_type(&left)?;
        let right_ty = self.expr_type(&right)?;

        // Check operator is valid for types
        let result_ty = match expr.op {
            ast::BinOp::Add
            | ast::BinOp::Sub
            | ast::BinOp::Mul
            | ast::BinOp::Div
            | ast::BinOp::Mod => {
                // Arithmetic: both operands must be numeric, result same type
                if !self.is_numeric(&left_ty) || !self.is_numeric(&right_ty) {
                    self.error("arithmetic operation requires numeric types".to_string());
                    return None;
                }
                self.unify_types(&left_ty, &right_ty)?
            }
            ast::BinOp::Eq
            | ast::BinOp::Ne
            | ast::BinOp::Lt
            | ast::BinOp::Gt
            | ast::BinOp::Le
            | ast::BinOp::Ge => {
                // Comparison: both operands must be comparable, result is bool
                if !self.is_comparable(&left_ty, &right_ty) {
                    self.error("cannot compare these types".to_string());
                    return None;
                }
                Type::Bool
            }
            ast::BinOp::And | ast::BinOp::Or => {
                // Logical: both operands must be bool
                if left_ty != Type::Bool || right_ty != Type::Bool {
                    self.error("logical operation requires boolean types".to_string());
                    return None;
                }
                Type::Bool
            }
            _ => unimplemented!(),
        };

        Some(Expr::Binary {
            op: self.convert_binop(expr.op),
            left: Box::new(left),
            right: Box::new(right),
            ty: result_ty,
        })
    }

    /// Analyze unary expression
    fn analyze_unary(&mut self, expr: ast::UnaryExpr) -> Option<Expr> {
        unimplemented!("Unary analysis not implemented")
    }

    /// Analyze function call
    fn analyze_call(&mut self, call: ast::CallExpr) -> Option<Expr> {
        unimplemented!("Call analysis not implemented")
    }

    /// Analyze block expression
    fn analyze_block(&mut self, block: ast::Block) -> Option<Expr> {
        unimplemented!("Block analysis not implemented")
    }

    /// Analyze if expression
    fn analyze_if(&mut self, if_expr: ast::IfExpr) -> Option<Expr> {
        unimplemented!("If analysis not implemented")
    }

    /// Analyze return expression
    fn analyze_return(&mut self, expr: Option<Box<ast::Expr>>) -> Option<Expr> {
        unimplemented!("Return analysis not implemented")
    }

    /// Get type of expression
    fn expr_type(&self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Literal { ty, .. } => Some(ty.clone()),
            Expr::Var { ty, .. } => Some(ty.clone()),
            Expr::Binary { ty, .. } => Some(ty.clone()),
            Expr::Unary { ty, .. } => Some(ty.clone()),
            Expr::Call { ty, .. } => Some(ty.clone()),
            Expr::MethodCall { ty, .. } => Some(ty.clone()),
            Expr::Field { ty, .. } => Some(ty.clone()),
            Expr::Block { ty, .. } => Some(ty.clone()),
            Expr::If { ty, .. } => Some(ty.clone()),
            Expr::Match { ty, .. } => Some(ty.clone()),
            _ => None,
        }
    }

    /// Check if type is numeric
    fn is_numeric(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int | Type::Float)
    }

    /// Check if types are comparable
    fn is_comparable(&self, t1: &Type, t2: &Type) -> bool {
        // Simplified: require same type for now
        t1 == t2 && matches!(t1, Type::Int | Type::Float | Type::Bool | Type::String)
    }

    /// Unify two types
    fn unify_types(&mut self, t1: &Type, t2: &Type) -> Option<Type> {
        if t1 == t2 {
            return Some(t1.clone());
        }

        // Add constraint for type inference
        self.type_context.add_eq_constraint(t1.clone(), t2.clone());

        // For now, just return first type
        // In full implementation, would solve constraints
        Some(t1.clone())
    }

    /// Convert AST binary op to HIR binary op
    fn convert_binop(&self, op: ast::BinOp) -> BinOp {
        match op {
            ast::BinOp::Add => BinOp::Add,
            ast::BinOp::Sub => BinOp::Sub,
            ast::BinOp::Mul => BinOp::Mul,
            ast::BinOp::Div => BinOp::Div,
            ast::BinOp::Mod => BinOp::Mod,
            ast::BinOp::Eq => BinOp::Eq,
            ast::BinOp::Ne => BinOp::Ne,
            ast::BinOp::Lt => BinOp::Lt,
            ast::BinOp::Gt => BinOp::Gt,
            ast::BinOp::Le => BinOp::Le,
            ast::BinOp::Ge => BinOp::Ge,
            ast::BinOp::And => BinOp::And,
            ast::BinOp::Or => BinOp::Or,
            _ => unimplemented!(),
        }
    }

    /// Report error
    fn error(&mut self, message: String) {
        // Emit diagnostic
        unimplemented!("Error reporting not implemented")
    }
}
