//! Stack Frame Management for x86-64
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 2
//! Manages stack frame layout, local variable allocation, and spill slots.

use crate::calling_convention::SystemVAbi;
use crate::lir::{PhysicalRegister, RegisterWidth, VirtualRegister};

/// Stack frame layout for a function
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Total frame size in bytes
    pub frame_size: u32,
    /// Offset to saved RBP
    pub saved_rbp_offset: i32,
    /// Offset to return address
    pub return_addr_offset: i32,
    /// Offset to first local variable
    pub locals_base_offset: i32,
    /// Offset to spill slots
    pub spill_base_offset: i32,
    /// Callee-saved registers saved in this frame
    pub saved_callee_regs: Vec<(PhysicalRegister, i32)>,
    /// Local variable offsets
    pub local_offsets: Vec<i32>,
    /// Next available spill slot
    pub next_spill_slot: i32,
}

impl StackFrame {
    pub fn new() -> Self {
        Self {
            frame_size: 0,
            saved_rbp_offset: 0,
            return_addr_offset: 8,  // After push rbp
            locals_base_offset: 16, // After saved rbp and return address
            spill_base_offset: 16,
            saved_callee_regs: Vec::new(),
            local_offsets: Vec::new(),
            next_spill_slot: 0,
        }
    }

    /// Calculate frame size based on locals and saved registers
    pub fn frame_size(
        &mut self,
        local_count: usize,
        spill_slot_count: usize,
        save_callee_regs: bool,
    ) {
        let mut size: i32 = 0;

        // Saved RBP (8 bytes)
        size += 8;
        self.saved_rbp_offset = 0;

        // Callee-saved registers if needed
        if save_callee_regs {
            let callee_saved = SystemVAbi::get_callee_saved_regs();
            for (_i, reg) in callee_saved.iter().enumerate() {
                let offset = size;
                self.saved_callee_regs.push((*reg, offset));
                size += 8;
            }
        }

        // Align to 16 bytes
        size = (size + 15) & !15;
        self.locals_base_offset = size;

        // Local variables (8 bytes each for simplicity)
        for _i in 0..local_count {
            self.local_offsets.push(size);
            size += 8;
        }

        // Spill slots
        self.spill_base_offset = size;
        self.next_spill_slot = 0;
        for _ in 0..spill_slot_count {
            size += 8;
            self.next_spill_slot += 8;
        }

        // Ensure 16-byte alignment
        size = (size + 15) & !15;
        self.frame_size = size as u32;
    }

    /// Get the stack offset for a local variable
    pub fn get_local_offset(&self, local_index: usize) -> Option<i32> {
        if local_index < self.local_offsets.len() {
            Some(self.local_offsets[local_index])
        } else {
            None
        }
    }

    /// Allocate a spill slot and return its offset
    pub fn allocate_spill_slot(&mut self) -> i32 {
        let offset = self.spill_base_offset + self.next_spill_slot;
        self.next_spill_slot += 8;
        // Recalculate frame size
        self.frame_size = ((self.spill_base_offset + self.next_spill_slot + 15) & !15) as u32;
        offset
    }

    /// Get the stack offset for a saved callee register
    pub fn get_saved_reg_offset(&self, reg: PhysicalRegister) -> Option<i32> {
        self.saved_callee_regs
            .iter()
            .find(|(r, _)| *r == reg)
            .map(|(_, offset)| *offset)
    }

    /// Generate stack-relative address for a local
    pub fn local_address(&self, local_index: usize) -> Option<crate::lir::Address> {
        use crate::lir::Address;
        self.get_local_offset(local_index)
            .map(|offset| Address::StackRelative { offset: -offset })
    }

    /// Generate stack-relative address for a spill slot
    pub fn spill_address(&self, slot_index: i32) -> crate::lir::Address {
        use crate::lir::Address;
        Address::StackRelative {
            offset: -(self.spill_base_offset + slot_index * 8),
        }
    }
}

impl Default for StackFrame {
    fn default() -> Self {
        Self::new()
    }
}

/// Register allocator spill information
#[derive(Debug, Clone)]
pub struct SpillInfo {
    pub virtual_reg: VirtualRegister,
    pub spill_slot: i32,
    pub width: RegisterWidth,
}

/// Local variable information
#[derive(Debug, Clone)]
pub struct LocalInfo {
    pub index: usize,
    pub stack_offset: i32,
    pub width: RegisterWidth,
    pub is_parameter: bool,
}

impl LocalInfo {
    pub fn new(index: usize, stack_offset: i32, is_parameter: bool) -> Self {
        Self {
            index,
            stack_offset,
            width: RegisterWidth::W64,
            is_parameter,
        }
    }
}

/// Parameter location (register or stack)
#[derive(Debug, Clone)]
pub enum ParamLocation {
    Register(PhysicalRegister),
    Stack(i32), // Offset from RBP
}

/// Parameter assignment for function entry
#[derive(Debug, Clone)]
pub struct ParamAssignment {
    pub param_index: usize,
    pub location: ParamLocation,
    pub target_local: usize,
}

impl ParamAssignment {
    pub fn from_systemv(param_index: usize, target_local: usize, is_fp: bool) -> Self {
        let location = if is_fp {
            if let Some(reg) = SystemVAbi::get_fp_arg_register(param_index) {
                ParamLocation::Register(reg)
            } else {
                let offset = SystemVAbi::get_stack_arg_offset(param_index, true);
                ParamLocation::Stack(offset)
            }
        } else {
            if let Some(reg) = SystemVAbi::get_arg_register(param_index) {
                ParamLocation::Register(reg)
            } else {
                let offset = SystemVAbi::get_stack_arg_offset(param_index, false);
                ParamLocation::Stack(offset)
            }
        };

        Self {
            param_index,
            location,
            target_local,
        }
    }
}

#[cfg(test)]
mod stack_frame_tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let frame = StackFrame::new();
        assert_eq!(frame.frame_size, 0);
        assert_eq!(frame.return_addr_offset, 8);
    }

    #[test]
    fn test_frame_calculation() {
        let mut frame = StackFrame::new();
        frame.frame_size(4, 2, true);

        assert!(frame.frame_size > 0);
        assert_eq!(frame.local_offsets.len(), 4);
        assert!(!frame.saved_callee_regs.is_empty());
    }

    #[test]
    fn test_spill_allocation() {
        let mut frame = StackFrame::new();
        frame.frame_size(2, 0, false);

        let slot1 = frame.allocate_spill_slot();
        let slot2 = frame.allocate_spill_slot();

        assert_eq!(slot2 - slot1, 8);
    }

    #[test]
    fn test_param_assignment() {
        let assign = ParamAssignment::from_systemv(0, 0, false);
        assert_eq!(assign.param_index, 0);
        assert!(matches!(
            assign.location,
            ParamLocation::Register(PhysicalRegister::RDI)
        ));
    }
}
