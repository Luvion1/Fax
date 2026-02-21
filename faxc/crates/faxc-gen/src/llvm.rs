use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::OptimizationLevel;
use faxc_lir::{Function as LirFunction, Instruction, Value, BinOp, Condition};
use std::path::Path;

pub struct LlvmBackend<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub target_triple: String,
    pub opt_level: OptimizationLevel,
}

use inkwell::values::{FunctionValue, PointerValue};
use faxc_lir::{Register, Value as LirValue};
use std::collections::HashMap;

impl<'ctx> LlvmBackend<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str, target_triple: String, opt_level: OptimizationLevel) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        Self { context, module, builder, target_triple, opt_level }
    }

    pub fn compile_function(&mut self, func: &LirFunction) -> FunctionValue<'ctx> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false); // MVP: Always returns i64
        let function = self.module.add_function(func.name.as_str(), fn_type, None);
        
        let mut registers: HashMap<Register, PointerValue<'ctx>> = HashMap::new();
        let mut llvm_blocks: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>> = HashMap::new();

        // Pass 1: Create all basic blocks
        for instr in &func.instructions {
            if let Instruction::Label { name } = instr {
                let block = self.context.append_basic_block(function, name);
                llvm_blocks.insert(name.clone(), block);
            }
        }

        // Add initial entry block if not present
        let entry_block = if let Some(b) = llvm_blocks.get(".Lbb0") {
            *b
        } else {
            self.context.append_basic_block(function, "entry")
        };
        self.builder.position_at_end(entry_block);

        let mut last_cmp_val = None;

        for instr in &func.instructions {
            match instr {
                Instruction::Label { name } => {
                    let block = llvm_blocks.get(name).unwrap();
                    self.builder.position_at_end(*block);
                }
                Instruction::Mov { dest, src } => {
                    let dest_ptr = *registers.entry(*dest).or_insert_with(|| {
                        self.builder.build_alloca(i64_type, &format!("r{}", dest.0)).expect("Failed to alloca")
                    });
                    
                    let val = match src {
                        LirValue::Imm(n) => i64_type.const_int(*n as u64, false),
                        LirValue::Reg(r) => {
                            let r_ptr = registers.get(r).expect("Register not found");
                            self.builder.build_load(i64_type, *r_ptr, &format!("tmp_load{}", r.0)).expect("Failed to load").into_int_value()
                        }
                        _ => i64_type.const_int(0, false),
                    };
                    
                    self.builder.build_store(dest_ptr, val).expect("Failed to store");
                }
                Instruction::BinOp { op, dest, src1, src2 } => {
                    let dest_ptr = *registers.entry(*dest).or_insert_with(|| {
                        self.builder.build_alloca(i64_type, &format!("r{}", dest.0)).expect("Failed to alloca")
                    });

                    let v1_ptr = registers.get(src1).expect("Register not found");
                    let v1 = self.builder.build_load(i64_type, *v1_ptr, "lhs").expect("Failed to load lhs").into_int_value();
                    
                    let v2 = match src2 {
                        LirValue::Imm(n) => i64_type.const_int(*n as u64, false),
                        LirValue::Reg(r) => {
                            let r_ptr = registers.get(r).expect("Register not found");
                            self.builder.build_load(i64_type, *r_ptr, "rhs").expect("Failed to load rhs").into_int_value()
                        }
                        _ => i64_type.const_int(0, false),
                    };

                    let res = match op {
                        faxc_lir::BinOp::Add => self.builder.build_int_add(v1, v2, "add_tmp").expect("Failed to add"),
                        faxc_lir::BinOp::Sub => self.builder.build_int_sub(v1, v2, "sub_tmp").expect("Failed to sub"),
                        _ => v1,
                    };

                    self.builder.build_store(dest_ptr, res).expect("Failed to store");
                }
                Instruction::Cmp { src1, src2 } => {
                    let v1_ptr = registers.get(src1).expect("Register not found");
                    let v1 = self.builder.build_load(i64_type, *v1_ptr, "cmp_lhs").expect("Failed to load").into_int_value();
                    
                    let v2 = match src2 {
                        LirValue::Imm(n) => i64_type.const_int(*n as u64, false),
                        LirValue::Reg(r) => {
                            let r_ptr = registers.get(r).expect("Register not found");
                            self.builder.build_load(i64_type, *r_ptr, "cmp_rhs").expect("Failed to load").into_int_value()
                        }
                        _ => i64_type.const_int(0, false),
                    };

                    last_cmp_val = Some(self.builder.build_int_compare(inkwell::IntPredicate::EQ, v1, v2, "cmp_res").expect("Failed to compare"));
                }
                Instruction::Jmp { target } => {
                    let target_block = llvm_blocks.get(target).expect("Target block not found");
                    self.builder.build_unconditional_branch(*target_block).expect("Failed to branch");
                }
                Instruction::Jcc { cond: _, target } => {
                    let target_block = llvm_blocks.get(target).expect("Target block not found");
                    // Mock: JCC always uses last comparison
                    let cmp_val = last_cmp_val.unwrap();
                    
                    // We need an "else" block for conditional branch in LLVM, 
                    // but LIR JCC is often followed by a jump to the next block.
                    // Simplified: We'll just branch based on cmp_val
                    let next_block = self.context.append_basic_block(function, "cont");
                    self.builder.build_conditional_branch(cmp_val, *target_block, next_block).expect("Failed branch");
                    self.builder.position_at_end(next_block);
                }
                Instruction::Ret => {
                    self.builder.build_return(Some(&i64_type.const_int(0, false))).expect("Failed to return");
                }
                _ => {}
            }
        }

        function
    }

    pub fn optimize(&mut self) {
        // Optimization passes
    }

    pub fn emit_llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }
}
