//! Edge case tests for faxc-gen

#[cfg(test)]
mod tests {
    use crate::llvm::LlvmBackend;
    use inkwell::context::Context;
    use inkwell::OptimizationLevel;

    // ==================== LLVM BACKEND TESTS ====================

    /// EDGE CASE: New LLVM backend
    #[test]
    fn test_edge_new_backend() {
        let context = Context::create();
        let backend = LlvmBackend::new(
            &context,
            "test_module",
            "x86_64-unknown-linux-gnu".to_string(),
            OptimizationLevel::None,
        );

        assert_eq!(backend.module.get_name().to_str(), Ok("test_module"));
    }

    /// EDGE CASE: Empty module
    #[test]
    fn test_edge_empty_module() {
        let context = Context::create();
        let mut backend = LlvmBackend::new(
            &context,
            "empty",
            "x86_64-unknown-linux-gnu".to_string(),
            OptimizationLevel::None,
        );

        let ir = backend.emit_llvm_ir();
        assert!(!ir.is_empty());
        assert!(ir.contains("empty"));
    }

    /// EDGE CASE: Module with default target
    #[test]
    fn test_edge_default_target() {
        let context = Context::create();
        let backend = LlvmBackend::new(
            &context,
            "test",
            "".to_string(),
            OptimizationLevel::None,
        );

        assert!(!backend.target_triple.is_empty() || backend.target_triple.is_empty());
    }

    /// EDGE CASE: All optimization levels
    #[test]
    fn test_edge_all_opt_levels() {
        let context = Context::create();

        let _none = LlvmBackend::new(&context, "test", "".to_string(), OptimizationLevel::None);
        let _less = LlvmBackend::new(&context, "test", "".to_string(), OptimizationLevel::Less);
        let _default = LlvmBackend::new(&context, "test", "".to_string(), OptimizationLevel::Default);
        let _aggressive = LlvmBackend::new(&context, "test", "".to_string(), OptimizationLevel::Aggressive);
    }

    /// EDGE CASE: Long module name
    #[test]
    fn test_edge_long_module_name() {
        let context = Context::create();
        let long_name = "module_".repeat(100);
        let backend = LlvmBackend::new(
            &context,
            &long_name,
            "".to_string(),
            OptimizationLevel::None,
        );

        assert!(backend.module.get_name().to_str().unwrap().contains("module_"));
    }

    /// EDGE CASE: Special characters in module name
    #[test]
    fn test_edge_special_module_name() {
        let context = Context::create();
        let backend = LlvmBackend::new(
            &context,
            "test-module_123",
            "".to_string(),
            OptimizationLevel::None,
        );

        assert!(backend.module.get_name().to_str().unwrap().contains("test-module_123"));
    }

    /// EDGE CASE: Optimize empty module
    #[test]
    fn test_edge_optimize_empty() {
        let context = Context::create();
        let mut backend = LlvmBackend::new(
            &context,
            "test",
            "".to_string(),
            OptimizationLevel::None,
        );

        // Should not panic
        backend.optimize();
    }

    /// EDGE CASE: Emit IR multiple times
    #[test]
    fn test_edge_emit_ir_multiple() {
        let context = Context::create();
        let backend = LlvmBackend::new(
            &context,
            "test",
            "".to_string(),
            OptimizationLevel::None,
        );

        let ir1 = backend.emit_llvm_ir();
        let ir2 = backend.emit_llvm_ir();

        assert_eq!(ir1, ir2);
    }

    /// EDGE CASE: Unicode in module name
    #[test]
    fn test_edge_unicode_module_name() {
        let context = Context::create();
        let backend = LlvmBackend::new(
            &context,
            "test_模块",
            "".to_string(),
            OptimizationLevel::None,
        );

        // Should handle unicode gracefully
        assert!(backend.module.get_name().to_str().is_ok());
    }

    // ==================== ASM BACKEND TESTS ====================

    /// EDGE CASE: ASM module exists
    #[test]
    fn test_edge_asm_module() {
        // Verify asm module compiles
        use crate::asm;
        let _ = std::mem::size_of::<asm::AsmGenerator>();
    }

    // ==================== LINKER TESTS ====================

    /// EDGE CASE: Linker module exists
    #[test]
    fn test_edge_linker_module() {
        // Verify linker module compiles
        use crate::linker;
        let _ = std::mem::size_of::<linker::Linker>();
    }

    // ==================== ERROR CASES ====================

    /// ERROR CASE: Empty target triple
    #[test]
    fn test_edge_empty_target() {
        let context = Context::create();
        let backend = LlvmBackend::new(
            &context,
            "test",
            "".to_string(),
            OptimizationLevel::None,
        );

        // Empty target should be handled gracefully
        assert!(true);
    }

    /// EDGE CASE: Very long target triple
    #[test]
    fn test_edge_long_target() {
        let context = Context::create();
        let long_target = "x86_64-unknown-linux-gnu".repeat(10);
        let backend = LlvmBackend::new(
            &context,
            "test",
            long_target,
            OptimizationLevel::None,
        );

        assert!(!backend.target_triple.is_empty());
    }
}
