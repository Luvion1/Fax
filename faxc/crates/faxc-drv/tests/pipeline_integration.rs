//! MIR → LIR → CodeGen Pipeline Integration Tests
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 4
//! End-to-end integration tests for the full compiler pipeline.

#[cfg(test)]
mod pipeline_tests {
    use inkwell::context::Context;
    use inkwell::OptimizationLevel;

    #[test]
    fn test_mir_to_lir_lowering() {
        use faxc_lir::lower_mir_to_lir;
        use faxc_mir::{
            BasicBlock, BlockId, Builder, Constant, ConstantKind, Function as MirFunction, LocalId,
            Operand, Rvalue, Statement, Terminator,
        };
        use faxc_sem::Type;
        use faxc_util::Symbol;

        // Create a simple MIR function: fn test() -> Int { 42 }
        let mut builder = Builder::new(Symbol::intern("test"), Type::Int);
        let entry = builder.new_block();
        builder.set_current_block(entry);

        let local = builder.add_local(Type::Int, None);
        builder.assign(
            faxc_mir::Place::Local(local),
            Rvalue::Use(Operand::Constant(Constant {
                ty: Type::Int,
                kind: ConstantKind::Int(42),
            })),
        );
        builder.terminator(Terminator::Return);

        let mir_func = builder.build();

        // Lower to LIR
        let lir_func = lower_mir_to_lir(&mir_func);

        assert_eq!(lir_func.name, Symbol::intern("test"));
        assert!(lir_func.instruction_count() > 0);
    }

    #[test]
    fn test_lir_to_llvm_codegen() {
        use faxc_gen::LlvmBackend;
        use faxc_lir::{Function as LirFunction, Instruction, Operand, VirtualRegister};
        use faxc_util::Symbol;

        // Create a simple LIR function
        let mut lir_func = LirFunction::new(Symbol::intern("simple"));
        lir_func.instructions.push(Instruction::Label {
            name: ".Lbb0".to_string(),
        });
        lir_func.instructions.push(Instruction::Ret { value: None });

        // Generate LLVM IR
        let context = Context::create();
        let mut backend = LlvmBackend::new(
            &context,
            "test_module",
            "x86_64-unknown-linux-gnu".to_string(),
            OptimizationLevel::None,
        );

        let func = backend.compile_function(&lir_func);
        assert_eq!(func.get_name().to_str(), Ok("simple"));
    }

    #[test]
    fn test_full_pipeline_constant() {
        use faxc_gen::LlvmBackend;
        use faxc_lir::lower_mir_to_lir;
        use faxc_mir::{
            BasicBlock, BlockId, Builder, Constant, ConstantKind, Function as MirFunction, LocalId,
            Operand, Rvalue, Statement, Terminator,
        };
        use faxc_sem::Type;
        use faxc_util::Symbol;

        // Stage 1: Create MIR
        let mut builder = Builder::new(Symbol::intern("main"), Type::Int);
        let entry = builder.new_block();
        builder.set_current_block(entry);

        let local = builder.add_local(Type::Int, None);
        builder.assign(
            faxc_mir::Place::Local(local),
            Rvalue::Use(Operand::Constant(Constant {
                ty: Type::Int,
                kind: ConstantKind::Int(100),
            })),
        );
        builder.terminator(Terminator::Return);

        let mir_func = builder.build();

        // Stage 2: Lower to LIR
        let lir_func = lower_mir_to_lir(&mir_func);

        // Stage 3: Generate LLVM IR
        let context = Context::create();
        let mut backend = LlvmBackend::new(
            &context,
            "pipeline_test",
            "x86_64-unknown-linux-gnu".to_string(),
            OptimizationLevel::None,
        );

        let func = backend.compile_function(&lir_func);
        assert_eq!(func.get_name().to_str(), Ok("main"));

        // Verify LLVM IR is valid
        let ir = backend.emit_llvm_ir();
        assert!(ir.contains("define"));
        assert!(ir.contains("main"));
    }

    #[test]
    fn test_pipeline_with_binary_op() {
        use faxc_gen::LlvmBackend;
        use faxc_lir::lower_mir_to_lir;
        use faxc_mir::{
            BinOp, Builder, Constant, ConstantKind, Operand, Rvalue, Statement, Terminator,
        };
        use faxc_sem::Type;
        use faxc_util::Symbol;

        // Create MIR: fn add() -> Int { 10 + 20 }
        let mut builder = Builder::new(Symbol::intern("add"), Type::Int);
        let entry = builder.new_block();
        builder.set_current_block(entry);

        let local = builder.add_local(Type::Int, None);
        builder.assign(
            faxc_mir::Place::Local(local),
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
        );
        builder.terminator(Terminator::Return);

        let mir_func = builder.build();

        // Lower to LIR
        let lir_func = lower_mir_to_lir(&mir_func);

        // Generate LLVM IR
        let context = Context::create();
        let mut backend = LlvmBackend::new(
            &context,
            "add_test",
            "x86_64-unknown-linux-gnu".to_string(),
            OptimizationLevel::None,
        );

        let func = backend.compile_function(&lir_func);
        assert_eq!(func.get_name().to_str(), Ok("add"));
    }

    #[test]
    fn test_pipeline_with_control_flow() {
        use faxc_gen::LlvmBackend;
        use faxc_lir::lower_mir_to_lir;
        use faxc_mir::{Builder, Constant, ConstantKind, Operand, Rvalue, Statement, Terminator};
        use faxc_sem::Type;
        use faxc_util::Symbol;

        // Create MIR with if-then-else
        let mut builder = Builder::new(Symbol::intern("conditional"), Type::Int);

        let entry = builder.new_block();
        builder.set_current_block(entry);

        let then_block = builder.new_block();
        let else_block = builder.new_block();
        let join_block = builder.new_block();

        // if (true) then ... else ...
        builder.terminator(Terminator::If {
            cond: Operand::Constant(Constant {
                ty: Type::Bool,
                kind: ConstantKind::Bool(true),
            }),
            then_block,
            else_block,
        });

        // Then branch
        builder.set_current_block(then_block);
        builder.terminator(Terminator::Goto { target: join_block });

        // Else branch
        builder.set_current_block(else_block);
        builder.terminator(Terminator::Goto { target: join_block });

        // Join block
        builder.set_current_block(join_block);
        builder.terminator(Terminator::Return);

        let mir_func = builder.build();

        // Lower to LIR
        let lir_func = lower_mir_to_lir(&mir_func);

        // Generate LLVM IR
        let context = Context::create();
        let mut backend = LlvmBackend::new(
            &context,
            "cf_test",
            "x86_64-unknown-linux-gnu".to_string(),
            OptimizationLevel::None,
        );

        let func = backend.compile_function(&lir_func);
        assert_eq!(func.get_name().to_str(), Ok("conditional"));

        // Verify multiple basic blocks in LLVM IR
        let ir = backend.emit_llvm_ir();
        assert!(ir.contains("if"));
        assert!(ir.contains("else") || ir.contains("cont"));
    }

    #[test]
    fn test_pipeline_optimization_constant_folding() {
        use faxc_gen::LlvmBackend;
        use faxc_lir::lower_mir_to_lir;
        use faxc_mir::{optimize::optimize_function, BinOp, Builder};
        use faxc_sem::Type;
        use faxc_util::Symbol;

        // Create MIR with constant expression: 5 * 6
        let mut builder = Builder::new(Symbol::intern("opt_test"), Type::Int);
        let entry = builder.new_block();
        builder.set_current_block(entry);

        let local = builder.add_local(Type::Int, None);
        builder.assign(
            faxc_mir::Place::Local(local),
            Rvalue::BinaryOp(
                BinOp::Mul,
                Box::new(faxc_mir::Operand::Constant(faxc_mir::Constant {
                    ty: Type::Int,
                    kind: faxc_mir::ConstantKind::Int(5),
                })),
                Box::new(faxc_mir::Operand::Constant(faxc_mir::Constant {
                    ty: Type::Int,
                    kind: faxc_mir::ConstantKind::Int(6),
                })),
            ),
        );
        builder.terminator(Terminator::Return);

        let mut mir_func = builder.build();

        // Apply optimizations
        optimize_function(&mut mir_func);

        // Lower to LIR
        let lir_func = lower_mir_to_lir(&mir_func);

        // Generate LLVM IR
        let context = Context::create();
        let mut backend = LlvmBackend::new(
            &context,
            "opt_test",
            "x86_64-unknown-linux-gnu".to_string(),
            OptimizationLevel::None,
        );

        let func = backend.compile_function(&lir_func);
        assert_eq!(func.get_name().to_str(), Ok("opt_test"));
    }

    #[test]
    fn test_type_mapping_in_pipeline() {
        use faxc_gen::TypeMapper;
        use faxc_sem::Type;

        let context = Context::create();
        let mapper = TypeMapper::new(&context);

        // Verify all primitive types map correctly
        assert_eq!(
            mapper
                .map_to_basic(&Type::Int)
                .into_int_type()
                .get_bit_width(),
            64
        );
        assert_eq!(
            mapper
                .map_to_basic(&Type::Int8)
                .into_int_type()
                .get_bit_width(),
            8
        );
        assert_eq!(
            mapper
                .map_to_basic(&Type::Bool)
                .into_int_type()
                .get_bit_width(),
            1
        );
        assert!(mapper.map_to_basic(&Type::Float).is_float_type());
    }

    #[test]
    fn test_abi_compliance() {
        use faxc_lir::calling_convention::SystemVAbi;
        use faxc_lir::stack_frame::{ParamAssignment, StackFrame};

        // Verify System V AMD64 ABI compliance
        assert_eq!(SystemVAbi::get_arg_register(0).is_some(), true);
        assert_eq!(SystemVAbi::get_arg_register(5).is_some(), true);
        assert_eq!(SystemVAbi::get_arg_register(6).is_none(), true);

        // Verify stack frame alignment
        let mut frame = StackFrame::new();
        frame.frame_size(10, 5, true);
        assert_eq!(frame.frame_size % 16, 0);
    }
}
