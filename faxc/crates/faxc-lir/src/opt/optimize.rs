//! LIR Optimization Passes

use crate::lir::*;

pub fn optimize_function(func: &mut Function) {
    let mut changed = true;
    let mut iterations = 0;
    let max_iterations = 10;

    while changed && iterations < max_iterations {
        changed = false;
        changed |= peephole(func);
        changed |= constant_fold(func);
        changed |= copy_propagation(func);
        changed |= dead_code(func);
        iterations += 1;
    }
}

fn constant_fold(func: &mut Function) -> bool {
    let mut changed = false;

    for inst in &mut func.instructions {
        match inst {
            Instruction::Add { dest, src } => {
                if let (Operand::Imm(a), Operand::Imm(b)) = (dest.clone(), src.clone()) {
                    *inst = Instruction::Mov {
                        dest: dest.clone(),
                        src: Operand::Imm(a.wrapping_add(b)),
                    };
                    changed = true;
                }
            },
            Instruction::Sub { dest, src } => {
                if let (Operand::Imm(a), Operand::Imm(b)) = (dest.clone(), src.clone()) {
                    *inst = Instruction::Mov {
                        dest: dest.clone(),
                        src: Operand::Imm(a.wrapping_sub(b)),
                    };
                    changed = true;
                }
            },
            Instruction::Mul {
                dest,
                src,
                signed: _,
            } => {
                if let (Operand::Imm(a), Operand::Imm(b)) = (dest.clone(), src.clone()) {
                    let result: i64 = (a as i64).wrapping_mul(b as i64);
                    *inst = Instruction::Mov {
                        dest: dest.clone(),
                        src: Operand::Imm(result),
                    };
                    changed = true;
                }
            },
            Instruction::And { dest, src } => {
                if let (Operand::Imm(a), Operand::Imm(b)) = (dest.clone(), src.clone()) {
                    *inst = Instruction::Mov {
                        dest: dest.clone(),
                        src: Operand::Imm(a & b),
                    };
                    changed = true;
                }
            },
            Instruction::Or { dest, src } => {
                if let (Operand::Imm(a), Operand::Imm(b)) = (dest.clone(), src.clone()) {
                    *inst = Instruction::Mov {
                        dest: dest.clone(),
                        src: Operand::Imm(a | b),
                    };
                    changed = true;
                }
            },
            Instruction::Xor { dest, src } => {
                if let (Operand::Imm(a), Operand::Imm(b)) = (dest.clone(), src.clone()) {
                    *inst = Instruction::Mov {
                        dest: dest.clone(),
                        src: Operand::Imm(a ^ b),
                    };
                    changed = true;
                }
            },
            Instruction::Shl { dest, count } => {
                if let (Operand::Imm(a), Operand::Imm(b)) = (dest.clone(), count.clone()) {
                    *inst = Instruction::Mov {
                        dest: dest.clone(),
                        src: Operand::Imm(a << b),
                    };
                    changed = true;
                }
            },
            Instruction::Shr { dest, count } => {
                if let (Operand::Imm(a), Operand::Imm(b)) = (dest.clone(), count.clone()) {
                    *inst = Instruction::Mov {
                        dest: dest.clone(),
                        src: Operand::Imm(a >> b),
                    };
                    changed = true;
                }
            },
            _ => {},
        }
    }

    changed
}

fn copy_propagation(func: &mut Function) -> bool {
    let mut changed = false;
    let mut replacements: Vec<(usize, Operand, Operand)> = Vec::new();

    let len = func.instructions.len();
    for i in 0..len {
        if let Instruction::Mov { dest, src } = &func.instructions[i] {
            for j in (i + 1)..len {
                if let Instruction::Add { dest: _d, src: s } = &func.instructions[j] {
                    if s == src {
                        replacements.push((j, s.clone(), dest.clone()));
                        changed = true;
                    }
                } else if let Instruction::Sub { dest: _d, src: s } = &func.instructions[j] {
                    if s == src {
                        replacements.push((j, s.clone(), dest.clone()));
                        changed = true;
                    }
                } else if let Instruction::Mul {
                    dest: _d, src: s, ..
                } = &func.instructions[j]
                {
                    if s == src {
                        replacements.push((j, s.clone(), dest.clone()));
                        changed = true;
                    }
                }
            }
        }
    }

    for (idx, old, new) in replacements {
        if let Instruction::Add { dest: _, src } = &mut func.instructions[idx] {
            if *src == old {
                *src = new;
            }
        } else if let Instruction::Sub { dest: _, src } = &mut func.instructions[idx] {
            if *src == old {
                *src = new;
            }
        } else if let Instruction::Mul { dest: _, src, .. } = &mut func.instructions[idx] {
            if *src == old {
                *src = new;
            }
        }
    }

    changed
}

fn peephole(func: &mut Function) -> bool {
    let mut changed = false;
    let mut insts = Vec::new();
    let mut i = 0;

    while i < func.instructions.len() {
        let window = &func.instructions[i..];
        let (replaced, new_insts) = optimize_window(window, &func.registers);

        if !new_insts.is_empty() {
            changed = true;
            insts.extend(new_insts);
        } else {
            insts.push(func.instructions[i].clone());
        }

        i += if replaced { 2 } else { 1 };
    }

    if changed {
        func.instructions = insts;
    }

    changed
}

fn optimize_window(
    window: &[Instruction],
    _registers: &[VirtualRegister],
) -> (bool, Vec<Instruction>) {
    if window.len() < 2 {
        return (false, Vec::new());
    }

    let inst0 = &window[0];
    let inst1 = &window[1];

    if let Some(optimized) = try_optimize_pair(inst0, inst1) {
        return (true, optimized);
    }

    if let Some(optimized) = try_optimize_single(inst0) {
        return (false, vec![optimized]);
    }

    (false, Vec::new())
}

fn try_optimize_single(inst: &Instruction) -> Option<Instruction> {
    match inst {
        Instruction::Add { dest, src } => {
            if let Operand::Imm(0) = src {
                return Some(Instruction::Nop);
            }
            if let Operand::Imm(1) = src {
                return Some(Instruction::Inc { dest: dest.clone() });
            }
        },
        Instruction::Sub { dest, src } => {
            if let Operand::Imm(0) = src {
                return Some(Instruction::Nop);
            }
            if let Operand::Imm(1) = src {
                return Some(Instruction::Dec { dest: dest.clone() });
            }
        },
        Instruction::Mul {
            dest,
            src,
            signed: _,
        } => {
            if let Operand::Imm(1) = src {
                return Some(Instruction::Nop);
            }
            if let Operand::Imm(0) = src {
                return Some(Instruction::Mov {
                    dest: dest.clone(),
                    src: Operand::Imm(0),
                });
            }
        },
        Instruction::Shl { dest, count } => {
            if let Operand::Imm(1) = count {
                return Some(Instruction::Add {
                    dest: dest.clone(),
                    src: dest.clone(),
                });
            }
            if let Operand::Imm(0) = count {
                return Some(Instruction::Nop);
            }
        },
        Instruction::And { dest, src } => {
            if let Operand::Imm(-1) = src {
                return Some(Instruction::Nop);
            }
            if let Operand::Imm(0) = src {
                return Some(Instruction::Mov {
                    dest: dest.clone(),
                    src: Operand::Imm(0),
                });
            }
        },
        Instruction::Or { dest, src } => {
            if let Operand::Imm(0) = src {
                return Some(Instruction::Nop);
            }
            if let Operand::Imm(-1) = src {
                return Some(Instruction::Mov {
                    dest: dest.clone(),
                    src: Operand::Imm(-1),
                });
            }
        },
        Instruction::Xor { dest, src } => {
            if dest == src {
                return Some(Instruction::Mov {
                    dest: dest.clone(),
                    src: Operand::Imm(0),
                });
            }
        },
        Instruction::Mov { dest, src } => {
            if dest == src {
                return Some(Instruction::Nop);
            }
            if let Operand::Imm(0) = src {
                return Some(Instruction::Xor {
                    dest: dest.clone(),
                    src: dest.clone(),
                });
            }
        },
        _ => {},
    }

    None
}

fn try_optimize_pair(inst0: &Instruction, inst1: &Instruction) -> Option<Vec<Instruction>> {
    #[allow(unreachable_patterns)]
    match (inst0, inst1) {
        (Instruction::Mov { dest: _d1, src: s1 }, Instruction::Mov { dest: d2, src: s2 }) => {
            if let Operand::Reg(vr1) = s1 {
                if let Operand::Reg(vr2) = s2 {
                    if vr1 == vr2 {
                        return Some(vec![Instruction::Mov {
                            dest: d2.clone(),
                            src: s1.clone(),
                        }]);
                    }
                }
            }
        },
        (Instruction::Mov { dest, src }, Instruction::Add { dest: d2, src: s2 }) => {
            if dest == d2 {
                if let Operand::Imm(0) = src {
                    return Some(vec![Instruction::Add {
                        dest: d2.clone(),
                        src: s2.clone(),
                    }]);
                }
                if dest == s2 {
                    return Some(vec![Instruction::Add {
                        dest: d2.clone(),
                        src: src.clone(),
                    }]);
                }
            }
        },
        (Instruction::Mov { dest, src: _ }, Instruction::Cmp { src1, src2 }) => {
            if let Operand::Imm(0) = src1 {
                return Some(vec![Instruction::Test {
                    src1: dest.clone(),
                    src2: src2.clone(),
                }]);
            }
        },
        (Instruction::Cmp { src1, src2 }, Instruction::Jcc { cond, target: _ }) => {
            if let Operand::Imm(0) = src2 {
                if *cond == Condition::Eq {
                    return Some(vec![Instruction::Test {
                        src1: src1.clone(),
                        src2: src1.clone(),
                    }]);
                }
                if *cond == Condition::Ne {
                    return Some(vec![Instruction::Test {
                        src1: src1.clone(),
                        src2: src1.clone(),
                    }]);
                }
            }
        },
        (Instruction::Add { dest: d1, src: s1 }, Instruction::Mov { dest: d2, src: s2 }) => {
            if d1 == d2 {
                if let Operand::Imm(0) = s1 {
                    return Some(vec![Instruction::Mov {
                        dest: d2.clone(),
                        src: s2.clone(),
                    }]);
                }
            }
        },
        (Instruction::Mov { dest: d1, src: s1 }, Instruction::Mov { dest: d2, src: s2 }) => {
            if let Operand::Reg(vr1) = s1 {
                if let Operand::Reg(vr2) = s2 {
                    if vr1 == vr2 && d1 == s2 {
                        return Some(vec![Instruction::Mov {
                            dest: d2.clone(),
                            src: s1.clone(),
                        }]);
                    }
                }
            }
        },
        _ => {},
    }

    None
}

fn dead_code(func: &mut Function) -> bool {
    let mut used = vec![false; func.registers.len()];

    for inst in &func.instructions {
        collect_uses(inst, &mut used);
    }

    let mut changed = false;
    let mut new_insts: Vec<Instruction> = Vec::new();

    for inst in &func.instructions {
        match inst {
            Instruction::Mov { dest, .. } => {
                if let Operand::Reg(vr) = dest {
                    let idx = vr.id as usize;
                    if idx < used.len() && !used[idx] {
                        changed = true;
                        continue;
                    }
                }
            },
            _ => {},
        }
        new_insts.push(inst.clone());
    }

    if changed {
        func.instructions = new_insts;
    }

    changed
}

fn collect_uses(inst: &Instruction, used: &mut Vec<bool>) {
    match inst {
        Instruction::Mov { dest, src } => {
            if let Operand::Reg(vr) = dest {
                let idx = vr.id as usize;
                if idx < used.len() {
                    mark_use(src, used);
                }
            }
        },
        Instruction::Add { dest, src } => mark_two(dest, src, used),
        Instruction::Sub { dest, src } => mark_two(dest, src, used),
        Instruction::Mul { dest, src, .. } => mark_two(dest, src, used),
        Instruction::And { dest, src } => mark_two(dest, src, used),
        Instruction::Or { dest, src } => mark_two(dest, src, used),
        Instruction::Xor { dest, src } => mark_two(dest, src, used),
        Instruction::Shl { dest, count } => mark_two(dest, count, used),
        Instruction::Shr { dest, count } => mark_two(dest, count, used),
        Instruction::Cmp { src1, src2 } => {
            mark_use(src1, used);
            mark_use(src2, used);
        },
        Instruction::Call { .. } => {},
        _ => {},
    }
}

fn mark_two(dest: &Operand, src: &Operand, used: &mut Vec<bool>) {
    if let Operand::Reg(vr) = dest {
        let idx = vr.id as usize;
        if idx < used.len() {
            mark_use(src, used);
        }
    }
}

fn mark_use(op: &Operand, used: &mut Vec<bool>) {
    match op {
        Operand::Reg(vr) => {
            let idx = vr.id as usize;
            if idx < used.len() {
                used[idx] = true;
            }
        },
        _ => {},
    }
}
