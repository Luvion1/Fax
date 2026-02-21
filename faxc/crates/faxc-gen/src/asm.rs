use faxc_lir::{Function as LirFunction, Instruction, Value, Register, BinOp, Condition};
use std::collections::HashMap;

pub struct AsmGenerator {
    pub output: String,
    pub indent: usize,
    pub reg_alloc: RegisterAllocator,
}

pub struct RegisterAllocator {
    pub allocation: HashMap<Register, Location>,
    pub frame_size: u32,
}

pub enum Location {
    PhysReg(String),
    Stack(i32),
}

impl AsmGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            reg_alloc: RegisterAllocator {
                allocation: HashMap::new(),
                frame_size: 0,
            },
        }
    }

    pub fn generate_function(&mut self, func: &LirFunction) {
        // Assembly generation logic
    }
}
