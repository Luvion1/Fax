//! LIR Crate Integration Tests
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 2
//! Unit and integration tests for LIR constructs, lowering, calling convention, and stack frames.

use crate::*;
use faxc_util::Symbol;

#[test]
fn test_virtual_register_creation() {
    let reg = VirtualRegister::new(0);
    assert_eq!(reg.id, 0);
    assert_eq!(reg.width, RegisterWidth::W64);

    let reg_w32 = VirtualRegister::with_width(1, RegisterWidth::W32);
    assert_eq!(reg_w32.id, 1);
    assert_eq!(reg_w32.width, RegisterWidth::W32);
}

#[test]
fn test_physical_register_properties() {
    // Caller-saved (volatile) registers
    assert!(PhysicalRegister::RAX.is_caller_saved());
    assert!(PhysicalRegister::RCX.is_caller_saved());
    assert!(PhysicalRegister::R10.is_caller_saved());

    // Callee-saved (non-volatile) registers
    assert!(PhysicalRegister::RBX.is_callee_saved());
    assert!(PhysicalRegister::RBP.is_callee_saved());
    assert!(PhysicalRegister::R12.is_callee_saved());

    // RAX is not callee-saved
    assert!(!PhysicalRegister::RAX.is_callee_saved());
}

#[test]
fn test_function_creation() {
    let name = Symbol::intern("test_fn");
    let func = Function::new(name);

    assert_eq!(func.name, name);
    assert_eq!(func.instruction_count(), 0);
    assert!(!func.is_external);
}

#[test]
fn test_lir_instructions() {
    use crate::lir::*;

    let reg1 = VirtualRegister::new(0);
    let reg2 = VirtualRegister::new(1);

    // Test Mov instruction
    let mov = Instruction::Mov {
        dest: Operand::Reg(reg1),
        src: Operand::Imm(42),
    };
    assert!(matches!(mov, Instruction::Mov { .. }));

    // Test BinOp instruction
    let add = Instruction::Add {
        dest: Operand::Reg(reg1),
        src: Operand::Reg(reg2),
    };
    assert!(matches!(add, Instruction::Add { .. }));

    // Test Cmp instruction
    let cmp = Instruction::Cmp {
        src1: Operand::Reg(reg1),
        src2: Operand::Imm(0),
    };
    assert!(matches!(cmp, Instruction::Cmp { .. }));
}

#[test]
fn test_condition_codes() {
    assert_eq!(Condition::Eq as u8, 0);
    assert_eq!(Condition::Ne as u8, 1);

    // Test condition conversion
    let mir_cond = MirCondition::Eq;
    let lir_cond = Condition::from_mir_condition(mir_cond);
    assert_eq!(lir_cond, Condition::Eq);
}

#[test]
fn test_addressing_modes() {
    use crate::lir::Address;

    // Base addressing
    let base = Address::Base {
        base: PhysicalRegister::RBP,
    };
    assert!(matches!(base, Address::Base { .. }));

    // Base + offset
    let base_off = Address::BaseOffset {
        base: PhysicalRegister::RBP,
        offset: 16,
    };
    assert!(matches!(base_off, Address::BaseOffset { .. }));

    // Indexed addressing
    let indexed = Address::Indexed {
        base: PhysicalRegister::RBP,
        index: PhysicalRegister::RAX,
        scale: 8,
        offset: 0,
    };
    assert!(matches!(indexed, Address::Indexed { .. }));

    // Stack relative
    let stack = Address::StackRelative { offset: -16 };
    assert!(matches!(stack, Address::StackRelative { .. }));
}

#[test]
fn test_lower_mir_to_lir_basic() {
    use faxc_mir::{
        BasicBlock, BlockId, Constant, ConstantKind, Function as MirFunction, LocalId, Operand,
        Rvalue, Statement, Terminator,
    };
    use faxc_sem::Type;

    // Create a simple MIR function
    let mut mir_func = MirFunction::new(Symbol::intern("test"), Type::Int, 0);

    let entry = BlockId::from_usize(0);
    mir_func.blocks.push(BasicBlock {
        id: entry,
        statements: vec![Statement::Assign(
            Place::Local(LocalId(1)),
            Rvalue::Use(Operand::Constant(Constant {
                ty: Type::Int,
                kind: ConstantKind::Int(42),
            })),
        )],
        terminator: Terminator::Return,
    });

    // Lower to LIR
    let lir_func = lower_mir_to_lir(&mir_func);

    assert_eq!(lir_func.name, Symbol::intern("test"));
    assert!(lir_func.instruction_count() > 0);
}

#[test]
fn test_systemv_abi_arg_registers() {
    use crate::calling_convention::SystemVAbi;

    // First 6 integer args in registers
    assert_eq!(SystemVAbi::get_arg_register(0), Some(PhysicalRegister::RDI));
    assert_eq!(SystemVAbi::get_arg_register(1), Some(PhysicalRegister::RSI));
    assert_eq!(SystemVAbi::get_arg_register(2), Some(PhysicalRegister::RDX));
    assert_eq!(SystemVAbi::get_arg_register(3), Some(PhysicalRegister::RCX));
    assert_eq!(SystemVAbi::get_arg_register(4), Some(PhysicalRegister::R8));
    assert_eq!(SystemVAbi::get_arg_register(5), Some(PhysicalRegister::R9));
    assert_eq!(SystemVAbi::get_arg_register(6), None); // Stack
}

#[test]
fn test_stack_frame_layout() {
    use crate::stack_frame::StackFrame;

    let mut frame = StackFrame::new();
    frame.frame_size(4, 2, true);

    assert!(frame.frame_size > 0);
    assert_eq!(frame.local_offsets.len(), 4);
    assert!(!frame.saved_callee_regs.is_empty());

    // Test 16-byte alignment
    assert_eq!(frame.frame_size % 16, 0);
}

#[test]
fn test_spill_slot_allocation() {
    use crate::stack_frame::StackFrame;

    let mut frame = StackFrame::new();
    frame.calculate_frame_size(2, 0, false);

    let slot1 = frame.allocate_spill_slot();
    let slot2 = frame.allocate_spill_slot();

    // Spill slots are 8 bytes apart
    assert_eq!(slot2 - slot1, 8);
}

#[test]
fn test_param_assignment() {
    use crate::stack_frame::{ParamAssignment, ParamLocation};

    // First param should be in RDI
    let assign0 = ParamAssignment::from_systemv(0, 0, false);
    assert!(matches!(
        assign0.location,
        ParamLocation::Register(PhysicalRegister::RDI)
    ));

    // Seventh param should be on stack
    let assign6 = ParamAssignment::from_systemv(6, 6, false);
    assert!(matches!(assign6.location, ParamLocation::Stack(_)));
}
