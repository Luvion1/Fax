//! LIR (Low-level Intermediate Representation)
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 2
//! x86-64 instruction set with virtual register management.

use faxc_util::Symbol;

/// LIR Function with complete x86-64 support
#[derive(Debug)]
pub struct Function {
    pub name: Symbol,
    pub registers: Vec<VirtualRegister>,
    pub instructions: Vec<Instruction>,
    pub labels: Vec<(usize, String)>,
    pub frame_size: u32,
    pub param_count: usize,
    pub is_external: bool,
}

impl Function {
    pub fn new(name: Symbol) -> Self {
        Self {
            name,
            registers: Vec::new(),
            instructions: Vec::new(),
            labels: Vec::new(),
            frame_size: 0,
            param_count: 0,
            is_external: false,
        }
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }
}

/// Virtual Register with type information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualRegister {
    pub id: u32,
    pub width: RegisterWidth,
}

impl VirtualRegister {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            width: RegisterWidth::W64,
        }
    }

    pub fn with_width(id: u32, width: RegisterWidth) -> Self {
        Self { id, width }
    }
}

/// Register width for x86-64
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegisterWidth {
    W8,  // 8-bit (al, bl, cl, dl, etc.)
    W16, // 16-bit (ax, bx, cx, dx, etc.)
    W32, // 32-bit (eax, ebx, ecx, edx, etc.)
    W64, // 64-bit (rax, rbx, rcx, rdx, etc.)
}

/// Physical registers for x86-64 (System V AMD64 ABI)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalRegister {
    // General purpose registers
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    RBP,
    RSP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    // FP registers
    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
    // Special
    #[allow(non_camel_case_types)]
    RAX_RDX, // For 128-bit returns
    #[allow(non_camel_case_types)]
    XMM0_XMM1, // For FP returns
}

impl PhysicalRegister {
    /// Returns true if this register is caller-saved (volatile)
    pub fn is_caller_saved(&self) -> bool {
        matches!(
            self,
            PhysicalRegister::RAX
                | PhysicalRegister::RCX
                | PhysicalRegister::RDX
                | PhysicalRegister::RSI
                | PhysicalRegister::RDI
                | PhysicalRegister::R8
                | PhysicalRegister::R9
                | PhysicalRegister::R10
                | PhysicalRegister::R11
                | PhysicalRegister::XMM0
                | PhysicalRegister::XMM1
                | PhysicalRegister::XMM2
                | PhysicalRegister::XMM3
                | PhysicalRegister::XMM4
                | PhysicalRegister::XMM5
                | PhysicalRegister::XMM6
                | PhysicalRegister::XMM7
        )
    }

    /// Returns true if this register is callee-saved (non-volatile)
    pub fn is_callee_saved(&self) -> bool {
        matches!(
            self,
            PhysicalRegister::RBX
                | PhysicalRegister::RBP
                | PhysicalRegister::RSP
                | PhysicalRegister::R12
                | PhysicalRegister::R13
                | PhysicalRegister::R14
                | PhysicalRegister::R15
        )
    }
}

/// x86-64 Instruction Set (complete)
#[derive(Debug, Clone)]
pub enum Instruction {
    // Data movement
    Nop,
    Mov {
        dest: Operand,
        src: Operand,
    },
    Movsx {
        dest: Operand,
        src: Operand,
        sign_extend: bool,
    }, // Sign/zero extend move
    Movzx {
        dest: Operand,
        src: Operand,
    },
    Lea {
        dest: Operand,
        addr: Address,
    },
    Push {
        src: Operand,
    },
    Pop {
        dest: Operand,
    },
    Xchg {
        dest: Operand,
        src: Operand,
    },
    Cmov {
        cond: Condition,
        dest: Operand,
        src: Operand,
    }, // Conditional move

    // Load/Store
    Load {
        dest: Operand,
        addr: Address,
        width: RegisterWidth,
    },
    Store {
        addr: Address,
        src: Operand,
        width: RegisterWidth,
    },

    // Arithmetic
    Add {
        dest: Operand,
        src: Operand,
    },
    Sub {
        dest: Operand,
        src: Operand,
    },
    Mul {
        dest: Operand,
        src: Operand,
        signed: bool,
    },
    Idiv {
        dest: Operand,
        src: Operand,
    }, // Signed divide
    IdivUnsigned {
        dest: Operand,
        src: Operand,
    },
    Imul {
        dest: Operand,
        src1: Operand,
        src2: Option<Operand>,
    }, // Signed multiply
    Inc {
        dest: Operand,
    },
    Dec {
        dest: Operand,
    },
    Neg {
        dest: Operand,
    },

    // Division with remainder
    Div {
        divisor: Operand,
    }, // RDX:RAX / divisor
    IdivSigned {
        divisor: Operand,
    },

    // Bitwise
    And {
        dest: Operand,
        src: Operand,
    },
    Or {
        dest: Operand,
        src: Operand,
    },
    Xor {
        dest: Operand,
        src: Operand,
    },
    Not {
        dest: Operand,
    },
    Shl {
        dest: Operand,
        count: Operand,
    },
    Shr {
        dest: Operand,
        count: Operand,
    }, // Logical shift right
    Sar {
        dest: Operand,
        count: Operand,
    }, // Arithmetic shift right
    Rol {
        dest: Operand,
        count: Operand,
    },
    Ror {
        dest: Operand,
        count: Operand,
    },

    // Comparison
    Cmp {
        src1: Operand,
        src2: Operand,
    },
    Test {
        src1: Operand,
        src2: Operand,
    },

    // Control flow
    Jmp {
        target: Label,
    },
    Jcc {
        cond: Condition,
        target: Label,
    },
    Call {
        target: CallTarget,
    },
    Ret {
        value: Option<Operand>,
    },
    Label {
        name: Label,
    },

    // Stack frame
    EnterFrame {
        frame_size: u32,
    },
    LeaveFrame,
    Alloca {
        dest: Operand,
        size: Operand,
    },

    // System V AMD64 ABI specific
    SaveCalleeSaved {
        regs: Vec<PhysicalRegister>,
    },
    RestoreCalleeSaved {
        regs: Vec<PhysicalRegister>,
    },
}

/// Call target
#[derive(Debug, Clone)]
pub enum CallTarget {
    Direct(Symbol),
    Indirect(Operand),
    External(Symbol),
}

/// Label for jump targets
pub type Label = String;

/// Operand for instructions
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Reg(VirtualRegister),
    PhysReg(PhysicalRegister),
    Imm(i64),
    Mem(Address),
    Label(Label),
}

/// Memory addressing modes (x86-64)
#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    /// [base]
    Base { base: PhysicalRegister },
    /// [base + offset]
    BaseOffset { base: PhysicalRegister, offset: i32 },
    /// [base + index*scale + offset]
    Indexed {
        base: PhysicalRegister,
        index: PhysicalRegister,
        scale: u8, // 1, 2, 4, or 8
        offset: i32,
    },
    /// RIP-relative: [rip + offset]
    RipRelative { offset: i32, symbol: Option<Symbol> },
    /// Stack relative: [rbp + offset]
    StackRelative { offset: i32 },
    /// Absolute address
    Absolute(u64),
    /// Global symbol
    Global(Symbol),
}

/// Condition codes for conditional jumps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    // Equality
    Eq, // Equal (ZF=1)
    Ne, // Not equal (ZF=0)
    // Unsigned comparisons
    B,  // Below (CF=1)
    Ae, // Above or equal (CF=0)
    A,  // Above (CF=0 && ZF=0)
    Be, // Below or equal (CF=1 || ZF=1)
    // Signed comparisons
    L,  // Less (SF!=OF)
    Ge, // Greater or equal (SF=OF)
    G,  // Greater (ZF=0 && SF=OF)
    Le, // Less or equal (ZF=1 || SF!=OF)
    // Special
    O,  // Overflow (OF=1)
    No, // No overflow (OF=0)
    S,  // Sign (SF=1)
    Ns, // No sign (SF=0)
    P,  // Parity (PF=1)
    Np, // No parity (PF=0)
}

/// Binary operations for simplified instruction encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Sar,
}

#[cfg(test)]
mod lir_tests {
    use super::*;

    #[test]
    fn test_virtual_register_creation() {
        let reg = VirtualRegister::new(0);
        assert_eq!(reg.id, 0);
        assert_eq!(reg.width, RegisterWidth::W64);
    }

    #[test]
    fn test_physical_register_properties() {
        assert!(PhysicalRegister::RAX.is_caller_saved());
        assert!(PhysicalRegister::RBX.is_callee_saved());
        assert!(!PhysicalRegister::RAX.is_callee_saved());
    }

    #[test]
    fn test_function_creation() {
        let name = Symbol::intern("test");
        let func = Function::new(name);
        assert_eq!(func.name, name);
        assert_eq!(func.instruction_count(), 0);
    }
}
