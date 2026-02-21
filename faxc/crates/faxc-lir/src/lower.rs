use crate::lir::*;
use faxc_mir as mir;
use faxc_util::Symbol;
use std::collections::HashMap;

use faxc_util::Idx;

pub fn lower_mir_to_lir(mir_fn: &mir::Function) -> Function {
    let mut lowerer = LirLowerer::new(mir_fn.name.clone());
    for i in 0..mir_fn.blocks.len() {
        let block = &mir_fn.blocks[mir::BlockId::from_usize(i)];
        lowerer.lower_block(block);
    }
    lowerer.finish()
}

pub struct LirLowerer {
    pub function: Function,
    pub register_counter: u32,
    pub label_counter: u32,
    pub mir_to_lir_reg: HashMap<mir::LocalId, Register>,
}

impl LirLowerer {
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
            mir_to_lir_reg: HashMap::new(),
        }
    }

    pub fn new_reg(&mut self) -> Register {
        let reg = Register(self.register_counter);
        self.register_counter += 1;
        self.function.registers.push(reg);
        reg
    }

    pub fn lower_block(&mut self, block: &mir::BasicBlock) {
        let label = format!(".Lbb{}", block.id.0);
        self.function.instructions.push(Instruction::Label { name: label });
        
        for stmt in &block.statements {
            if let mir::Statement::Assign(place, rvalue) = stmt {
                let dest = self.get_place_reg(place);
                self.lower_rvalue(dest, rvalue);
            }
        }
        self.lower_terminator(&block.terminator);
    }

    fn lower_rvalue(&mut self, dest: Register, rvalue: &mir::Rvalue) {
        match rvalue {
            mir::Rvalue::Use(operand) => {
                let src = self.lower_operand(operand);
                self.function.instructions.push(Instruction::Mov { dest, src });
            }
            mir::Rvalue::BinaryOp(op, left, right) => {
                let src1 = self.lower_operand_to_reg(left);
                let src2 = self.lower_operand(right);
                self.function.instructions.push(Instruction::BinOp {
                    op: convert_binop(*op),
                    dest,
                    src1,
                    src2,
                });
            }
            _ => {}
        }
    }

    fn lower_operand(&mut self, operand: &mir::Operand) -> Value {
        match operand {
            mir::Operand::Copy(p) | mir::Operand::Move(p) => Value::Reg(self.get_place_reg(p)),
            mir::Operand::Constant(c) => match &c.kind {
                mir::ConstantKind::Int(n) => Value::Imm(*n),
                _ => Value::Imm(0),
            },
        }
    }

    fn lower_operand_to_reg(&mut self, operand: &mir::Operand) -> Register {
        match self.lower_operand(operand) {
            Value::Reg(r) => r,
            Value::Imm(i) => {
                let reg = self.new_reg();
                self.function.instructions.push(Instruction::Mov { dest: reg, src: Value::Imm(i) });
                reg
            }
            _ => self.new_reg(),
        }
    }

    fn get_place_reg(&mut self, place: &mir::Place) -> Register {
        match place {
            mir::Place::Local(id) => {
                if let Some(reg) = self.mir_to_lir_reg.get(id) {
                    *reg
                } else {
                    let reg = self.new_reg();
                    self.mir_to_lir_reg.insert(*id, reg);
                    reg
                }
            }
            _ => self.new_reg(),
        }
    }

    fn lower_terminator(&mut self, term: &mir::Terminator) {
        match term {
            mir::Terminator::Return => self.function.instructions.push(Instruction::Ret),
            mir::Terminator::Goto { target } => {
                self.function.instructions.push(Instruction::Jmp { target: format!(".Lbb{}", target.0) });
            }
            mir::Terminator::If { cond, then_block, else_block } => {
                let cond_reg = self.lower_operand_to_reg(cond);
                // Simple comparison against zero/false
                self.function.instructions.push(Instruction::Cmp { 
                    src1: cond_reg, 
                    src2: Value::Imm(1) 
                });
                self.function.instructions.push(Instruction::Jcc { 
                    cond: Condition::Eq, 
                    target: format!(".Lbb{}", then_block.0) 
                });
                self.function.instructions.push(Instruction::Jmp { 
                    target: format!(".Lbb{}", else_block.0) 
                });
            }
            _ => {}
        }
    }

    pub fn finish(self) -> Function { self.function }
}

fn convert_binop(op: mir::BinOp) -> BinOp {
    match op {
        mir::BinOp::Add => BinOp::Add,
        mir::BinOp::Sub => BinOp::Sub,
        mir::BinOp::Mul => BinOp::Mul,
        mir::BinOp::Div => BinOp::Div,
        mir::BinOp::Rem => BinOp::Rem,
        mir::BinOp::BitAnd => BinOp::And,
        mir::BinOp::BitOr => BinOp::Or,
        mir::BinOp::BitXor => BinOp::Xor,
        mir::BinOp::Shl => BinOp::Shl,
        mir::BinOp::Shr => BinOp::Shr,
        mir::BinOp::Eq | mir::BinOp::Ne | mir::BinOp::Lt | mir::BinOp::Le | mir::BinOp::Gt | mir::BinOp::Ge => {
            // Comparisons in LIR are handled by Cmp + Jcc or explicit BinOp
            // For now, map to Add as placeholder but they should trigger Cmp logic elsewhere
            BinOp::Add 
        }
        _ => BinOp::Add,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use faxc_mir::{Builder, BlockId};
    use faxc_sem::Type;
    use faxc_util::DefId;
    use faxc_util::Symbol;

    #[test]
    fn test_mir_to_lir_basic() {
        let name = Symbol::intern("test_fn");
        let mut builder = Builder::new(name, Type::Int);
        
        let entry = builder.new_block();
        builder.set_current_block(entry);
        
        // let x = 5;
        let x_local = builder.add_local(Type::Int, None);
        let x_place = mir::Place::Local(x_local);
        builder.assign(x_place.clone(), mir::Rvalue::Use(mir::Operand::Constant(mir::Constant {
            ty: Type::Int,
            kind: mir::ConstantKind::Int(5),
        })));
        
        // return x;
        builder.terminator(mir::Terminator::Return);
        
        let mir_fn = builder.build();
        let lir_fn = lower_mir_to_lir(&mir_fn);
        
        assert_eq!(lir_fn.name.as_str(), "test_fn");
        // Should have at least one instruction (Mov or Ret)
        assert!(!lir_fn.instructions.is_empty());
    }
}
