//! System V AMD64 ABI Calling Convention
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 2
//! Implements the System V AMD64 calling convention used on Linux, macOS, BSD.

use crate::lir::{Operand, PhysicalRegister, RegisterWidth};

/// System V AMD64 ABI calling convention handler
pub struct SystemVAbi;

impl SystemVAbi {
    /// Integer/pointer argument registers in order
    pub const ARG_REGS: [PhysicalRegister; 6] = [
        PhysicalRegister::RDI,
        PhysicalRegister::RSI,
        PhysicalRegister::RDX,
        PhysicalRegister::RCX,
        PhysicalRegister::R8,
        PhysicalRegister::R9,
    ];

    /// Floating point argument registers in order
    pub const FP_ARG_REGS: [PhysicalRegister; 8] = [
        PhysicalRegister::XMM0,
        PhysicalRegister::XMM1,
        PhysicalRegister::XMM2,
        PhysicalRegister::XMM3,
        PhysicalRegister::XMM4,
        PhysicalRegister::XMM5,
        PhysicalRegister::XMM6,
        PhysicalRegister::XMM7,
    ];

    /// Return register for integer/pointer types
    pub const RET_REG: PhysicalRegister = PhysicalRegister::RAX;

    /// Return register for floating point types
    pub const FP_RET_REG: PhysicalRegister = PhysicalRegister::XMM0;

    /// Get the argument register for a given argument index (0-5)
    pub fn get_arg_register(index: usize) -> Option<PhysicalRegister> {
        if index < 6 {
            Some(Self::ARG_REGS[index])
        } else {
            None
        }
    }

    /// Get the FP argument register for a given argument index (0-7)
    pub fn get_fp_arg_register(index: usize) -> Option<PhysicalRegister> {
        if index < 8 {
            Some(Self::FP_ARG_REGS[index])
        } else {
            None
        }
    }

    /// Check if an argument should be passed on the stack
    pub fn is_stack_arg(arg_index: usize, is_fp: bool) -> bool {
        if is_fp {
            arg_index >= 8
        } else {
            arg_index >= 6
        }
    }

    /// Calculate stack offset for argument at index (beyond register args)
    pub fn get_stack_arg_offset(arg_index: usize, is_fp: bool) -> i32 {
        // First stack arg is at [rsp + 16] (after return address)
        let first_stack_idx = if is_fp { 8 } else { 6 };
        let stack_idx = arg_index - first_stack_idx;
        16 + (stack_idx as i32 * 8)
    }

    /// Get callee-saved registers that need to be preserved
    pub fn get_callee_saved_regs() -> &'static [PhysicalRegister] {
        &[
            PhysicalRegister::RBX,
            PhysicalRegister::RBP,
            PhysicalRegister::R12,
            PhysicalRegister::R13,
            PhysicalRegister::R14,
            PhysicalRegister::R15,
        ]
    }

    /// Generate prologue instructions for a function
    pub fn generate_prologue(frame_size: u32, uses_fp: bool) -> Vec<crate::lir::Instruction> {
        use crate::lir::Instruction;
        let mut prologue = Vec::new();

        // Push base pointer
        prologue.push(Instruction::Push {
            src: Operand::PhysReg(PhysicalRegister::RBP),
        });

        // Set up new base pointer
        prologue.push(Instruction::Mov {
            dest: Operand::PhysReg(PhysicalRegister::RBP),
            src: Operand::PhysReg(PhysicalRegister::RSP),
        });

        // Allocate stack frame
        if frame_size > 0 {
            prologue.push(Instruction::Sub {
                dest: Operand::PhysReg(PhysicalRegister::RSP),
                src: Operand::Imm(frame_size as i64),
            });
        }

        // Save callee-saved registers if used
        if uses_fp {
            let callee_saved: Vec<PhysicalRegister> = Self::get_callee_saved_regs().to_vec();
            prologue.push(Instruction::SaveCalleeSaved { regs: callee_saved });
        }

        prologue
    }

    /// Generate epilogue instructions for a function
    pub fn generate_epilogue(frame_size: u32, uses_fp: bool) -> Vec<crate::lir::Instruction> {
        use crate::lir::Instruction;
        let mut epilogue = Vec::new();

        // Restore callee-saved registers if used
        if uses_fp {
            let callee_saved: Vec<PhysicalRegister> = Self::get_callee_saved_regs().to_vec();
            epilogue.push(Instruction::RestoreCalleeSaved { regs: callee_saved });
        }

        // Deallocate stack frame
        if frame_size > 0 {
            epilogue.push(Instruction::Add {
                dest: Operand::PhysReg(PhysicalRegister::RSP),
                src: Operand::Imm(frame_size as i64),
            });
        }

        // Restore base pointer
        epilogue.push(Instruction::Pop {
            dest: Operand::PhysReg(PhysicalRegister::RBP),
        });

        epilogue
    }
}

/// Argument classification for System V AMD64
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgClass {
    Integer,
    Float,
    Memory,
    Complex,
}

impl ArgClass {
    pub fn for_type(_width: RegisterWidth, is_float: bool) -> Self {
        if is_float {
            ArgClass::Float
        } else {
            ArgClass::Integer
        }
    }
}

/// Call frame information
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub return_address_offset: i32,
    pub first_arg_offset: i32,
    pub frame_size: u32,
    pub saved_regs: Vec<PhysicalRegister>,
}

impl CallFrame {
    pub fn new(frame_size: u32) -> Self {
        Self {
            return_address_offset: 0,
            first_arg_offset: 8, // After return address
            frame_size,
            saved_regs: Vec::new(),
        }
    }

    pub fn with_saved_regs(mut self, regs: Vec<PhysicalRegister>) -> Self {
        self.saved_regs = regs;
        self
    }
}

#[cfg(test)]
mod abi_tests {
    use super::*;

    #[test]
    fn test_arg_registers() {
        assert_eq!(SystemVAbi::get_arg_register(0), Some(PhysicalRegister::RDI));
        assert_eq!(SystemVAbi::get_arg_register(5), Some(PhysicalRegister::R9));
        assert_eq!(SystemVAbi::get_arg_register(6), None);
    }

    #[test]
    fn test_fp_arg_registers() {
        assert_eq!(
            SystemVAbi::get_fp_arg_register(0),
            Some(PhysicalRegister::XMM0)
        );
        assert_eq!(
            SystemVAbi::get_fp_arg_register(7),
            Some(PhysicalRegister::XMM7)
        );
        assert_eq!(SystemVAbi::get_fp_arg_register(8), None);
    }

    #[test]
    fn test_stack_arg_detection() {
        assert!(!SystemVAbi::is_stack_arg(0, false));
        assert!(!SystemVAbi::is_stack_arg(5, false));
        assert!(SystemVAbi::is_stack_arg(6, false));
        assert!(SystemVAbi::is_stack_arg(8, true));
    }

    #[test]
    fn test_callee_saved_regs() {
        let saved = SystemVAbi::get_callee_saved_regs();
        assert!(saved.contains(&PhysicalRegister::RBX));
        assert!(saved.contains(&PhysicalRegister::RBP));
        assert!(!saved.contains(&PhysicalRegister::RAX));
    }
}
