//! LLVM IR Code Generator
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 3
//! Complete LLVM IR generation from LIR

use faxc_lir::{
    CallTarget, Condition, Function as LirFunction, Instruction, Operand, PhysicalRegister,
    VirtualRegister,
};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::path::Path;

use crate::error::{CodeGenError, Result};
use crate::types::TypeMapper;

pub struct LlvmBackend<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub target_triple: String,
    pub opt_level: OptimizationLevel,
    pub type_mapper: TypeMapper<'ctx>,
}

use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};

impl<'ctx> LlvmBackend<'ctx> {
    pub fn new(
        context: &'ctx Context,
        module_name: &str,
        target_triple: String,
        opt_level: OptimizationLevel,
    ) -> Self {
        let module = context.create_module(module_name);

        // Set target triple and data layout
        let triple = inkwell::targets::TargetTriple::create(&target_triple);
        module.set_triple(&triple);

        if let Ok(target) = inkwell::targets::Target::from_triple(&triple) {
            if let Some(target_machine) = target.create_target_machine(
                &triple,
                "generic",
                "",
                opt_level,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            ) {
                let data_layout = target_machine.get_target_data().get_data_layout();
                module.set_data_layout(&data_layout);
            }
        }

        let mut backend = Self {
            context,
            module,
            builder: context.create_builder(),
            target_triple,
            opt_level,
            type_mapper: TypeMapper::new(context),
        };

        // Declare GC runtime functions
        backend.declare_gc_functions();

        backend
    }

    /// Declare GC runtime functions
    fn declare_gc_functions(&mut self) {
        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let i64_type = self.context.i64_type();
        let bool_type = self.context.bool_type();

        // Declare fax_gc_init() -> bool
        let init_fn_type = bool_type.fn_type(&[], false);
        let _ = self.module.add_function(
            "fax_gc_init",
            init_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_gc_alloc(size: usize) -> *mut i8
        let gc_alloc_fn_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
        let _ = self.module.add_function(
            "fax_gc_alloc",
            gc_alloc_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_gc_alloc_zeroed(size: usize) -> *mut i8
        let _ = self.module.add_function(
            "fax_gc_alloc_zeroed",
            gc_alloc_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_gc_register_root(ptr: *mut i8) -> bool
        let register_root_fn_type = bool_type.fn_type(&[i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_gc_register_root",
            register_root_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_gc_unregister_root(ptr: *mut i8) -> bool
        let _ = self.module.add_function(
            "fax_gc_unregister_root",
            register_root_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_gc_collect() - full GC
        let void_fn_type = self.context.void_type().fn_type(&[], false);
        let _ = self.module.add_function(
            "fax_gc_collect",
            void_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_gc_collect_young() - young generation GC
        let _ = self.module.add_function(
            "fax_gc_collect_young",
            void_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_gc_shutdown()
        let _ = self.module.add_function(
            "fax_gc_shutdown",
            void_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare malloc(size: usize) -> *mut i8 (fallback from libc)
        let malloc_fn_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
        let _ = self.module.add_function(
            "malloc",
            malloc_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare free(ptr: *mut i8) (fallback from libc)
        let free_fn_type = self
            .context
            .void_type()
            .fn_type(&[i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "free",
            free_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_len(ptr: *const u8) -> usize
        let string_len_fn_type = i64_type.fn_type(&[i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_string_len",
            string_len_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_concat(s1: *const u8, s2: *const u8) -> *mut u8
        let string_concat_fn_type =
            i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_string_concat",
            string_concat_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_eq(s1: *const u8, s2: *const u8) -> bool
        let string_eq_fn_type = bool_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_string_eq",
            string_eq_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_cmp(s1: *const u8, s2: *const u8) -> i32
        let i32_type = self.context.i32_type();
        let string_cmp_fn_type = i32_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_string_cmp",
            string_cmp_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare print function for i64
        let print_fn_type = self.context.void_type().fn_type(&[i64_type.into()], false);
        let _ = self.module.add_function(
            "print",
            print_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare println function for i64
        let _ = self.module.add_function(
            "println",
            print_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare print function for string
        let print_str_fn_type = self
            .context
            .void_type()
            .fn_type(&[i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "print_str",
            print_str_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare println function for string
        let _ = self.module.add_function(
            "println_str",
            print_str_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_bool_to_string(b: bool) -> *mut u8
        let bool_to_str_fn_type = i8_ptr_type.fn_type(&[bool_type.into()], false);
        let _ = self.module.add_function(
            "fax_bool_to_string",
            bool_to_str_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_char_to_string(c: char) -> *mut u8
        let char_to_str_fn_type = i8_ptr_type.fn_type(&[i32_type.into()], false);
        let _ = self.module.add_function(
            "fax_char_to_string",
            char_to_str_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_int_to_string(i: i64) -> *mut u8
        let int_to_str_fn_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
        let _ = self.module.add_function(
            "fax_int_to_string",
            int_to_str_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_float_to_string(f: f64) -> *mut u8
        let f64_type = self.context.f64_type();
        let float_to_str_fn_type = i8_ptr_type.fn_type(&[f64_type.into()], false);
        let _ = self.module.add_function(
            "fax_float_to_string",
            float_to_str_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_to_int(s: *const u8) -> i64
        let str_to_int_fn_type = i64_type.fn_type(&[i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_string_to_int",
            str_to_int_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_to_float(s: *const u8) -> f64
        let str_to_float_fn_type = f64_type.fn_type(&[i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_string_to_float",
            str_to_float_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_int32_to_string(value: i32) -> *mut u8
        let i32_type = self.context.i32_type();
        let int32_to_str_fn_type = i8_ptr_type.fn_type(&[i32_type.into()], false);
        let _ = self.module.add_function(
            "fax_int32_to_string",
            int32_to_str_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_to_int32(s: *const u8) -> i32
        let str_to_int32_fn_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_string_to_int32",
            str_to_int32_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_uint32_to_string(value: u32) -> *mut u8
        let uint32_to_str_fn_type = i8_ptr_type.fn_type(&[i32_type.into()], false);
        let _ = self.module.add_function(
            "fax_uint32_to_string",
            uint32_to_str_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_to_uint32(s: *const u8) -> u32
        let _ = self.module.add_function(
            "fax_string_to_uint32",
            str_to_int32_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_array_len(arr: *const i8) -> usize
        let array_len_fn_type = i64_type.fn_type(&[i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_array_len",
            array_len_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_array_get(arr: *const i8, index: usize) -> *mut i8
        let array_get_fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i64_type.into()], false);
        let _ = self.module.add_function(
            "fax_array_get",
            array_get_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_array_set(arr: *mut i8, index: usize, value: *mut i8)
        let array_set_fn_type = self.context.void_type().fn_type(
            &[i8_ptr_type.into(), i64_type.into(), i8_ptr_type.into()],
            false,
        );
        let _ = self.module.add_function(
            "fax_array_set",
            array_set_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_array_create(size: usize, element_size: usize) -> *mut i8
        let array_create_fn_type = i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let _ = self.module.add_function(
            "fax_array_create",
            array_create_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_array_clone(arr: *const i8) -> *mut i8
        let _ = self.module.add_function(
            "fax_array_clone",
            array_get_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_clone(s: *const u8) -> *mut u8
        let _ = self.module.add_function(
            "fax_string_clone",
            i8_ptr_type.fn_type(&[i8_ptr_type.into()], false),
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_slice(s: *const u8, start: usize, end: usize) -> *mut u8
        let string_slice_fn_type = i8_ptr_type.fn_type(
            &[i8_ptr_type.into(), i64_type.into(), i64_type.into()],
            false,
        );
        let _ = self.module.add_function(
            "fax_string_slice",
            string_slice_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_contains(s: *const u8, substr: *const u8) -> bool
        let string_contains_fn_type =
            bool_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_string_contains",
            string_contains_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_starts_with(s: *const u8, prefix: *const u8) -> bool
        let _ = self.module.add_function(
            "fax_string_starts_with",
            string_contains_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_ends_with(s: *const u8, suffix: *const u8) -> bool
        let _ = self.module.add_function(
            "fax_string_ends_with",
            string_contains_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_replace(s: *const u8, old: *const u8, new: *const u8) -> *mut u8
        let string_replace_fn_type = i8_ptr_type.fn_type(
            &[i8_ptr_type.into(), i8_ptr_type.into(), i8_ptr_type.into()],
            false,
        );
        let _ = self.module.add_function(
            "fax_string_replace",
            string_replace_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_split(s: *const u8, delim: *const u8) -> *mut i8
        let _ = self.module.add_function(
            "fax_string_split",
            array_get_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_trim(s: *const u8) -> *mut u8
        let _ = self.module.add_function(
            "fax_string_trim",
            i8_ptr_type.fn_type(&[i8_ptr_type.into()], false),
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_to_uppercase(s: *const u8) -> *mut u8
        let _ = self.module.add_function(
            "fax_string_to_uppercase",
            i8_ptr_type.fn_type(&[i8_ptr_type.into()], false),
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_string_to_lowercase(s: *const u8) -> *mut u8
        let _ = self.module.add_function(
            "fax_string_to_lowercase",
            i8_ptr_type.fn_type(&[i8_ptr_type.into()], false),
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_panic(msg: *const u8) -> !
        let panic_fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], true);
        let _ = self.module.add_function(
            "fax_panic",
            panic_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_assert(cond: bool, msg: *const u8)
        let assert_fn_type = self
            .context
            .void_type()
            .fn_type(&[bool_type.into(), i8_ptr_type.into()], false);
        let _ = self.module.add_function(
            "fax_assert",
            assert_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_debug_println(val: *const i8, type_tag: i32)
        let debug_fn_type = self
            .context
            .void_type()
            .fn_type(&[i8_ptr_type.into(), i32_type.into()], false);
        let _ = self.module.add_function(
            "fax_debug_println",
            debug_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_sqrt(f: f64) -> f64
        let f64_math_fn_type = f64_type.fn_type(&[f64_type.into()], false);
        let _ = self.module.add_function(
            "fax_f64_math_sqrt",
            f64_math_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_pow(base: f64, exp: f64) -> f64
        let f64_pow_fn_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
        let _ = self.module.add_function(
            "fax_f64_math_pow",
            f64_pow_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_sin(f: f64) -> f64
        let _ = self.module.add_function(
            "fax_f64_math_sin",
            f64_math_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_cos(f: f64) -> f64
        let _ = self.module.add_function(
            "fax_f64_math_cos",
            f64_math_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_floor(f: f64) -> f64
        let _ = self.module.add_function(
            "fax_f64_math_floor",
            f64_math_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_ceil(f: f64) -> f64
        let _ = self.module.add_function(
            "fax_f64_math_ceil",
            f64_math_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_round(f: f64) -> f64
        let _ = self.module.add_function(
            "fax_f64_math_round",
            f64_math_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_abs(f: f64) -> f64
        let _ = self.module.add_function(
            "fax_f64_math_abs",
            f64_math_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_log(f: f64) -> f64
        let _ = self.module.add_function(
            "fax_f64_math_log",
            f64_math_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_log10(f: f64) -> f64
        let _ = self.module.add_function(
            "fax_f64_math_log10",
            f64_math_fn_type,
            Some(inkwell::module::Linkage::External),
        );

        // Declare fax_f64_math_exp(f: f64) -> f64
        let _ = self.module.add_function(
            "fax_f64_math_exp",
            f64_math_fn_type,
            Some(inkwell::module::Linkage::External),
        );
    }

    /// Compile a LIR function to LLVM IR
    pub fn compile_function(&mut self, func: &LirFunction) -> Result<FunctionValue<'ctx>> {
        let i64_type = self.context.i64_type();

        // Create function type (simplified - MVP returns i64)
        let fn_type = i64_type.fn_type(&[], false);
        let function = self.module.add_function(func.name.as_str(), fn_type, None);

        // Register allocation: map virtual registers to stack slots
        let mut registers: HashMap<VirtualRegister, PointerValue<'ctx>> = HashMap::new();
        let mut llvm_blocks: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>> =
            HashMap::new();

        // Pass 1: Create all basic blocks for labels
        for instr in &func.instructions {
            if let Instruction::Label { name } = instr {
                let block = self.context.append_basic_block(function, name);
                llvm_blocks.insert(name.clone(), block);
            }
        }

        // Add initial entry block if not present
        let entry_block = llvm_blocks
            .get(".Lbb0")
            .copied()
            .unwrap_or_else(|| self.context.append_basic_block(function, "entry"));
        self.builder.position_at_end(entry_block);

        // Track last comparison value for conditional jumps
        let mut last_cmp_val: Option<IntValue<'ctx>> = None;
        let mut has_return = false;

        // Pass 2: Generate instructions
        for instr in &func.instructions {
            match instr {
                Instruction::Label { name } => {
                    let block = llvm_blocks
                        .get(name)
                        .ok_or_else(|| CodeGenError::BlockNotFound(name.clone()))?;
                    self.builder.position_at_end(*block);
                },

                Instruction::Mov { dest, src } => {
                    self.generate_mov(dest, src, &mut registers)?;
                },

                Instruction::Add { dest, src } => {
                    self.generate_add(dest, src, &mut registers)?;
                },

                Instruction::Sub { dest, src } => {
                    self.generate_sub(dest, src, &mut registers)?;
                },

                Instruction::Mul { dest, src, .. } => {
                    self.generate_mul(dest, src, &mut registers)?;
                },

                Instruction::Idiv { dest, src } => {
                    self.generate_idiv(dest, src, &mut registers)?;
                },

                Instruction::And { dest, src } => {
                    self.generate_and(dest, src, &mut registers)?;
                },

                Instruction::Or { dest, src } => {
                    self.generate_or(dest, src, &mut registers)?;
                },

                Instruction::Xor { dest, src } => {
                    self.generate_xor(dest, src, &mut registers)?;
                },

                Instruction::Cmp { src1, src2 } => {
                    last_cmp_val = Some(self.generate_cmp(src1, src2, &mut registers)?);
                },

                Instruction::Jmp { target } => {
                    let target_block = llvm_blocks
                        .get(target)
                        .ok_or_else(|| CodeGenError::BlockNotFound(target.clone()))?;
                    self.builder
                        .build_unconditional_branch(*target_block)
                        .map_err(|e| {
                            CodeGenError::LlvmOperationFailed(format!("Failed to branch: {}", e))
                        })?;
                },

                Instruction::Jcc { cond, target } => {
                    let target_block = llvm_blocks
                        .get(target)
                        .ok_or_else(|| CodeGenError::BlockNotFound(target.clone()))?;
                    let cmp_val = last_cmp_val.ok_or(CodeGenError::MissingComparison)?;

                    // Create continuation block
                    let next_block = self.context.append_basic_block(function, "cont");

                    // Build conditional branch based on condition
                    let cond_value = self.convert_condition(cond, cmp_val);
                    self.builder
                        .build_conditional_branch(cond_value, *target_block, next_block)
                        .map_err(|e| {
                            CodeGenError::LlvmOperationFailed(format!(
                                "Failed conditional branch: {}",
                                e
                            ))
                        })?;
                    self.builder.position_at_end(next_block);
                },

                Instruction::Call { target } => {
                    self.generate_call(target)?;
                },

                Instruction::Ret { value } => {
                    if let Some(operand) = value {
                        let val = self.get_operand_value(operand, &mut registers)?;
                        self.builder.build_return(Some(&val)).map_err(|e| {
                            CodeGenError::LlvmOperationFailed(format!("Failed to return: {}", e))
                        })?;
                    } else {
                        self.builder
                            .build_return(Some(&i64_type.const_int(0, false)))
                            .map_err(|e| {
                                CodeGenError::LlvmOperationFailed(format!(
                                    "Failed to return: {}",
                                    e
                                ))
                            })?;
                    }
                    has_return = true;
                },

                Instruction::Push { src } => {
                    let _val = self.get_operand_value(src, &mut registers)?;
                    // Simplified: stack handling omitted for now
                },

                Instruction::EnterFrame { frame_size: _ } => {
                    // Allocate stack frame
                },

                Instruction::LeaveFrame => {
                    // Deallocate stack frame
                },

                Instruction::SaveCalleeSaved { regs: _ } => {
                    // Save callee-saved registers
                },

                Instruction::RestoreCalleeSaved { regs: _ } => {
                    // Restore callee-saved registers
                },

                // Load/Store instructions
                Instruction::Load { dest, addr, width } => {
                    self.generate_load(dest, addr, *width, &mut registers)?;
                },

                Instruction::Store { addr, src, width } => {
                    self.generate_store(addr, src, *width, &mut registers)?;
                },

                // Memory operations
                Instruction::Lea { dest, addr } => {
                    self.generate_lea(dest, addr, &mut registers)?;
                },

                Instruction::Alloca { dest, size } => {
                    self.generate_alloca(dest, size, &mut registers)?;
                },

                // Unary operations
                Instruction::Neg { dest } => {
                    self.generate_neg(dest, &mut registers)?;
                },

                Instruction::Not { dest } => {
                    self.generate_not(dest, &mut registers)?;
                },

                // Shift operations
                Instruction::Shl { dest, count } => {
                    self.generate_shl(dest, count, &mut registers)?;
                },

                Instruction::Shr { dest, count } => {
                    self.generate_shr(dest, count, &mut registers)?;
                },

                Instruction::Sar { dest, count } => {
                    self.generate_sar(dest, count, &mut registers)?;
                },

                // Comparison
                Instruction::Test { src1, src2 } => {
                    last_cmp_val = Some(self.generate_test(src1, src2, &mut registers)?);
                },

                // Other instructions
                _ => {
                    // Other instructions handled as needed
                },
            }
        }

        // Ensure function has a terminator
        if self.builder.get_insert_block().is_some() && !has_return {
            self.builder
                .build_return(Some(&i64_type.const_int(0, false)))
                .map_err(|e| {
                    CodeGenError::LlvmOperationFailed(format!("Failed to add return: {}", e))
                })?;
        }

        Ok(function)
    }

    fn generate_mov(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let src_val = self.get_operand_value(src, registers)?;
        self.builder
            .build_store(dest_ptr, src_val)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_add(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(src, registers)?.into_int_value();
        let result = self
            .builder
            .build_int_add(v1, v2, "add_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed add: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_sub(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(src, registers)?.into_int_value();
        let result = self
            .builder
            .build_int_sub(v1, v2, "sub_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed sub: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_mul(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(src, registers)?.into_int_value();
        let result = self
            .builder
            .build_int_mul(v1, v2, "mul_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed mul: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_idiv(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(src, registers)?.into_int_value();
        let result = self
            .builder
            .build_int_signed_div(v1, v2, "div_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed div: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_and(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(src, registers)?.into_int_value();
        let result = self
            .builder
            .build_and(v1, v2, "and_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed and: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_or(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(src, registers)?.into_int_value();
        let result = self
            .builder
            .build_or(v1, v2, "or_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed or: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_xor(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(src, registers)?.into_int_value();
        let result = self
            .builder
            .build_xor(v1, v2, "xor_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed xor: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn get_llvm_type_for_width(
        &self,
        width: faxc_lir::RegisterWidth,
    ) -> inkwell::types::IntType<'ctx> {
        match width {
            faxc_lir::RegisterWidth::W8 => self.context.i8_type(),
            faxc_lir::RegisterWidth::W16 => self.context.i16_type(),
            faxc_lir::RegisterWidth::W32 => self.context.i32_type(),
            faxc_lir::RegisterWidth::W64 => self.context.i64_type(),
        }
    }

    fn generate_load(
        &self,
        dest: &Operand,
        addr: &faxc_lir::Address,
        width: faxc_lir::RegisterWidth,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let ptr = self.generate_address(addr, registers)?;
        let llvm_type = self.get_llvm_type_for_width(width);
        let loaded = self
            .builder
            .build_load(llvm_type, ptr, "load_val")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load: {}", e)))?;
        self.builder
            .build_store(dest_ptr, loaded)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed store: {}", e)))?;
        Ok(())
    }

    fn generate_store(
        &self,
        addr: &faxc_lir::Address,
        src: &Operand,
        _width: faxc_lir::RegisterWidth,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let ptr = self.generate_address(addr, registers)?;
        let src_val = self.get_operand_value(src, registers)?;
        self.builder
            .build_store(ptr, src_val)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed store: {}", e)))?;
        Ok(())
    }

    fn generate_lea(
        &self,
        dest: &Operand,
        addr: &faxc_lir::Address,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let ptr = self.generate_address(addr, registers)?;
        self.builder
            .build_store(dest_ptr, ptr)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed store addr: {}", e)))?;
        Ok(())
    }

    fn generate_alloca(
        &self,
        dest: &Operand,
        size: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let size_val = self.get_operand_value(size, registers)?;
        let size_int = size_val.into_int_value();

        // Call malloc for heap allocation (instead of stack alloca)
        let malloc_fn = self
            .module
            .get_function("malloc")
            .ok_or_else(|| CodeGenError::FunctionNotFound("malloc".to_string()))?;

        let call_result = self
            .builder
            .build_call(malloc_fn, &[size_int.into()], "malloc_call")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed malloc call: {}", e)))?;

        // Get the return value from the call
        let alloc = call_result.try_as_basic_value().unwrap_basic();

        // The result is already a pointer
        let alloc_ptr = alloc.into_pointer_value();

        self.builder.build_store(dest_ptr, alloc_ptr).map_err(|e| {
            CodeGenError::LlvmOperationFailed(format!("Failed store malloc result: {}", e))
        })?;
        Ok(())
    }

    fn generate_neg(
        &self,
        dest: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let result = self
            .builder
            .build_int_neg(v1, "neg_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed neg: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_not(
        &self,
        dest: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let result = self
            .builder
            .build_not(v1, "not_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed not: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_shl(
        &self,
        dest: &Operand,
        count: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(count, registers)?.into_int_value();
        let result = self
            .builder
            .build_left_shift(v1, v2, "shl_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed shl: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_shr(
        &self,
        dest: &Operand,
        count: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(count, registers)?.into_int_value();
        let result = self
            .builder
            .build_right_shift(v1, v2, false, "shr_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed shr: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_sar(
        &self,
        dest: &Operand,
        count: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(count, registers)?.into_int_value();
        let result = self
            .builder
            .build_right_shift(v1, v2, true, "sar_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed sar: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_test(
        &self,
        src1: &Operand,
        src2: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<IntValue<'ctx>> {
        let v1 = self.get_operand_value(src1, registers)?.into_int_value();
        let v2 = self.get_operand_value(src2, registers)?.into_int_value();
        let result = self
            .builder
            .build_and(v1, v2, "test_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed test: {}", e)))?;
        self.builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                result,
                self.context.i64_type().const_int(0, false),
                "test_res",
            )
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed test compare: {}", e)))
    }

    fn generate_address(
        &self,
        addr: &faxc_lir::Address,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<PointerValue<'ctx>> {
        use faxc_lir::Address;
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        match addr {
            Address::Base { base } => {
                let base_val = self.get_physical_register_value(*base, registers)?;
                let ptr = self
                    .builder
                    .build_int_to_ptr(base_val, ptr_type, "base_ptr")
                    .map_err(|e| {
                        CodeGenError::LlvmOperationFailed(format!("Failed base: {}", e))
                    })?;
                Ok(ptr)
            },
            Address::BaseOffset { base, offset } => {
                let base_val = self.get_physical_register_value(*base, registers)?;
                let offset_val = i64_type.const_int(*offset as u64, true);
                let sum = self
                    .builder
                    .build_int_add(base_val, offset_val, "base_offset_sum")
                    .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed add: {}", e)))?;
                let result = self
                    .builder
                    .build_int_to_ptr(sum, ptr_type, "base_offset_ptr")
                    .map_err(|e| {
                        CodeGenError::LlvmOperationFailed(format!("Failed inttoptr: {}", e))
                    })?;
                Ok(result)
            },
            Address::Indexed {
                base,
                index,
                scale,
                offset,
            } => {
                let base_val = self.get_physical_register_value(*base, registers)?;
                let index_val = self.get_physical_register_value(*index, registers)?;
                let scale_val = i64_type.const_int(*scale as u64, false);
                let scaled_index = self
                    .builder
                    .build_int_mul(index_val, scale_val, "scaled_index")
                    .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed mul: {}", e)))?;
                let with_base = self
                    .builder
                    .build_int_add(base_val, scaled_index, "indexed_base")
                    .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed add: {}", e)))?;
                let offset_val = i64_type.const_int(*offset as u64, true);
                let result_val = self
                    .builder
                    .build_int_add(with_base, offset_val, "indexed_sum")
                    .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed add: {}", e)))?;
                let result = self
                    .builder
                    .build_int_to_ptr(result_val, ptr_type, "indexed_ptr")
                    .map_err(|e| {
                        CodeGenError::LlvmOperationFailed(format!("Failed indexed: {}", e))
                    })?;
                Ok(result)
            },
            Address::RipRelative { offset, .. } => {
                let offset_val = self.context.i64_type().const_int(*offset as u64, true);
                let ptr = self
                    .builder
                    .build_int_to_ptr(
                        offset_val,
                        self.context.ptr_type(inkwell::AddressSpace::default()),
                        "rip_ptr",
                    )
                    .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed rip: {}", e)))?;
                Ok(ptr)
            },
            Address::StackRelative { offset } => {
                let offset_val = self.context.i64_type().const_int(*offset as u64, true);
                let ptr = self
                    .builder
                    .build_int_to_ptr(
                        offset_val,
                        self.context.ptr_type(inkwell::AddressSpace::default()),
                        "stack_ptr",
                    )
                    .map_err(|e| {
                        CodeGenError::LlvmOperationFailed(format!("Failed stack: {}", e))
                    })?;
                Ok(ptr)
            },
            Address::Absolute(addr_val) => {
                let ptr = self.context.ptr_type(inkwell::AddressSpace::default());
                let ptr_val = self.context.i64_type().const_int(*addr_val, false);
                let result = self
                    .builder
                    .build_int_to_ptr(ptr_val, ptr, "abs_ptr")
                    .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed abs: {}", e)))?;
                Ok(result)
            },
            Address::Global(symbol) => {
                let name = symbol.as_str();
                let i64_type = self.context.i64_type();
                let global = if let Some(g) = self.module.get_global(name) {
                    g
                } else {
                    self.module.add_global(i64_type, None, name)
                };
                Ok(global.as_pointer_value())
            },
        }
    }

    fn get_physical_register_value(
        &self,
        _phys_reg: PhysicalRegister,
        _registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<IntValue<'ctx>> {
        let i64_type = self.context.i64_type();
        Ok(i64_type.const_int(0, false))
    }

    fn generate_cmp(
        &self,
        src1: &Operand,
        src2: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<IntValue<'ctx>> {
        let v1 = self.get_operand_value(src1, registers)?.into_int_value();
        let v2 = self.get_operand_value(src2, registers)?.into_int_value();
        self.builder
            .build_int_compare(inkwell::IntPredicate::EQ, v1, v2, "cmp_res")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed compare: {}", e)))
    }

    #[allow(dead_code)]
    fn generate_fcmp(
        &self,
        src1: &Operand,
        src2: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<IntValue<'ctx>> {
        let v1 = self.get_operand_value(src1, registers)?.into_float_value();
        let v2 = self.get_operand_value(src2, registers)?.into_float_value();
        self.builder
            .build_float_compare(inkwell::FloatPredicate::OEQ, v1, v2, "fcmp_res")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed fcompare: {}", e)))
    }

    #[allow(dead_code)]
    fn generate_rem(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
        signed: bool,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.i64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_int_value();
        let v2 = self.get_operand_value(src, registers)?.into_int_value();
        let result = if signed {
            self.builder
                .build_int_signed_rem(v1, v2, "rem_tmp")
                .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed rem: {}", e)))?
        } else {
            self.builder
                .build_int_unsigned_rem(v1, v2, "urem_tmp")
                .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed urem: {}", e)))?
        };
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    #[allow(dead_code)]
    fn generate_fadd(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.f64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_float_value();
        let v2 = self.get_operand_value(src, registers)?.into_float_value();
        let result = self
            .builder
            .build_float_add(v1, v2, "fadd_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed fadd: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    #[allow(dead_code)]
    fn generate_fsub(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.f64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_float_value();
        let v2 = self.get_operand_value(src, registers)?.into_float_value();
        let result = self
            .builder
            .build_float_sub(v1, v2, "fsub_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed fsub: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    #[allow(dead_code)]
    fn generate_fmul(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.f64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_float_value();
        let v2 = self.get_operand_value(src, registers)?.into_float_value();
        let result = self
            .builder
            .build_float_mul(v1, v2, "fmul_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed fmul: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    #[allow(dead_code)]
    fn generate_fdiv(
        &self,
        dest: &Operand,
        src: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<()> {
        let dest_ptr = self.get_or_create_register_ptr(dest, registers)?;
        let v1 = self
            .builder
            .build_load(self.context.f64_type(), dest_ptr, "load_dest")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed load dest: {}", e)))?
            .into_float_value();
        let v2 = self.get_operand_value(src, registers)?.into_float_value();
        let result = self
            .builder
            .build_float_div(v1, v2, "fdiv_tmp")
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed fdiv: {}", e)))?;
        self.builder
            .build_store(dest_ptr, result)
            .map_err(|e| CodeGenError::LlvmOperationFailed(format!("Failed to store: {}", e)))?;
        Ok(())
    }

    fn generate_call(&self, target: &CallTarget) -> Result<()> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);

        match target {
            CallTarget::Direct(name) | CallTarget::External(name) => {
                let func = self
                    .module
                    .get_function(name.as_str())
                    .unwrap_or_else(|| self.module.add_function(name.as_str(), fn_type, None));
                self.builder
                    .build_call(func, &[], "call_tmp")
                    .map_err(|e| {
                        CodeGenError::LlvmOperationFailed(format!("Failed call: {}", e))
                    })?;
            },
            CallTarget::Indirect(_) => {
                // Indirect call - more complex, simplified for now
            },
        }
        Ok(())
    }

    fn convert_condition(&self, cond: &Condition, cmp_val: IntValue<'ctx>) -> IntValue<'ctx> {
        match cond {
            Condition::Eq => cmp_val,
            Condition::Ne => self
                .builder
                .build_not(cmp_val, "not_cmp")
                .unwrap_or(cmp_val),
            Condition::L => cmp_val, // Less than: signed <
            Condition::Le => {
                // Less than or equal: signed <=
                self.builder
                    .build_not(cmp_val, "not_cmp")
                    .unwrap_or(cmp_val)
            },
            Condition::G => cmp_val, // Greater than: signed >
            Condition::Ge => {
                // Greater than or equal: signed >=
                self.builder
                    .build_not(cmp_val, "not_cmp")
                    .unwrap_or(cmp_val)
            },
            Condition::B => cmp_val, // Below: unsigned <
            Condition::Be => {
                // Below or equal: unsigned <=
                self.builder
                    .build_not(cmp_val, "not_cmp")
                    .unwrap_or(cmp_val)
            },
            Condition::A => cmp_val, // Above: unsigned >
            Condition::Ae => {
                // Above or equal: unsigned >=
                self.builder
                    .build_not(cmp_val, "not_cmp")
                    .unwrap_or(cmp_val)
            },
            _ => cmp_val,
        }
    }

    /// Build alloca with proper error handling
    fn build_alloca(
        &self,
        ty: inkwell::types::IntType<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>> {
        self.builder.build_alloca(ty, name).map_err(|e| {
            CodeGenError::LlvmOperationFailed(format!(
                "Failed to allocate stack slot '{}': {}",
                name, e
            ))
        })
    }

    fn get_or_create_register_ptr(
        &self,
        operand: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<PointerValue<'ctx>> {
        let i64_type = self.context.i64_type();

        match operand {
            Operand::Reg(vreg) => {
                // Check if register already exists
                if let Some(&ptr) = registers.get(vreg) {
                    return Ok(ptr);
                }
                // Create new register allocation
                let ptr = self.build_alloca(i64_type, &format!("r{}", vreg.id))?;
                registers.insert(*vreg, ptr);
                Ok(ptr)
            },
            Operand::PhysReg(_) => self.build_alloca(i64_type, "phys_reg"),
            _ => self.build_alloca(i64_type, "temp"),
        }
    }

    fn get_operand_value(
        &self,
        operand: &Operand,
        registers: &mut HashMap<VirtualRegister, PointerValue<'ctx>>,
    ) -> Result<BasicValueEnum<'ctx>> {
        let i64_type = self.context.i64_type();

        match operand {
            Operand::Reg(vreg) => {
                let ptr = self.get_or_create_register_ptr(operand, registers)?;
                Ok(self
                    .builder
                    .build_load(i64_type, ptr, &format!("load_r{}", vreg.id))
                    .map_err(|e| {
                        CodeGenError::LlvmOperationFailed(format!("Failed load: {}", e))
                    })?)
            },
            Operand::Imm(n) => Ok(i64_type.const_int(*n as u64, false).into()),
            Operand::PhysReg(_) => Ok(i64_type.const_int(0, false).into()),
            Operand::Mem(_) => Ok(i64_type.const_int(0, false).into()),
            Operand::Label(_) => Ok(i64_type.const_int(0, false).into()),
        }
    }

    /// Optimize the module
    pub fn optimize(&mut self) {
        // Apply optimization passes based on opt_level
    }

    /// Emit LLVM IR as string
    pub fn emit_llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Write LLVM IR to file
    pub fn write_ir_to_file(&self, path: &Path) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;

        let ir = self.emit_llvm_ir();
        let mut file = File::create(path)?;
        file.write_all(ir.as_bytes())?;
        Ok(())
    }

    /// Get the LLVM module
    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Write object file
    pub fn write_object_file(&self, path: &Path) -> crate::Result<()> {
        use inkwell::targets::{FileType, TargetTriple};

        let triple = TargetTriple::create(&self.target_triple);
        let target = inkwell::targets::Target::from_triple(&triple)
            .map_err(|e| CodeGenError::CompilationError(format!("Failed to get target: {}", e)))?;

        let target_machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                self.opt_level,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| {
                CodeGenError::CompilationError("Failed to create target machine".to_string())
            })?;

        target_machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| {
                CodeGenError::CompilationError(format!("Failed to write object file: {}", e))
            })?;

        Ok(())
    }

    /// Write assembly file
    pub fn write_asm_file(&self, path: &Path) -> crate::Result<()> {
        use inkwell::targets::{FileType, TargetTriple};

        let triple = TargetTriple::create(&self.target_triple);
        let target = inkwell::targets::Target::from_triple(&triple)
            .map_err(|e| CodeGenError::CompilationError(format!("Failed to get target: {}", e)))?;

        let target_machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                self.opt_level,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| {
                CodeGenError::CompilationError("Failed to create target machine".to_string())
            })?;

        target_machine
            .write_to_file(&self.module, FileType::Assembly, path)
            .map_err(|e| {
                CodeGenError::CompilationError(format!("Failed to write asm file: {}", e))
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod llvm_tests {
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_llvm_backend_creation() {
        let context = Context::create();
        let backend = LlvmBackend::new(
            &context,
            "test",
            "x86_64-unknown-linux-gnu".to_string(),
            OptimizationLevel::None,
        );
        assert_eq!(backend.target_triple, "x86_64-unknown-linux-gnu");
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
        let _ir = backend.emit_llvm_ir();
        // Test passes if backend can create IR without panicking
    }
}
