//! Edge case tests for faxc-lir

#[cfg(test)]
mod tests {
    use crate::{Function, Register, Instruction, Value, Address, BinOp, UnOp, Condition};

    // ==================== FUNCTION TESTS ====================

    /// EDGE CASE: Empty function
    #[test]
    fn test_edge_empty_function() {
        let func = Function {
            name: faxc_util::Symbol::intern("empty"),
            registers: vec![],
            instructions: vec![],
            labels: vec![],
            frame_size: 0,
        };
        assert_eq!(func.instructions.len(), 0);
        assert_eq!(func.frame_size, 0);
    }

    /// EDGE CASE: Function with single instruction
    #[test]
    fn test_edge_single_instruction() {
        let func = Function {
            name: faxc_util::Symbol::intern("one"),
            registers: vec![Register(0)],
            instructions: vec![Instruction::Nop],
            labels: vec![],
            frame_size: 0,
        };
        assert_eq!(func.instructions.len(), 1);
    }

    /// EDGE CASE: Function with many instructions
    #[test]
    fn test_edge_many_instructions() {
        let instrs: Vec<_> = (0..100).map(|_| Instruction::Nop).collect();
        let func = Function {
            name: faxc_util::Symbol::intern("many"),
            registers: vec![],
            instructions: instrs,
            labels: vec![],
            frame_size: 0,
        };
        assert_eq!(func.instructions.len(), 100);
    }

    /// EDGE CASE: Large frame size
    #[test]
    fn test_edge_large_frame() {
        let func = Function {
            name: faxc_util::Symbol::intern("big_frame"),
            registers: vec![],
            instructions: vec![],
            labels: vec![],
            frame_size: u32::MAX,
        };
        assert_eq!(func.frame_size, u32::MAX);
    }

    // ==================== INSTRUCTION TESTS ====================

    /// EDGE CASE: Nop instruction
    #[test]
    fn test_edge_nop() {
        let instr = Instruction::Nop;
        assert!(matches!(instr, Instruction::Nop));
    }

    /// EDGE CASE: Mov instruction with immediate
    #[test]
    fn test_edge_mov_imm() {
        let instr = Instruction::Mov {
            dest: Register(0),
            src: Value::Imm(42),
        };
        assert!(matches!(instr, Instruction::Mov { .. }));
    }

    /// EDGE CASE: Mov instruction with register
    #[test]
    fn test_edge_mov_reg() {
        let instr = Instruction::Mov {
            dest: Register(0),
            src: Value::Reg(Register(1)),
        };
        assert!(matches!(instr, Instruction::Mov { .. }));
    }

    /// EDGE CASE: Load instruction
    #[test]
    fn test_edge_load() {
        let instr = Instruction::Load {
            dest: Register(0),
            addr: Address::Stack(0),
        };
        assert!(matches!(instr, Instruction::Load { .. }));
    }

    /// EDGE CASE: Store instruction
    #[test]
    fn test_edge_store() {
        let instr = Instruction::Store {
            addr: Address::Stack(0),
            src: Register(0),
        };
        assert!(matches!(instr, Instruction::Store { .. }));
    }

    /// EDGE CASE: Lea instruction
    #[test]
    fn test_edge_lea() {
        let instr = Instruction::Lea {
            dest: Register(0),
            addr: Address::Base { base: Register(1), offset: 8 },
        };
        assert!(matches!(instr, Instruction::Lea { .. }));
    }

    /// EDGE CASE: BinOp instruction
    #[test]
    fn test_edge_binop() {
        let instr = Instruction::BinOp {
            op: BinOp::Add,
            dest: Register(0),
            src1: Register(1),
            src2: Value::Imm(42),
        };
        assert!(matches!(instr, Instruction::BinOp { .. }));
    }

    /// EDGE CASE: UnOp instruction
    #[test]
    fn test_edge_unop() {
        let instr = Instruction::UnOp {
            op: UnOp::Neg,
            dest: Register(0),
            src: Register(1),
        };
        assert!(matches!(instr, Instruction::UnOp { .. }));
    }

    /// EDGE CASE: Cmp instruction
    #[test]
    fn test_edge_cmp() {
        let instr = Instruction::Cmp {
            src1: Register(0),
            src2: Value::Imm(0),
        };
        assert!(matches!(instr, Instruction::Cmp { .. }));
    }

    /// EDGE CASE: Jmp instruction
    #[test]
    fn test_edge_jmp() {
        let instr = Instruction::Jmp {
            target: ".Lbb1".to_string(),
        };
        assert!(matches!(instr, Instruction::Jmp { .. }));
    }

    /// EDGE CASE: Jcc instruction
    #[test]
    fn test_edge_jcc() {
        let instr = Instruction::Jcc {
            cond: Condition::Eq,
            target: ".Lbb1".to_string(),
        };
        assert!(matches!(instr, Instruction::Jcc { .. }));
    }

    /// EDGE CASE: Call instruction
    #[test]
    fn test_edge_call() {
        let instr = Instruction::Call {
            func: Value::Label("foo".to_string()),
        };
        assert!(matches!(instr, Instruction::Call { .. }));
    }

    /// EDGE CASE: Ret instruction
    #[test]
    fn test_edge_ret() {
        let instr = Instruction::Ret;
        assert!(matches!(instr, Instruction::Ret));
    }

    /// EDGE CASE: Push instruction
    #[test]
    fn test_edge_push() {
        let instr = Instruction::Push { src: Register(0) };
        assert!(matches!(instr, Instruction::Push { .. }));
    }

    /// EDGE CASE: Pop instruction
    #[test]
    fn test_edge_pop() {
        let instr = Instruction::Pop { dest: Register(0) };
        assert!(matches!(instr, Instruction::Pop { .. }));
    }

    /// EDGE CASE: Label instruction
    #[test]
    fn test_edge_label() {
        let instr = Instruction::Label { name: ".Lbb0".to_string() };
        assert!(matches!(instr, Instruction::Label { .. }));
    }

    // ==================== VALUE TESTS ====================

    /// EDGE CASE: Register value
    #[test]
    fn test_edge_value_reg() {
        let v = Value::Reg(Register(0));
        assert!(matches!(v, Value::Reg(_)));
    }

    /// EDGE CASE: Immediate value - zero
    #[test]
    fn test_edge_value_imm_zero() {
        let v = Value::Imm(0);
        assert!(matches!(v, Value::Imm(0)));
    }

    /// EDGE CASE: Immediate value - max
    #[test]
    fn test_edge_value_imm_max() {
        let v = Value::Imm(i64::MAX);
        assert!(matches!(v, Value::Imm(i64::MAX)));
    }

    /// EDGE CASE: Immediate value - min
    #[test]
    fn test_edge_value_imm_min() {
        let v = Value::Imm(i64::MIN);
        assert!(matches!(v, Value::Imm(i64::MIN)));
    }

    /// EDGE CASE: Label value
    #[test]
    fn test_edge_value_label() {
        let v = Value::Label("foo".to_string());
        assert!(matches!(v, Value::Label(_)));
    }

    // ==================== ADDRESS TESTS ====================

    /// EDGE CASE: Base address with zero offset
    #[test]
    fn test_edge_addr_base_zero() {
        let addr = Address::Base { base: Register(0), offset: 0 };
        assert!(matches!(addr, Address::Base { .. }));
    }

    /// EDGE CASE: Base address with large offset
    #[test]
    fn test_edge_addr_base_large() {
        let addr = Address::Base { base: Register(0), offset: i32::MAX };
        assert!(matches!(addr, Address::Base { .. }));
    }

    /// EDGE CASE: Indexed address
    #[test]
    fn test_edge_addr_indexed() {
        let addr = Address::Indexed {
            base: Register(0),
            index: Register(1),
            scale: 8,
            offset: 0,
        };
        assert!(matches!(addr, Address::Indexed { .. }));
    }

    /// EDGE CASE: Stack address - zero
    #[test]
    fn test_edge_addr_stack_zero() {
        let addr = Address::Stack(0);
        assert!(matches!(addr, Address::Stack(0)));
    }

    /// EDGE CASE: Stack address - negative
    #[test]
    fn test_edge_addr_stack_neg() {
        let addr = Address::Stack(-100);
        assert!(matches!(addr, Address::Stack(-100)));
    }

    /// EDGE CASE: Global address
    #[test]
    fn test_edge_addr_global() {
        let addr = Address::Global(faxc_util::Symbol::intern("global_var"));
        assert!(matches!(addr, Address::Global(_)));
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
        let _and = BinOp::And;
        let _or = BinOp::Or;
        let _xor = BinOp::Xor;
        let _shl = BinOp::Shl;
        let _shr = BinOp::Shr;
    }

    /// EDGE CASE: All unary operators
    #[test]
    fn test_edge_all_un_ops() {
        let _neg = UnOp::Neg;
        let _not = UnOp::Not;
    }

    /// EDGE CASE: All conditions
    #[test]
    fn test_edge_all_conditions() {
        let _eq = Condition::Eq;
        let _ne = Condition::Ne;
        let _lt = Condition::Lt;
        let _gt = Condition::Gt;
        let _le = Condition::Le;
        let _ge = Condition::Ge;
    }

    // ==================== REGISTER TESTS ====================

    /// EDGE CASE: Register zero
    #[test]
    fn test_edge_reg_zero() {
        let r = Register(0);
        assert_eq!(r.0, 0);
    }

    /// EDGE CASE: Register max
    #[test]
    fn test_edge_reg_max() {
        let r = Register(u32::MAX);
        assert_eq!(r.0, u32::MAX);
    }

    /// EDGE CASE: Many registers
    #[test]
    fn test_edge_many_registers() {
        let regs: Vec<_> = (0..256).map(|i| Register(i)).collect();
        assert_eq!(regs.len(), 256);
    }

    // ==================== ERROR CASES ====================

    /// ERROR CASE: Empty label name
    #[test]
    fn test_edge_empty_label() {
        let instr = Instruction::Label { name: "".to_string() };
        assert!(matches!(instr, Instruction::Label { .. }));
    }

    /// ERROR CASE: Empty target name
    #[test]
    fn test_edge_empty_target() {
        let instr = Instruction::Jmp { target: "".to_string() };
        assert!(matches!(instr, Instruction::Jmp { .. }));
    }

    /// EDGE CASE: Complex address mode
    #[test]
    fn test_edge_complex_addr() {
        let addr = Address::Indexed {
            base: Register(0),
            index: Register(1),
            scale: 4,
            offset: -100,
        };
        assert!(matches!(addr, Address::Indexed { .. }));
    }
}