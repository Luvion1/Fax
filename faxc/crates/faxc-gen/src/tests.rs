//! CodeGen Crate Integration Tests
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 3
//! Unit and integration tests for LLVM IR generation, type mapping, and code emission.

use crate::*;
use inkwell::context::Context;
use inkwell::OptimizationLevel;

#[test]
fn test_llvm_backend_creation() {
    let context = Context::create();
    let backend = LlvmBackend::new(
        &context,
        "test_module",
        "x86_64-unknown-linux-gnu".to_string(),
        OptimizationLevel::None,
    );

    assert_eq!(backend.target_triple, "x86_64-unknown-linux-gnu");
    assert_eq!(backend.opt_level, OptimizationLevel::None);
}

#[test]
fn test_emit_empty_module() {
    let context = Context::create();
    let mut backend = LlvmBackend::new(
        &context,
        "test",
        "x86_64-unknown-linux-gnu".to_string(),
        OptimizationLevel::None,
    );

    let ir = backend.emit_llvm_ir();

    assert!(ir.contains("target triple"));
    assert!(ir.contains("x86_64-unknown-linux-gnu"));
}

#[test]
fn test_type_mapper_int_types() {
    let context = Context::create();
    let mapper = TypeMapper::new(&context);

    // Test Int -> i64
    let int_ty = faxc_sem::Type::Int;
    let llvm_ty = mapper.map_to_basic(&int_ty);
    assert_eq!(llvm_ty.into_int_type().get_bit_width(), 64);

    // Test Int8 -> i8
    let int8_ty = faxc_sem::Type::Int8;
    let llvm_ty = mapper.map_to_basic(&int8_ty);
    assert_eq!(llvm_ty.into_int_type().get_bit_width(), 8);

    // Test Int16 -> i16
    let int16_ty = faxc_sem::Type::Int16;
    let llvm_ty = mapper.map_to_basic(&int16_ty);
    assert_eq!(llvm_ty.into_int_type().get_bit_width(), 16);

    // Test Int32 -> i32
    let int32_ty = faxc_sem::Type::Int32;
    let llvm_ty = mapper.map_to_basic(&int32_ty);
    assert_eq!(llvm_ty.into_int_type().get_bit_width(), 32);
}

#[test]
fn test_type_mapper_unsigned_types() {
    let context = Context::create();
    let mapper = TypeMapper::new(&context);

    // Test UInt -> i64 (unsigned 64-bit)
    let uint_ty = faxc_sem::Type::UInt;
    let llvm_ty = mapper.map_to_basic(&uint_ty);
    assert_eq!(llvm_ty.into_int_type().get_bit_width(), 64);

    // Test UInt8 -> i8
    let uint8_ty = faxc_sem::Type::UInt8;
    let llvm_ty = mapper.map_to_basic(&uint8_ty);
    assert_eq!(llvm_ty.into_int_type().get_bit_width(), 8);

    // Test UInt16 -> i16
    let uint16_ty = faxc_sem::Type::UInt16;
    let llvm_ty = mapper.map_to_basic(&uint16_ty);
    assert_eq!(llvm_ty.into_int_type().get_bit_width(), 16);

    // Test UInt32 -> i32
    let uint32_ty = faxc_sem::Type::UInt32;
    let llvm_ty = mapper.map_to_basic(&uint32_ty);
    assert_eq!(llvm_ty.into_int_type().get_bit_width(), 32);
}

#[test]
fn test_type_mapper_float_types() {
    let context = Context::create();
    let mapper = TypeMapper::new(&context);

    // Test Float -> double
    let float_ty = faxc_sem::Type::Float;
    let llvm_ty = mapper.map_to_basic(&float_ty);
    assert!(llvm_ty.is_float_type());
    assert_eq!(llvm_ty.into_float_type().get_bit_width(), 64);

    // Test Float32 -> float
    let float32_ty = faxc_sem::Type::Float32;
    let llvm_ty = mapper.map_to_basic(&float32_ty);
    assert!(llvm_ty.is_float_type());
    assert_eq!(llvm_ty.into_float_type().get_bit_width(), 32);
}

#[test]
fn test_type_mapper_bool_type() {
    let context = Context::create();
    let mapper = TypeMapper::new(&context);

    let bool_ty = faxc_sem::Type::Bool;
    let llvm_ty = mapper.map_to_basic(&bool_ty);
    assert_eq!(llvm_ty.into_int_type().get_bit_width(), 1);
}

#[test]
fn test_type_mapper_array_type() {
    let context = Context::create();
    let mapper = TypeMapper::new(&context);

    let array_ty = faxc_sem::Type::Array(Box::new(faxc_sem::Type::Int), 10);
    let llvm_ty = mapper.map_to_basic(&array_ty);
    assert!(llvm_ty.is_array_type());
}

#[test]
fn test_type_mapper_pointer_type() {
    let context = Context::create();
    let mapper = TypeMapper::new(&context);

    let ptr_ty = faxc_sem::Type::Pointer(Box::new(faxc_sem::Type::Int));
    let llvm_ty = mapper.map_to_basic(&ptr_ty);
    assert!(llvm_ty.is_pointer_type());
}

#[test]
fn test_type_mapper_reference_type() {
    let context = Context::create();
    let mapper = TypeMapper::new(&context);

    // Test immutable reference
    let ref_ty = faxc_sem::Type::Ref(Box::new(faxc_sem::Type::Int), false);
    let llvm_ty = mapper.map_to_basic(&ref_ty);
    assert!(llvm_ty.is_pointer_type());

    // Test mutable reference
    let mut_ref_ty = faxc_sem::Type::Ref(Box::new(faxc_sem::Type::Int), true);
    let llvm_mut_ty = mapper.map_to_basic(&mut_ref_ty);
    assert!(llvm_mut_ty.is_pointer_type());
}

#[test]
fn test_type_mapper_function_type() {
    let context = Context::create();
    let mapper = TypeMapper::new(&context);

    // Test fn(i32) -> i64
    let fn_ty = faxc_sem::Type::Fn(vec![faxc_sem::Type::Int32], Box::new(faxc_sem::Type::Int));
    let llvm_ty = mapper.map_to_basic(&fn_ty);
    assert!(llvm_ty.is_function_type());
}

#[test]
fn test_type_size_calculations() {
    let context = Context::create();
    let mapper = TypeMapper::new(&context);

    assert_eq!(mapper.size_of(&faxc_sem::Type::Int8), 1);
    assert_eq!(mapper.size_of(&faxc_sem::Type::Int16), 2);
    assert_eq!(mapper.size_of(&faxc_sem::Type::Int32), 4);
    assert_eq!(mapper.size_of(&faxc_sem::Type::Int), 8);
    assert_eq!(mapper.size_of(&faxc_sem::Type::Float), 8);
}

#[test]
fn test_type_alignment_calculations() {
    let context = Context::create();
    let mapper = TypeMapper::new(&context);

    assert_eq!(mapper.alignment_of(&faxc_sem::Type::Int8), 1);
    assert_eq!(mapper.alignment_of(&faxc_sem::Type::Int16), 2);
    assert_eq!(mapper.alignment_of(&faxc_sem::Type::Int32), 4);
    assert_eq!(mapper.alignment_of(&faxc_sem::Type::Int), 8);
}

#[test]
fn test_compile_lir_function() {
    use faxc_lir::{Function as LirFunction, Instruction, Operand, VirtualRegister};

    let context = Context::create();
    let mut backend = LlvmBackend::new(
        &context,
        "test",
        "x86_64-unknown-linux-gnu".to_string(),
        OptimizationLevel::None,
    );

    // Create a simple LIR function
    let mut lir_func = LirFunction::new(faxc_util::Symbol::intern("simple_fn"));
    lir_func.instructions.push(Instruction::Label {
        name: ".Lbb0".to_string(),
    });
    lir_func.instructions.push(Instruction::Ret { value: None });

    let func_val = backend.compile_function(&lir_func);
    assert_eq!(func_val.get_name().to_str(), Ok("simple_fn"));
}

#[test]
fn test_write_ir_to_file() {
    use std::fs;
    use std::path::PathBuf;

    let context = Context::create();
    let mut backend = LlvmBackend::new(
        &context,
        "test",
        "x86_64-unknown-linux-gnu".to_string(),
        OptimizationLevel::None,
    );

    let temp_path = PathBuf::from("/tmp/test_faxc_ir.ll");
    let result = backend.write_ir_to_file(&temp_path);

    assert!(result.is_ok());
    assert!(temp_path.exists());

    // Cleanup
    let _ = fs::remove_file(&temp_path);
}

#[test]
fn test_optimization_levels() {
    let context = Context::create();

    let backend_none = LlvmBackend::new(
        &context,
        "test",
        "x86_64".to_string(),
        OptimizationLevel::None,
    );
    assert_eq!(backend_none.opt_level, OptimizationLevel::None);

    let backend_default = LlvmBackend::new(
        &context,
        "test",
        "x86_64".to_string(),
        OptimizationLevel::Default,
    );
    assert_eq!(backend_default.opt_level, OptimizationLevel::Default);
}
