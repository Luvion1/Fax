use faxc_util::{Idx, IndexVec, Symbol, DefId};
use std::collections::HashMap;

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
    /// Character type
    Char,
    /// String type (GC-managed)
    String,
    /// Named type (struct, enum, etc.)
    Adt(DefId),
    /// Type parameter
    Param(ParamId),
    /// Reference type
    Ref(Box<Type>, bool),
    /// Tuple type
    Tuple(Vec<Type>),
    /// Array type [T; N]
    Array(Box<Type>, usize),
    /// Slice type [T]
    Slice(Box<Type>),
    /// Function type fn(A, B) -> C
    Fn(Vec<Type>, Box<Type>),
    /// Future type
    Future(Box<Type>),
    /// Type variable (for inference)
    Infer(InferId),
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

/// Type context - stores all type information
#[derive(Default)]
pub struct TypeContext {
    /// Type of each definition
    pub def_types: HashMap<DefId, Type>,
    /// Type of each expression
    pub expr_types: HashMap<ExprId, Type>,
    /// Inference variable substitutions
    pub substitutions: IndexVec<InferId, Option<Type>>,
    /// Constraints to solve
    pub constraints: Vec<Constraint>,
}

/// Expression ID (placeholder, should match HIR)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub u32);

/// Type constraint
#[derive(Debug, Clone)]
pub enum Constraint {
    Eq(Type, Type),
    Trait(Type, DefId),
}

impl TypeContext {
    pub fn type_of_def(&self, def_id: DefId) -> Option<&Type> {
        self.def_types.get(&def_id)
    }

    pub fn set_def_type(&mut self, def_id: DefId, ty: Type) {
        self.def_types.insert(def_id, ty);
    }

    pub fn add_eq_constraint(&mut self, t1: Type, t2: Type) {
        self.constraints.push(Constraint::Eq(t1, t2));
    }

    pub fn new_infer_var(&mut self) -> InferId {
        self.substitutions.push(None)
    }

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
