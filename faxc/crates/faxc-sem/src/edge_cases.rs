//! Edge case tests for faxc-sem

#[cfg(test)]
mod tests {
    use crate::{Type, TypeContext, ScopeTree, RibKind, SemanticAnalyzer};
    use faxc_util::{Handler, Symbol, DefIdGenerator};

    // ==================== SCOPE TREE TESTS ====================

    /// EDGE CASE: New scope tree
    #[test]
    fn test_edge_new_scope_tree() {
        let tree = ScopeTree::new();
        // Should have root scope
        assert!(tree.resolve(Symbol::intern("nonexistent")).is_none());
    }

    /// EDGE CASE: Enter and exit scope
    #[test]
    fn test_edge_enter_exit_scope() {
        let mut tree = ScopeTree::new();
        tree.enter_scope(RibKind::Block);
        tree.exit_scope();
        // Should be back to root
    }

    /// EDGE CASE: Nested scopes
    #[test]
    fn test_edge_nested_scopes() {
        let mut tree = ScopeTree::new();
        tree.enter_scope(RibKind::Block);
        tree.enter_scope(RibKind::Block);
        tree.enter_scope(RibKind::Block);
        
        let def_id = DefIdGenerator::new().next();
        tree.add_binding(Symbol::intern("x"), def_id);
        
        // Should resolve in nested scope
        assert!(tree.resolve(Symbol::intern("x")).is_some());
        
        tree.exit_scope();
        tree.exit_scope();
        tree.exit_scope();
        
        // Should still resolve after exiting (binding is in inner scope but we're back at root)
        // Actually, after exiting all scopes, we're at root which doesn't have the binding
        assert!(tree.resolve(Symbol::intern("x")).is_none());
    }

    /// EDGE CASE: Shadowing in nested scope
    #[test]
    fn test_edge_shadowing() {
        let mut tree = ScopeTree::new();
        let gen = &mut DefIdGenerator::new();
        
        let outer_id = gen.next();
        tree.add_binding(Symbol::intern("x"), outer_id);
        
        tree.enter_scope(RibKind::Block);
        let inner_id = gen.next();
        tree.add_binding(Symbol::intern("x"), inner_id);
        
        // Should resolve to inner binding
        let resolved = tree.resolve(Symbol::intern("x")).unwrap();
        assert_eq!(resolved, inner_id);
        
        tree.exit_scope();
        
        // Should resolve to outer binding
        let resolved = tree.resolve(Symbol::intern("x")).unwrap();
        assert_eq!(resolved, outer_id);
    }

    /// EDGE CASE: Multiple bindings in same scope
    #[test]
    fn test_edge_multiple_bindings() {
        let mut tree = ScopeTree::new();
        let gen = &mut DefIdGenerator::new();
        
        tree.add_binding(Symbol::intern("a"), gen.next());
        tree.add_binding(Symbol::intern("b"), gen.next());
        tree.add_binding(Symbol::intern("c"), gen.next());
        
        assert!(tree.resolve(Symbol::intern("a")).is_some());
        assert!(tree.resolve(Symbol::intern("b")).is_some());
        assert!(tree.resolve(Symbol::intern("c")).is_some());
    }

    /// EDGE CASE: Function scope
    #[test]
    fn test_edge_function_scope() {
        let mut tree = ScopeTree::new();
        tree.enter_scope(RibKind::Function);
        
        let def_id = DefIdGenerator::new().next();
        tree.add_binding(Symbol::intern("param"), def_id);
        
        assert!(tree.resolve(Symbol::intern("param")).is_some());
        tree.exit_scope();
    }

    /// EDGE CASE: Loop scope
    #[test]
    fn test_edge_loop_scope() {
        let mut tree = ScopeTree::new();
        tree.enter_scope(RibKind::Loop(None));
        
        let def_id = DefIdGenerator::new().next();
        tree.add_binding(Symbol::intern("i"), def_id);
        
        assert!(tree.resolve(Symbol::intern("i")).is_some());
        tree.exit_scope();
    }

    // ==================== TYPE CONTEXT TESTS ====================

    /// EDGE CASE: New type context
    #[test]
    fn test_edge_new_type_context() {
        let ctx = TypeContext::default();
        assert!(ctx.def_types.is_empty());
    }

    /// EDGE CASE: Set and get type
    #[test]
    fn test_edge_set_get_type() {
        let mut ctx = TypeContext::default();
        let def_id = DefIdGenerator::new().next();
        
        ctx.set_def_type(def_id, Type::Int);
        
        assert_eq!(ctx.type_of_def(def_id), Some(&Type::Int));
    }

    /// EDGE CASE: Get non-existent type
    #[test]
    fn test_edge_get_nonexistent_type() {
        let ctx = TypeContext::default();
        let def_id = DefIdGenerator::new().next();
        
        assert_eq!(ctx.type_of_def(def_id), None);
    }

    /// EDGE CASE: Type equality constraint
    #[test]
    fn test_edge_eq_constraint() {
        let mut ctx = TypeContext::default();
        ctx.add_eq_constraint(Type::Int, Type::Int);
        
        assert_eq!(ctx.constraints.len(), 1);
    }

    /// EDGE CASE: New inference variable
    #[test]
    fn test_edge_new_infer_var() {
        let mut ctx = TypeContext::default();
        let id = ctx.new_infer_var();
        
        assert_eq!(id.0, 0);
    }

    /// EDGE CASE: Substitute uninferred variable
    #[test]
    fn test_edge_substitute_uninferred() {
        let mut ctx = TypeContext::default();
        let id = ctx.new_infer_var();
        
        let result = ctx.substitute(&Type::Infer(id));
        assert_eq!(result, Type::Infer(id));
    }

    /// EDGE CASE: Substitute inferred variable
    #[test]
    fn test_edge_substitute_inferred() {
        let mut ctx = TypeContext::default();
        let id = ctx.new_infer_var();
        ctx.substitutions[id] = Some(Type::Int);
        
        let result = ctx.substitute(&Type::Infer(id));
        assert_eq!(result, Type::Int);
    }

    /// EDGE CASE: Substitute tuple type
    #[test]
    fn test_edge_substitute_tuple() {
        let mut ctx = TypeContext::default();
        let id = ctx.new_infer_var();
        ctx.substitutions[id] = Some(Type::Int);
        
        let tuple = Type::Tuple(vec![Type::Infer(id), Type::Bool]);
        let result = ctx.substitute(&tuple);
        
        assert_eq!(result, Type::Tuple(vec![Type::Int, Type::Bool]));
    }

    /// EDGE CASE: Substitute ref type
    #[test]
    fn test_edge_substitute_ref() {
        let mut ctx = TypeContext::default();
        let id = ctx.new_infer_var();
        ctx.substitutions[id] = Some(Type::Int);
        
        let reff = Type::Ref(Box::new(Type::Infer(id)), false);
        let result = ctx.substitute(&reff);
        
        assert_eq!(result, Type::Ref(Box::new(Type::Int), false));
    }

    /// EDGE CASE: Substitute fn type
    #[test]
    fn test_edge_substitute_fn() {
        let mut ctx = TypeContext::default();
        let id = ctx.new_infer_var();
        ctx.substitutions[id] = Some(Type::Int);
        
        let fn_ty = Type::Fn(vec![Type::Infer(id)], Box::new(Type::Bool));
        let result = ctx.substitute(&fn_ty);
        
        assert_eq!(result, Type::Fn(vec![Type::Int], Box::new(Type::Bool)));
    }

    // ==================== TYPE VARIANTS TESTS ====================

    /// EDGE CASE: All primitive types
    #[test]
    fn test_edge_all_primitives() {
        let _int = Type::Int;
        let _float = Type::Float;
        let _bool = Type::Bool;
        let _char = Type::Char;
        let _string = Type::String;
        let _unit = Type::Unit;
        let _error = Type::Error;
        let _never = Type::Never;
    }

    /// EDGE CASE: Complex types
    #[test]
    fn test_edge_complex_types() {
        let _array = Type::Array(Box::new(Type::Int), 10);
        let _slice = Type::Slice(Box::new(Type::Int));
        let _tuple = Type::Tuple(vec![Type::Int, Type::Bool, Type::String]);
        let _ref = Type::Ref(Box::new(Type::Int), true);
        let _fn_ty = Type::Fn(vec![Type::Int, Type::Int], Box::new(Type::Int));
        let _future = Type::Future(Box::new(Type::Int));
    }

    /// EDGE CASE: Type equality
    #[test]
    fn test_edge_type_equality() {
        assert_eq!(Type::Int, Type::Int);
        assert_eq!(Type::Bool, Type::Bool);
        assert_ne!(Type::Int, Type::Bool);
        assert_ne!(Type::Int, Type::Float);
    }

    /// EDGE CASE: Nested array type
    #[test]
    fn test_edge_nested_array() {
        let nested = Type::Array(Box::new(Type::Array(Box::new(Type::Int), 5)), 10);
        assert!(matches!(nested, Type::Array(_, _)));
    }

    /// EDGE CASE: Empty tuple
    #[test]
    fn test_edge_empty_tuple() {
        let empty = Type::Tuple(vec![]);
        assert_eq!(empty, Type::Unit);
    }

    // ==================== ERROR CASES ====================

    /// ERROR CASE: Type mismatch in constraint
    #[test]
    fn test_err_type_mismatch() {
        let mut ctx = TypeContext::default();
        ctx.add_eq_constraint(Type::Int, Type::Bool);
        
        // Constraint is added but not solved - this is expected behavior
        assert_eq!(ctx.constraints.len(), 1);
    }

    /// ERROR CASE: Circular type reference (would cause infinite loop in solver)
    #[test]
    fn test_edge_circular_reference() {
        let mut ctx = TypeContext::default();
        let id1 = ctx.new_infer_var();
        let id2 = ctx.new_infer_var();
        
        // Create circular reference
        ctx.substitutions[id1] = Some(Type::Infer(id2));
        ctx.substitutions[id2] = Some(Type::Infer(id1));
        
        // This should not panic - just return the uninferred type
        let result = ctx.substitute(&Type::Infer(id1));
        // Result depends on implementation - may be Error or original type
        assert!(matches!(result, Type::Infer(_) | Type::Error));
    }

    /// ERROR CASE: Deep nesting
    #[test]
    fn test_edge_deep_nesting() {
        let mut ctx = TypeContext::default();
        let mut current = Type::Int;
        
        for _ in 0..100 {
            current = Type::Array(Box::new(current), 1);
        }
        
        // Should not panic
        let _ = ctx.substitute(&current);
    }

    // ==================== SEMANTIC ANALYZER TESTS ====================

    /// EDGE CASE: New analyzer
    #[test]
    fn test_edge_new_analyzer() {
        let mut type_ctx = TypeContext::default();
        let def_id_gen = DefIdGenerator::new();
        let mut handler = Handler::new();
        
        let analyzer = SemanticAnalyzer::new(&mut type_ctx, &def_id_gen, &mut handler);
        
        assert!(analyzer.loop_stack.is_empty());
        assert_eq!(analyzer.current_ret_type, None);
    }

    /// EDGE CASE: Empty items analysis
    #[test]
    fn test_edge_empty_items() {
        let mut type_ctx = TypeContext::default();
        let def_id_gen = DefIdGenerator::new();
        let mut handler = Handler::new();
        
        let mut analyzer = SemanticAnalyzer::new(&mut type_ctx, &def_id_gen, &mut handler);
        let result = analyzer.analyze_items(vec![]);
        
        assert!(result.is_empty());
    }

    /// EDGE CASE: Scope tree access
    #[test]
    fn test_edge_scope_access() {
        let mut type_ctx = TypeContext::default();
        let def_id_gen = DefIdGenerator::new();
        let mut handler = Handler::new();
        
        let mut analyzer = SemanticAnalyzer::new(&mut type_ctx, &def_id_gen, &mut handler);
        
        // Access scope tree
        analyzer.scope_tree.enter_scope(RibKind::Block);
        analyzer.scope_tree.add_binding(Symbol::intern("x"), def_id_gen.next());
        
        assert!(analyzer.scope_tree.resolve(Symbol::intern("x")).is_some());
    }
}