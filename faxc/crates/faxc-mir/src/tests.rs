//! MIR Crate Integration Tests
//! 
//! MIR-LIR-CODEGEN-DEV-001: Subtask 1
//! Unit and integration tests for MIR constructs, builder, lowering, and optimizations.

use crate::*;
use faxc_sem::Type;
use faxc_util::Symbol;

#[test]
fn test_function_creation() {
    let name = Symbol::intern("test_fn");
    let func = Function::new(name, Type::Int, 2);
    
    assert_eq!(func.name, name);
    assert_eq!(func.return_ty, Type::Int);
    assert_eq!(func.arg_count, 2);
    assert_eq!(func.block_count(), 0);
}

#[test]
fn test_builder_basic_block() {
    let name = Symbol::intern("test");
    let mut builder = Builder::new(name, Type::Int);
    
    let entry = builder.new_block();
    assert_eq!(entry.0, 0);
    
    let block2 = builder.new_block();
    assert_eq!(block2.0, 1);
}

#[test]
fn test_builder_add_local() {
    let name = Symbol::intern("test");
    let mut builder = Builder::new(name, Type::Int);
    
    let local1 = builder.add_local(Type::Int, None);
    assert_eq!(local1.0, 1); // 0 is return value
    
    let local2 = builder.add_local(Type::Bool, None);
    assert_eq!(local2.0, 2);
}

#[test]
fn test_builder_assign() {
    let name = Symbol::intern("test");
    let mut builder = Builder::new(name, Type::Int);
    
    let entry = builder.new_block();
    builder.set_current_block(entry);
    
    let local = builder.add_local(Type::Int, None);
    builder.assign(
        Place::Local(local),
        Rvalue::Use(Operand::Constant(Constant {
            ty: Type::Int,
            kind: ConstantKind::Int(42),
        })),
    );
    
    let func = builder.build();
    assert_eq!(func.blocks.len(), 1);
    assert_eq!(func.blocks[entry].statements.len(), 1);
}

#[test]
fn test_constant_folding_optimization() {
    use crate::optimize::constant_folding;
    
    let name = Symbol::intern("test");
    let mut func = Function::new(name, Type::Int, 0);
    
    // Create a block with constant expression
    let entry = BlockId::from_usize(0);
    func.blocks.push(BasicBlock {
        id: entry,
        statements: vec![
            Statement::Assign(
                Place::Local(LocalId(1)),
                Rvalue::BinaryOp(
                    BinOp::Add,
                    Box::new(Operand::Constant(Constant {
                        ty: Type::Int,
                        kind: ConstantKind::Int(10),
                    })),
                    Box::new(Operand::Constant(Constant {
                        ty: Type::Int,
                        kind: ConstantKind::Int(20),
                    })),
                ),
            ),
        ],
        terminator: Terminator::Return,
    });
    
    constant_folding(&mut func);
    
    // After folding, should be a constant 30
    if let Statement::Assign(_, Rvalue::Use(Operand::Constant(c))) = &func.blocks[entry].statements[0] {
        if let ConstantKind::Int(val) = c.kind {
            assert_eq!(val, 30);
        } else {
            panic!("Expected Int constant");
        }
    } else {
        panic!("Expected folded constant");
    }
}

#[test]
fn test_lower_literal() {
    use faxc_sem::hir;
    use faxc_util::Span;
    
    let lit_expr = hir::Expr::Literal {
        lit: hir::Literal::Int(42),
        ty: Type::Int,
        span: Span::DUMMY,
    };
    
    let fn_item = hir::FnItem {
        name: Symbol::intern("test"),
        params: Vec::new(),
        ret_type: Type::Int,
        body: Box::new(hir::Block {
            stmts: Vec::new(),
            expr: Some(Box::new(lit_expr)),
            span: Span::DUMMY,
        }),
        span: Span::DUMMY,
    };
    
    let mir_func = lower_hir_function(&fn_item);
    assert_eq!(mir_func.name, Symbol::intern("test"));
    assert_eq!(mir_func.return_ty, Type::Int);
}

#[test]
fn test_mir_terminators() {
    let name = Symbol::intern("test");
    let mut builder = Builder::new(name, Type::Int);
    
    let entry = builder.new_block();
    builder.set_current_block(entry);
    
    // Test Goto terminator
    let target = builder.new_block();
    builder.terminator(Terminator::Goto { target });
    
    builder.set_current_block(target);
    builder.terminator(Terminator::Return);
    
    let func = builder.build();
    assert_eq!(func.blocks.len(), 2);
}

#[test]
fn test_aggregate_kinds() {
    let tuple_agg = AggregateKind::Tuple;
    let array_agg = AggregateKind::Array(Type::Int);
    
    assert!(matches!(tuple_agg, AggregateKind::Tuple));
    assert!(matches!(array_agg, AggregateKind::Array(_)));
}

#[test]
fn test_projection_types() {
    let field_proj = Projection::Field(0);
    let index_proj = Projection::Index(LocalId(1));
    let deref_proj = Projection::Deref;
    
    assert!(matches!(field_proj, Projection::Field(_)));
    assert!(matches!(index_proj, Projection::Index(_)));
    assert!(matches!(deref_proj, Projection::Deref));
}