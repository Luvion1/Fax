use faxc_util::Symbol;

/// LIR Function
#[derive(Debug)]
pub struct Function {
    pub name: Symbol,
    pub registers: Vec<Register>,
    pub instructions: Vec<Instruction>,
    pub labels: Vec<(usize, String)>,
    pub frame_size: u32,
}

/// Virtual Register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Register(pub u32);

/// Instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    Nop,
    Mov { dest: Register, src: Value },
    Load { dest: Register, addr: Address },
    Store { addr: Address, src: Register },
    Lea { dest: Register, addr: Address },
    BinOp { op: BinOp, dest: Register, src1: Register, src2: Value },
    UnOp { op: UnOp, dest: Register, src: Register },
    Cmp { src1: Register, src2: Value },
    Jmp { target: String },
    Jcc { cond: Condition, target: String },
    Call { func: Value },
    Ret,
    Push { src: Register },
    Pop { dest: Register },
    Label { name: String },
}

#[derive(Debug, Clone)]
pub enum Value {
    Reg(Register),
    Imm(i64),
    Label(String),
}

#[derive(Debug, Clone)]
pub enum Address {
    Base { base: Register, offset: i32 },
    Indexed { base: Register, index: Register, scale: u8, offset: i32 },
    Stack(i32),
    Global(Symbol),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp { Add, Sub, Mul, Div, Rem, And, Or, Xor, Shl, Shr }

#[derive(Debug, Clone, Copy)]
pub enum UnOp { Neg, Not }

#[derive(Debug, Clone, Copy)]
pub enum Condition { Eq, Ne, Lt, Gt, Le, Ge }
