//! faxc-lir - Low-Level Intermediate Representation (LIR)
//!
//! ============================================================================
//! LIR OVERVIEW
//! ============================================================================
//!
//! LIR adalah representasi low-level yang sangat dekat dengan machine code.
//! LIR berada satu level di atas assembly dan merupakan bentuk terakhir
//! dari intermediate representation sebelum code generation.
//!
//! KARAKTERISTIK LIR:
//! ------------------
//!
//! 1. MACHINE-ORIENTED
//!    Instruksi sangat mirip dengan CPU instructions (add, sub, mov, etc.)
//!
//! 2. REGISTER-BASED
//!    Menggunakan virtual registers (belum dialokasikan ke physical registers)
//!
//! 3. LINEAR CONTROL FLOW
//!    Control flow menggunakan labels dan jumps
//!
//! 4. PLATFORM-ABSTRACT
//!    Tidak terikat ke architecture tertentu (x86, ARM, etc.)
//!    Tapi lebih low-level daripada MIR
//!
//! ============================================================================
//! DARI MIR KE LIR
//! ============================================================================
//!
//! Lowering MIR constructs ke LIR primitives:
//!
//! PHI NODES -> PARALLEL COPIES
//! -----------------------------
//! MIR menggunakan PHI nodes untuk merge values dari berbagai predecessors.
//! LIR mengubah ini menjadi parallel copies di masing-masing predecessor.
//!
//! MIR:
//! ```
//! BB3:
//!   x3 = φ(x1: BB1, x2: BB2)
//! ```
//!
//! LIR:
//! ```
//! BB1:
//!   ...
//!   copy x3, x1
//!   goto BB3
//!
//! BB2:
//!   ...
//!   copy x3, x2
//!   goto BB3
//!
//! BB3:
//!   ...
//! ```
//!
//! FUNCTION CALLS -> STACK OPERATIONS
//! ----------------------------------
//! MIR:
//! ```
//! result = call foo(arg1, arg2)
//! ```
//!
//! LIR:
//! ```
//!   push arg2              // Stack arguments (right to left)
//!   push arg1
//!   call foo               // Call instruction
//!   add sp, 16             // Cleanup stack
//!   copy result, rax       // Return value di register
//! ```
//!
//! CONTROL FLOW -> LABELS & JUMPS
//! ------------------------------
//! MIR:
//! ```
//! if cond then BB1 else BB2
//! ```
//!
//! LIR:
//! ```
//!   cmp cond, 0            // Compare dengan zero
//!   je BB2                 // Jump if equal (false)
//!   jmp BB1                // Jump ke then
//! ```
//!
//! ============================================================================
//! LIR INSTRUCTION SET
//! ============================================================================
//!
//! LIR menggunakan load-store architecture dengan tiga-operand format.
//!
//! CATEGORIES:
//!
//! 1. DATA MOVEMENT
//!    - mov dest, src              // Register to register
//!    - load dest, [addr]          // Load dari memory
//!    - store [addr], src          // Store ke memory
//!    - lea dest, [addr]           // Load effective address
//!
//! 2. ARITHMETIC
//!    - add dest, src1, src2
//!    - sub dest, src1, src2
//!    - mul dest, src1, src2
//!    - div dest, src1, src2
//!    - mod dest, src1, src2
//!    - neg dest, src
//!
//! 3. LOGICAL
//!    - and dest, src1, src2
//!    - or dest, src1, src2
//!    - xor dest, src1, src2
//!    - not dest, src
//!    - shl dest, src, amount
//!    - shr dest, src, amount
//!
//! 4. COMPARISON
//!    - cmp src1, src2             // Set flags
//!    - test src1, src2            // Logical AND dan set flags
//!
//! 5. CONTROL FLOW
//!    - jmp label                  // Unconditional jump
//!    - je label                   // Jump if equal
//!    - jne label                  // Jump if not equal
//!    - jl label                   // Jump if less
//!    - jg label                   // Jump if greater
//!    - jle label                  // Jump if less or equal
//!    - jge label                  // Jump if greater or equal
//!    - call function
//!    - ret                        // Return
//!
//! 6. STACK OPERATIONS
//!    - push src
//!    - pop dest
//!
//! ============================================================================
//! VIRTUAL REGISTERS
//! ============================================================================
//!
//! LIR menggunakan virtual registers (v0, v1, v2, ...) dalam jumlah tak terbatas.
//! Register allocation pass kemudian akan memetakan ke physical registers.
//!
//! VIRTUAL REGISTER PROPERTIES:
//! - Unlimited quantity
//! - Single assignment (SSA-like)
//! - Typed (int, float, pointer)
//!
//! PHYSICAL REGISTERS (x86-64 example):
//! ------------------------------------
//! General Purpose:
//! - RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP
//! - R8, R9, R10, R11, R12, R13, R14, R15
//!
//! Special Purpose:
//! - RAX: Return value, accumulator
//! - RDI, RSI, RDX, RCX, R8, R9: Arguments (System V AMD64 ABI)
//! - RSP: Stack pointer
//! - RBP: Base pointer
//!
//! CALLING CONVENTION (System V AMD64 ABI):
//! ----------------------------------------
//! Arguments: RDI, RSI, RDX, RCX, R8, R9, kemudian stack
//! Return: RAX (RDX untuk 128-bit)
//! Callee-saved: RBX, RBP, R12-R15 (harus dipreserve)
//! Caller-saved: RAX, RCX, RDX, RSI, RDI, R8-R11 (bisa diubah)

use faxc_mir::{Function as MirFunction, Type};
use faxc_util::Symbol;

/// LIR Function
#[derive(Debug)]
pub struct Function {
    /// Function name
    pub name: Symbol,
    
    /// Virtual registers
    pub registers: Vec<Register>,
    
    /// Instructions
    pub instructions: Vec<Instruction>,
    
    /// Labels (instruction index -> label)
    pub labels: Vec<(usize, String)>,
    
    /// Stack frame size
    pub frame_size: u32,
}

/// Virtual Register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Register(pub u32);

impl Register {
    pub fn new(id: u32) -> Self { Register(id) }
}

/// Instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    /// No operation
    Nop,
    
    /// Move value
    Mov { dest: Register, src: Value },
    
    /// Load from memory
    Load { dest: Register, addr: Address },
    
    /// Store to memory
    Store { addr: Address, src: Register },
    
    /// Load effective address
    Lea { dest: Register, addr: Address },
    
    /// Binary operation
    BinOp { op: BinOp, dest: Register, src1: Register, src2: Value },
    
    /// Unary operation
    UnOp { op: UnOp, dest: Register, src: Register },
    
    /// Compare
    Cmp { src1: Register, src2: Value },
    
    /// Jump
    Jmp { target: String },
    
    /// Conditional jump
    Jcc { cond: Condition, target: String },
    
    /// Call function
    Call { func: Value },
    
    /// Return
    Ret,
    
    /// Push to stack
    Push { src: Register },
    
    /// Pop from stack
    Pop { dest: Register },
    
    /// Label definition
    Label { name: String },
}

/// Value operand
#[derive(Debug, Clone)]
pub enum Value {
    /// Register
    Reg(Register),
    
    /// Immediate constant
    Imm(i64),
    
    /// Label address
    Label(String),
}

/// Memory address
#[derive(Debug, Clone)]
pub enum Address {
    /// Base register + offset
    Base { base: Register, offset: i32 },
    
    /// Base + index*scale + offset
    Indexed { base: Register, index: Register, scale: u8, offset: i32 },
    
    /// Stack offset
    Stack(i32),
    
    /// Global symbol
    Global(Symbol),
}

/// Binary operations
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    And, Or, Xor,
    Shl, Shr,
}

/// Unary operations
#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg, Not,
}

/// Condition codes
#[derive(Debug, Clone, Copy)]
pub enum Condition {
    Eq, Ne, Lt, Gt, Le, Ge,
}

/// Lower MIR function to LIR
pub fn lower_mir_to_lir(mir_fn: &MirFunction) -> Function {
    let mut lowerer = LirLowerer::new(mir_fn.name.clone());
    
    // Lower each basic block
    for block in mir_fn.blocks.iter() {
        lowerer.lower_block(block);
    }
    
    lowerer.finish()
}

/// MIR to LIR lowerer
pub struct LirLowerer {
    function: Function,
    register_counter: u32,
    label_counter: u32,
    mir_to_lir_reg: std::collections::HashMap<faxc_mir::LocalId, Register>,
}

impl LirLowerer {
    /// Create new lowerer
    pub fn new(name: Symbol) -> Self {
        Self {
            function: Function {
                name,
                registers: Vec::new(),
                instructions: Vec::new(),
                labels: Vec::new(),
                frame_size: 0,
            },
            register_counter: 0,
            label_counter: 0,
            mir_to_lir_reg: std::collections::HashMap::new(),
        }
    }
    
    /// Create new virtual register
    pub fn new_reg(&mut self) -> Register {
        let reg = Register(self.register_counter);
        self.register_counter += 1;
        self.function.registers.push(reg);
        reg
    }
    
    /// Create new label
    pub fn new_label(&mut self, prefix: &str) -> String {
        let label = format!(".L{}{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }
    
    /// Emit instruction
    pub fn emit(&mut self, insn: Instruction) {
        self.function.instructions.push(insn);
    }
    
    /// Lower MIR basic block
    pub fn lower_block(&mut self, block: &faxc_mir::BasicBlock) {
        // Add label for block
        let label = self.new_label("bb");
        self.emit(Instruction::Label { name: label });
        
        // Lower statements
        for stmt in &block.statements {
            self.lower_statement(stmt);
        }
        
        // Lower terminator
        self.lower_terminator(&block.terminator);
    }
    
    /// Lower MIR statement
    fn lower_statement(&mut self, stmt: &faxc_mir::Statement) {
        use faxc_mir::Statement;
        
        match stmt {
            Statement::Assign(place, rvalue) => {
                // Get register for place
                let dest = self.get_place_reg(place);
                
                // Lower rvalue
                self.lower_rvalue(dest, rvalue);
            }
            _ => {}
        }
    }
    
    /// Lower Rvalue ke LIR
    fn lower_rvalue(&mut self, dest: Register, rvalue: &faxc_mir::Rvalue) {
        use faxc_mir::Rvalue;
        
        match rvalue {
            Rvalue::Use(operand) => {
                let src = self.lower_operand(operand);
                self.emit(Instruction::Mov { dest, src });
            }
            Rvalue::BinaryOp(op, left, right) => {
                let left_reg = self.lower_operand_to_reg(left);
                let right_val = self.lower_operand(right);
                
                self.emit(Instruction::BinOp {
                    op: convert_binop(*op),
                    dest,
                    src1: left_reg,
                    src2: right_val,
                });
            }
            _ => unimplemented!("Rvalue lowering"),
        }
    }
    
    /// Lower operand
    fn lower_operand(&mut self, operand: &faxc_mir::Operand) -> Value {
        match operand {
            faxc_mir::Operand::Copy(place) |
            faxc_mir::Operand::Move(place) => {
                Value::Reg(self.get_place_reg(place))
            }
            faxc_mir::Operand::Constant(c) => {
                match &c.kind {
                    faxc_mir::ConstantKind::Int(n) => Value::Imm(*n),
                    _ => unimplemented!("Constant lowering"),
                }
            }
        }
    }
    
    /// Lower operand to register
    fn lower_operand_to_reg(&mut self, operand: &faxc_mir::Operand) -> Register {
        match self.lower_operand(operand) {
            Value::Reg(reg) => reg,
            Value::Imm(imm) => {
                let reg = self.new_reg();
                self.emit(Instruction::Mov {
                    dest: reg,
                    src: Value::Imm(imm),
                });
                reg
            }
            _ => panic!("Cannot lower label to register directly"),
        }
    }
    
    /// Get register for place
    fn get_place_reg(&mut self, place: &faxc_mir::Place) -> Register {
        match place {
            faxc_mir::Place::Local(local) => {
                *self.mir_to_lir_reg.entry(*local).or_insert_with(|| self.new_reg())
            }
            _ => unimplemented!("Projection lowering"),
        }
    }
    
    /// Lower terminator
    fn lower_terminator(&mut self, term: &faxc_mir::Terminator) {
        use faxc_mir::Terminator;
        
        match term {
            Terminator::Goto { target } => {
                let label = format!(".Lbb{}", target.0);
                self.emit(Instruction::Jmp { target: label });
            }
            Terminator::If { cond, then_block, else_block } => {
                let cond_reg = self.lower_operand_to_reg(cond);
                self.emit(Instruction::Cmp { src1: cond_reg, src2: Value::Imm(0) });
                
                let else_label = format!(".Lbb{}", else_block.0);
                self.emit(Instruction::Jcc { cond: Condition::Eq, target: else_label });
                
                let then_label = format!(".Lbb{}", then_block.0);
                self.emit(Instruction::Jmp { target: then_label });
            }
            Terminator::Return => {
                self.emit(Instruction::Ret);
            }
            _ => unimplemented!("Terminator lowering"),
        }
    }
    
    /// Finish lowering
    pub fn finish(self) -> Function {
        self.function
    }
}

/// Convert MIR binary op to LIR binary op
fn convert_binop(op: faxc_mir::BinOp) -> BinOp {
    match op {
        faxc_mir::BinOp::Add => BinOp::Add,
        faxc_mir::BinOp::Sub => BinOp::Sub,
        faxc_mir::BinOp::Mul => BinOp::Mul,
        faxc_mir::BinOp::Div => BinOp::Div,
        faxc_mir::BinOp::Rem => BinOp::Rem,
        faxc_mir::BinOp::BitAnd => BinOp::And,
        faxc_mir::BinOp::BitOr => BinOp::Or,
        _ => unimplemented!("Binary op conversion"),
    }
}
