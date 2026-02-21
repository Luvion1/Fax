//! Edge case tests for faxc-mir

#[cfg(test)]
mod tests {
    use crate::{Function, Local, LocalId, BlockId, BasicBlock, Statement, Terminator};
    use crate::{Place, Rvalue, Operand, Constant, ConstantKind};
    use crate::{BinOp, UnOp, Mutability, CastKind, AggregateKind};
    use faxc_sem::Type;
    use faxc_util::{Symbol, IndexVec, Span};

    // ==================== FUNCTION TESTS ====================

    /// EDGE CASE: Empty function
    #[test]
    fn test_edge_empty_function() {
        let mut locals: IndexVec<LocalId, Local> = IndexVec::new();
        let mut blocks: IndexVec<BlockId, BasicBlock> = IndexVec::new();
        
        let entry = BlockId(0);
        
        let func = Function {
            name: Symbol::intern("empty"),
            locals,
            blocks,
            entry_block: entry,
            return_ty: Type::Unit,
        };
        
        assert_eq!(func.name, Symbol::intern("empty"));
        assert_eq!(func.return_ty, Type::Unit);
    }

    /// EDGE CASE: Function with single local
    #[test]
    fn test_edge_single_local() {
        let mut locals: IndexVec<LocalId, Local> = IndexVec::new();
        locals.push(Local {
            ty: Type::Int,
            span: Span::DUMMY,
            name: Some(Symbol::intern("x")),
        });
        
        assert_eq!(locals.len(), 1);
    }

    /// EDGE CASE: Function with many locals
    #[test]
    fn test_edge_many_locals() {
        let mut locals: IndexVec<LocalId, Local> = IndexVec::new();
        for i in 0..100 {
            locals.push(Local {
                ty: Type::Int,
                span: Span::DUMMY,
                name: Some(Symbol::intern(&format!("var{}", i))),
            });
        }
        assert_eq!(locals.len(), 100);
    }

    // ==================== BASIC BLOCK TESTS ====================

    /// EDGE CASE: Empty basic block
    #[test]
    fn test_edge_empty_block() {
        let block = BasicBlock {
            id: BlockId(0),
            statements: vec![],
            terminator: Terminator::Return,
        };
        
        assert!(block.statements.is_empty());
    }

    /// EDGE CASE: Block with single statement
    #[test]
    fn test_edge_single_stmt() {
        let block = BasicBlock {
            id: BlockId(0),
            statements: vec![Statement::Nop],
            terminator: Terminator::Return,
        };
        
        assert_eq!(block.statements.len(), 1);
    }

    /// EDGE CASE: Block with many statements
    #[test]
    fn test_edge_many_stmts() {
        let stmts: Vec<_> = (0..100).map(|_| Statement::Nop).collect();
        let block = BasicBlock {
            id: BlockId(0),
            statements: stmts,
            terminator: Terminator::Return,
        };
        
        assert_eq!(block.statements.len(), 100);
    }

    // ==================== TERMINATOR TESTS ====================

    /// EDGE CASE: Goto terminator
    #[test]
    fn test_edge_goto() {
        let term = Terminator::Goto { target: BlockId(1) };
        assert!(matches!(term, Terminator::Goto { .. }));
    }

    /// EDGE CASE: If terminator
    #[test]
    fn test_edge_if_term() {
        let term = Terminator::If {
            cond: Operand::Constant(Constant {
                ty: Type::Bool,
                kind: ConstantKind::Bool(true),
            }),
            then_block: BlockId(1),
            else_block: BlockId(2),
        };
        assert!(matches!(term, Terminator::If { .. }));
    }

    /// EDGE CASE: SwitchInt terminator
    #[test]
    fn test_edge_switch_int() {
        let term = Terminator::SwitchInt {
            discr: Operand::Constant(Constant {
                ty: Type::Int,
                kind: ConstantKind::Int(0),
            }),
            switch_ty: Type::Int,
            targets: vec![(0, BlockId(1)), (1, BlockId(2))],
            otherwise: BlockId(3),
        };
        assert!(matches!(term, Terminator::SwitchInt { .. }));
    }

    /// EDGE CASE: Return terminator
    #[test]
    fn test_edge_return() {
        let term = Terminator::Return;
        assert!(matches!(term, Terminator::Return));
    }

    /// EDGE CASE: Unreachable terminator
    #[test]
    fn test_edge_unreachable() {
        let term = Terminator::Unreachable;
        assert!(matches!(term, Terminator::Unreachable));
    }

    /// EDGE CASE: Call terminator
    #[test]
    fn test_edge_call() {
        let term = Terminator::Call {
            func: Operand::Constant(Constant {
                ty: Type::Fn(vec![], Box::new(Type::Int)),
                kind: ConstantKind::Unit,
            }),
            args: vec![],
            destination: Place::Local(LocalId(0)),
            target: Some(BlockId(1)),
            cleanup: None,
        };
        assert!(matches!(term, Terminator::Call { .. }));
    }

    // ==================== STATEMENT TESTS ====================

    /// EDGE CASE: Nop statement
    #[test]
    fn test_edge_nop() {
        let stmt = Statement::Nop;
        assert!(matches!(stmt, Statement::Nop));
    }

    /// EDGE CASE: Assign statement
    #[test]
    fn test_edge_assign() {
        let stmt = Statement::Assign(
            Place::Local(LocalId(0)),
            Rvalue::Use(Operand::Constant(Constant {
                ty: Type::Int,
                kind: ConstantKind::Int(42),
            })),
        );
        assert!(matches!(stmt, Statement::Assign(_, _)));
    }

    /// EDGE CASE: StorageLive statement
    #[test]
    fn test_edge_storage_live() {
        let stmt = Statement::StorageLive(LocalId(0));
        assert!(matches!(stmt, Statement::StorageLive(_)));
    }

    /// EDGE CASE: StorageDead statement
    #[test]
    fn test_edge_storage_dead() {
        let stmt = Statement::StorageDead(LocalId(0));
        assert!(matches!(stmt, Statement::StorageDead(_)));
    }

    // ==================== PLACE TESTS ====================

    /// EDGE CASE: Local place
    #[test]
    fn test_edge_local_place() {
        let place = Place::Local(LocalId(0));
        assert!(matches!(place, Place::Local(_)));
    }

    /// EDGE CASE: Projection place
    #[test]
    fn test_edge_projection_place() {
        use crate::Projection;
        let place = Place::Projection(
            Box::new(Place::Local(LocalId(0))),
            Projection::Field(0),
        );
        assert!(matches!(place, Place::Projection(_, _)));
    }

    // ==================== RVALUE TESTS ====================

    /// EDGE CASE: Use rvalue
    #[test]
    fn test_edge_use() {
        let rv = Rvalue::Use(Operand::Constant(Constant {
            ty: Type::Int,
            kind: ConstantKind::Int(42),
        }));
        assert!(matches!(rv, Rvalue::Use(_)));
    }

    /// EDGE CASE: BinaryOp rvalue
    #[test]
    fn test_edge_binary_op() {
        let rv = Rvalue::BinaryOp(
            BinOp::Add,
            Box::new(Operand::Constant(Constant {
                ty: Type::Int,
                kind: ConstantKind::Int(1),
            })),
            Box::new(Operand::Constant(Constant {
                ty: Type::Int,
                kind: ConstantKind::Int(2),
            })),
        );
        assert!(matches!(rv, Rvalue::BinaryOp(_, _, _)));
    }

    /// EDGE CASE: UnaryOp rvalue
    #[test]
    fn test_edge_unary_op() {
        let rv = Rvalue::UnaryOp(
            UnOp::Neg,
            Operand::Constant(Constant {
                ty: Type::Int,
                kind: ConstantKind::Int(5),
            }),
        );
        assert!(matches!(rv, Rvalue::UnaryOp(_, _)));
    }

    /// EDGE CASE: Cast rvalue
    #[test]
    fn test_edge_cast() {
        let rv = Rvalue::Cast(
            CastKind::IntToInt,
            Operand::Constant(Constant {
                ty: Type::Int,
                kind: ConstantKind::Int(42),
            }),
            Type::Int,
        );
        assert!(matches!(rv, Rvalue::Cast(_, _, _)));
    }

    /// EDGE CASE: Aggregate rvalue
    #[test]
    fn test_edge_aggregate() {
        let rv = Rvalue::Aggregate(
            AggregateKind::Tuple,
            vec![],
        );
        assert!(matches!(rv, Rvalue::Aggregate(_, _)));
    }

    // ==================== CONSTANT TESTS ====================

    /// EDGE CASE: Int constant
    #[test]
    fn test_edge_int_const() {
        let c = Constant {
            ty: Type::Int,
            kind: ConstantKind::Int(42),
        };
        assert!(matches!(c.kind, ConstantKind::Int(42)));
    }

    /// EDGE CASE: Float constant
    #[test]
    fn test_edge_float_const() {
        let c = Constant {
            ty: Type::Float,
            kind: ConstantKind::Float(3.14),
        };
        assert!(matches!(c.kind, ConstantKind::Float(f) if (f - 3.14).abs() < 0.01));
    }

    /// EDGE CASE: Bool constant
    #[test]
    fn test_edge_bool_const() {
        let c = Constant {
            ty: Type::Bool,
            kind: ConstantKind::Bool(true),
        };
        assert!(matches!(c.kind, ConstantKind::Bool(true)));
    }

    /// EDGE CASE: String constant
    #[test]
    fn test_edge_string_const() {
        let c = Constant {
            ty: Type::String,
            kind: ConstantKind::String(Symbol::intern("hello")),
        };
        assert!(matches!(c.kind, ConstantKind::String(_)));
    }

    /// EDGE CASE: Unit constant
    #[test]
    fn test_edge_unit_const() {
        let c = Constant {
            ty: Type::Unit,
            kind: ConstantKind::Unit,
        };
        assert!(matches!(c.kind, ConstantKind::Unit));
    }

    // ==================== OPERATOR TESTS ====================

    /// EDGE CASE: All binary operators
    #[test]
    fn test_edge_all_bin_ops() {
        let _add = BinOp::Add;
        let _sub = BinOp::Sub;
        let _mul = BinOp::Mul;
        let _div = BinOp::Div;
        let _rem = BinOp::Rem;
        let _eq = BinOp::Eq;
        let _ne = BinOp::Ne;
        let _lt = BinOp::Lt;
        let _le = BinOp::Le;
        let _gt = BinOp::Gt;
        let _ge = BinOp::Ge;
        let _and = BinOp::BitAnd;
        let _or = BinOp::BitOr;
        let _xor = BinOp::BitXor;
        let _shl = BinOp::Shl;
        let _shr = BinOp::Shr;
    }

    /// EDGE CASE: All unary operators
    #[test]
    fn test_edge_all_un_ops() {
        let _neg = UnOp::Neg;
        let _not = UnOp::Not;
    }

    // ==================== ERROR CASES ====================

    /// ERROR CASE: Invalid block ID reference
    #[test]
    fn test_edge_invalid_block_ref() {
        // This tests that we can create a block referencing a non-existent target
        let block = BasicBlock {
            id: BlockId(0),
            statements: vec![],
            terminator: Terminator::Goto { target: BlockId(999) },
        };
        // Should not panic - validation happens elsewhere
        assert_eq!(block.terminator, Terminator::Goto { target: BlockId(999) });
    }

    /// ERROR CASE: Invalid local ID reference
    #[test]
    fn test_edge_invalid_local_ref() {
        let place = Place::Local(LocalId(999));
        // Should not panic - validation happens elsewhere
        assert!(matches!(place, Place::Local(_)));
    }

    /// EDGE CASE: Deep projection chain
    #[test]
    fn test_edge_deep_projection() {
        use crate::Projection;
        let mut place = Place::Local(LocalId(0));
        for i in 0..10 {
            place = Place::Projection(Box::new(place), Projection::Field(i));
        }
        assert!(matches!(place, Place::Projection(_, _)));
    }
}