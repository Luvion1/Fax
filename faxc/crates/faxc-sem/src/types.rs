use faxc_util::{DefId, Idx, IndexVec};
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
    /// Primitive integer type (64-bit)
    Int,
    /// Primitive unsigned integer type (64-bit)
    UInt,
    /// Primitive float type (64-bit)
    Float,
    /// Boolean type
    Bool,
    /// Character type
    Char,
    /// String type (GC-managed)
    String,
    /// 8-bit integer
    Int8,
    /// 8-bit unsigned integer
    UInt8,
    /// 16-bit integer
    Int16,
    /// 16-bit unsigned integer
    UInt16,
    /// 32-bit integer
    Int32,
    /// 32-bit unsigned integer
    UInt32,
    /// 32-bit float
    Float32,
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
    /// Option type Option<T>
    Option(Box<Type>),
    /// Result type Result<T, E>
    Result(Box<Type>, Box<Type>),
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

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Type Enum Tests
    // ========================================================================

    #[test]
    fn test_type_error() {
        let ty = Type::Error;
        assert_eq!(ty, Type::Error);
    }

    #[test]
    fn test_type_unit() {
        let ty = Type::Unit;
        assert_eq!(ty, Type::Unit);
    }

    #[test]
    fn test_type_never() {
        let ty = Type::Never;
        assert_eq!(ty, Type::Never);
    }

    #[test]
    fn test_type_int() {
        let ty = Type::Int;
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn test_type_float() {
        let ty = Type::Float;
        assert_eq!(ty, Type::Float);
    }

    #[test]
    fn test_type_bool() {
        let ty = Type::Bool;
        assert_eq!(ty, Type::Bool);
    }

    #[test]
    fn test_type_char() {
        let ty = Type::Char;
        assert_eq!(ty, Type::Char);
    }

    #[test]
    fn test_type_string() {
        let ty = Type::String;
        assert_eq!(ty, Type::String);
    }

    #[test]
    fn test_type_adt() {
        let def_id = DefId(42);
        let ty = Type::Adt(def_id);
        assert_eq!(ty, Type::Adt(def_id));
    }

    #[test]
    fn test_type_param() {
        let param_id = ParamId(0);
        let ty = Type::Param(param_id);
        assert_eq!(ty, Type::Param(param_id));
    }

    #[test]
    fn test_type_ref() {
        let ty = Type::Ref(Box::new(Type::Int), false);
        assert_eq!(ty, Type::Ref(Box::new(Type::Int), false));

        let mutable_ref = Type::Ref(Box::new(Type::Int), true);
        assert_eq!(mutable_ref, Type::Ref(Box::new(Type::Int), true));
    }

    #[test]
    fn test_type_tuple() {
        let ty = Type::Tuple(vec![Type::Int, Type::Bool, Type::String]);
        assert_eq!(ty, Type::Tuple(vec![Type::Int, Type::Bool, Type::String]));

        let empty_tuple = Type::Tuple(vec![]);
        assert_eq!(empty_tuple, Type::Tuple(vec![]));
    }

    #[test]
    fn test_type_array() {
        let ty = Type::Array(Box::new(Type::Int), 10);
        assert_eq!(ty, Type::Array(Box::new(Type::Int), 10));
    }

    #[test]
    fn test_type_slice() {
        let ty = Type::Slice(Box::new(Type::Int));
        assert_eq!(ty, Type::Slice(Box::new(Type::Int)));
    }

    #[test]
    fn test_type_fn() {
        let ty = Type::Fn(vec![Type::Int, Type::String], Box::new(Type::Bool));
        assert_eq!(
            ty,
            Type::Fn(vec![Type::Int, Type::String], Box::new(Type::Bool))
        );

        let no_params = Type::Fn(vec![], Box::new(Type::Unit));
        assert_eq!(no_params, Type::Fn(vec![], Box::new(Type::Unit)));
    }

    #[test]
    fn test_type_future() {
        let ty = Type::Future(Box::new(Type::Int));
        assert_eq!(ty, Type::Future(Box::new(Type::Int)));
    }

    #[test]
    fn test_type_infer() {
        let infer_id = InferId(0);
        let ty = Type::Infer(infer_id);
        assert_eq!(ty, Type::Infer(infer_id));
    }

    #[test]
    fn test_type_clone() {
        let ty = Type::Tuple(vec![Type::Int, Type::Bool]);
        let cloned = ty.clone();
        assert_eq!(ty, cloned);
    }

    #[test]
    fn test_type_debug() {
        let ty = Type::Int;
        let debug_str = format!("{:?}", ty);
        assert!(debug_str.contains("Int"));
    }

    // ========================================================================
    // ParamId Tests
    // ========================================================================

    #[test]
    fn test_param_id_from_usize() {
        let param_id = ParamId::from_usize(42);
        assert_eq!(param_id.0, 42);
    }

    #[test]
    fn test_param_id_index() {
        let param_id = ParamId(100);
        assert_eq!(param_id.index(), 100);
    }

    #[test]
    fn test_param_id_equality() {
        let p1 = ParamId(1);
        let p2 = ParamId(1);
        let p3 = ParamId(2);

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_param_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let p1 = ParamId(1);
        let p2 = ParamId(2);
        let p3 = ParamId(1);

        set.insert(p1);
        set.insert(p2);
        set.insert(p3);

        assert_eq!(set.len(), 2);
    }

    // ========================================================================
    // InferId Tests
    // ========================================================================

    #[test]
    fn test_infer_id_from_usize() {
        let infer_id = InferId::from_usize(42);
        assert_eq!(infer_id.0, 42);
    }

    #[test]
    fn test_infer_id_index() {
        let infer_id = InferId(100);
        assert_eq!(infer_id.index(), 100);
    }

    #[test]
    fn test_infer_id_equality() {
        let i1 = InferId(1);
        let i2 = InferId(1);
        let i3 = InferId(2);

        assert_eq!(i1, i2);
        assert_ne!(i1, i3);
    }

    #[test]
    fn test_infer_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let i1 = InferId(1);
        let i2 = InferId(2);
        let i3 = InferId(1);

        set.insert(i1);
        set.insert(i2);
        set.insert(i3);

        assert_eq!(set.len(), 2);
    }

    // ========================================================================
    // ExprId Tests
    // ========================================================================

    #[test]
    fn test_expr_id_creation() {
        let expr_id = ExprId(42);
        assert_eq!(expr_id.0, 42);
    }

    #[test]
    fn test_expr_id_equality() {
        let e1 = ExprId(1);
        let e2 = ExprId(1);
        let e3 = ExprId(2);

        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
    }

    #[test]
    fn test_expr_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let e1 = ExprId(1);
        let e2 = ExprId(2);
        let e3 = ExprId(1);

        set.insert(e1);
        set.insert(e2);
        set.insert(e3);

        assert_eq!(set.len(), 2);
    }

    // ========================================================================
    // Constraint Tests
    // ========================================================================

    #[test]
    fn test_constraint_eq() {
        let c = Constraint::Eq(Type::Int, Type::Int);
        match c {
            Constraint::Eq(t1, t2) => {
                assert_eq!(t1, Type::Int);
                assert_eq!(t2, Type::Int);
            },
            _ => panic!("Expected Eq constraint"),
        }
    }

    #[test]
    fn test_constraint_trait() {
        let def_id = DefId(42);
        let c = Constraint::Trait(Type::Int, def_id);
        match c {
            Constraint::Trait(t, d) => {
                assert_eq!(t, Type::Int);
                assert_eq!(d, def_id);
            },
            _ => panic!("Expected Trait constraint"),
        }
    }

    #[test]
    fn test_constraint_clone() {
        let c = Constraint::Eq(Type::Int, Type::Bool);
        let cloned = c.clone();
        match cloned {
            Constraint::Eq(t1, t2) => {
                assert_eq!(t1, Type::Int);
                assert_eq!(t2, Type::Bool);
            },
            _ => panic!("Expected Eq constraint"),
        }
    }

    #[test]
    fn test_constraint_debug() {
        let c = Constraint::Eq(Type::Int, Type::Bool);
        let debug_str = format!("{:?}", c);
        assert!(debug_str.contains("Eq"));
    }

    // ========================================================================
    // TypeContext Tests
    // ========================================================================

    #[test]
    fn test_type_context_default() {
        let ctx: TypeContext = TypeContext::default();
        assert!(ctx.def_types.is_empty());
        assert!(ctx.expr_types.is_empty());
        assert!(ctx.constraints.is_empty());
    }

    #[test]
    fn test_type_context_set_def_type() {
        let mut ctx = TypeContext::default();
        let def_id = DefId(1);

        ctx.set_def_type(def_id, Type::Int);

        let ty = ctx.type_of_def(def_id);
        assert_eq!(ty, Some(&Type::Int));
    }

    #[test]
    fn test_type_context_type_of_def_not_found() {
        let ctx = TypeContext::default();
        let def_id = DefId(1);

        let ty = ctx.type_of_def(def_id);
        assert_eq!(ty, None);
    }

    #[test]
    fn test_type_context_multiple_def_types() {
        let mut ctx = TypeContext::default();

        ctx.set_def_type(DefId(1), Type::Int);
        ctx.set_def_type(DefId(2), Type::Bool);
        ctx.set_def_type(DefId(3), Type::String);

        assert_eq!(ctx.type_of_def(DefId(1)), Some(&Type::Int));
        assert_eq!(ctx.type_of_def(DefId(2)), Some(&Type::Bool));
        assert_eq!(ctx.type_of_def(DefId(3)), Some(&Type::String));
        assert_eq!(ctx.type_of_def(DefId(4)), None);
    }

    #[test]
    fn test_type_context_add_eq_constraint() {
        let mut ctx = TypeContext::default();

        ctx.add_eq_constraint(Type::Int, Type::Int);

        assert_eq!(ctx.constraints.len(), 1);
        match &ctx.constraints[0] {
            Constraint::Eq(t1, t2) => {
                assert_eq!(t1, &Type::Int);
                assert_eq!(t2, &Type::Int);
            },
            _ => panic!("Expected Eq constraint"),
        }
    }

    #[test]
    fn test_type_context_multiple_constraints() {
        let mut ctx = TypeContext::default();

        ctx.add_eq_constraint(Type::Int, Type::Int);
        ctx.add_eq_constraint(Type::Bool, Type::Bool);

        assert_eq!(ctx.constraints.len(), 2);
    }

    #[test]
    fn test_type_context_new_infer_var() {
        let mut ctx = TypeContext::default();

        let id1 = ctx.new_infer_var();
        let id2 = ctx.new_infer_var();
        let id3 = ctx.new_infer_var();

        assert_eq!(id1, InferId(0));
        assert_eq!(id2, InferId(1));
        assert_eq!(id3, InferId(2));
    }

    #[test]
    fn test_type_context_substitute_infer() {
        let mut ctx = TypeContext::default();
        let infer_id = ctx.new_infer_var();

        // Without substitution, should return the same infer type
        let ty = Type::Infer(infer_id);
        let result = ctx.substitute(&ty);
        assert_eq!(result, Type::Infer(infer_id));
    }

    #[test]
    fn test_type_context_substitute_tuple() {
        let ctx = TypeContext::default();
        let ty = Type::Tuple(vec![Type::Int, Type::Bool]);

        let result = ctx.substitute(&ty);
        assert_eq!(result, Type::Tuple(vec![Type::Int, Type::Bool]));
    }

    #[test]
    fn test_type_context_substitute_ref() {
        let ctx = TypeContext::default();
        let ty = Type::Ref(Box::new(Type::Int), false);

        let result = ctx.substitute(&ty);
        assert_eq!(result, Type::Ref(Box::new(Type::Int), false));
    }

    #[test]
    fn test_type_context_substitute_array() {
        let ctx = TypeContext::default();
        let ty = Type::Array(Box::new(Type::Int), 10);

        let result = ctx.substitute(&ty);
        assert_eq!(result, Type::Array(Box::new(Type::Int), 10));
    }

    #[test]
    fn test_type_context_substitute_fn() {
        let ctx = TypeContext::default();
        let ty = Type::Fn(vec![Type::Int, Type::Bool], Box::new(Type::String));

        let result = ctx.substitute(&ty);
        assert_eq!(
            result,
            Type::Fn(vec![Type::Int, Type::Bool], Box::new(Type::String))
        );
    }

    #[test]
    fn test_type_context_substitute_primitive() {
        let ctx = TypeContext::default();

        assert_eq!(ctx.substitute(&Type::Int), Type::Int);
        assert_eq!(ctx.substitute(&Type::Bool), Type::Bool);
        assert_eq!(ctx.substitute(&Type::String), Type::String);
        assert_eq!(ctx.substitute(&Type::Unit), Type::Unit);
    }
}
