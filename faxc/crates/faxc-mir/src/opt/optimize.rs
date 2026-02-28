//! MIR Optimization Passes

use crate::mir::*;
use faxc_sem::types::Type;
use std::collections::HashMap;

pub fn optimize_function(func: &mut Function) {
    let mut changed = true;
    let mut iterations = 0;
    let max_iterations = 10;

    while changed && iterations < max_iterations {
        changed = false;

        simplify(func);
        fold(func);
        propagate(func);
        reduce(func);
        cse(func);
        licm(func);
        simplify_br(func);
        eliminate_phi(func);
        changed |= jump_threading(func);
        changed |= merge_blocks(func);
        changed |= eliminate_unreachable(func);
        changed |= optimize_cond(func);
        changed |= dead_code(func);

        iterations += 1;
    }
}

fn simplify(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        let mut i = 0;
        while i < block.statements.len() {
            if let Statement::Assign(place, rvalue) = &block.statements[i] {
                let simple = match rvalue {
                    Rvalue::BinaryOp(op, left, right) => simp_bin(*op, left, right),
                    Rvalue::UnaryOp(op, operand) => simp_un(*op, operand),
                    _ => None,
                };
                if let Some(r) = simple {
                    block.statements[i] = Statement::Assign(place.clone(), r);
                }
            }
            i += 1;
        }
    }
}

fn simp_bin(op: BinOp, left: &Box<Operand>, right: &Box<Operand>) -> Option<Rvalue> {
    match op {
        BinOp::Add => {
            if is_zero(right) {
                return Some(Rvalue::Use(*left.clone()));
            }
            if is_zero(left) {
                return Some(Rvalue::Use(*right.clone()));
            }
            if eq_place(left, right) {
                return Some(Rvalue::BinaryOp(
                    BinOp::Mul,
                    left.clone(),
                    Box::new(Operand::Constant(Constant {
                        ty: Type::Int,
                        kind: ConstantKind::Int(2),
                    })),
                ));
            }
        },
        BinOp::Sub => {
            if is_zero(right) {
                return Some(Rvalue::Use(*left.clone()));
            }
            if eq_place(left, right) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Int,
                    kind: ConstantKind::Int(0),
                })));
            }
        },
        BinOp::Mul => {
            if is_one(right) {
                return Some(Rvalue::Use(*left.clone()));
            }
            if is_one(left) {
                return Some(Rvalue::Use(*right.clone()));
            }
            if is_zero(right) || is_zero(left) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Int,
                    kind: ConstantKind::Int(0),
                })));
            }
        },
        BinOp::Div => {
            if is_one(right) {
                return Some(Rvalue::Use(*left.clone()));
            }
            if is_zero(left) && !is_zero(right) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Int,
                    kind: ConstantKind::Int(0),
                })));
            }
        },
        BinOp::Rem => {
            if is_zero(right) {
                return None;
            }
            if is_zero(left) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Int,
                    kind: ConstantKind::Int(0),
                })));
            }
        },
        BinOp::BitAnd => {
            if eq_place(left, right) {
                return Some(Rvalue::Use(*left.clone()));
            }
            if is_zero(right) || is_zero(left) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Int,
                    kind: ConstantKind::Int(0),
                })));
            }
            if is_all_ones(right) {
                return Some(Rvalue::Use(*left.clone()));
            }
            if is_all_ones(left) {
                return Some(Rvalue::Use(*right.clone()));
            }
        },
        BinOp::BitOr => {
            if eq_place(left, right) {
                return Some(Rvalue::Use(*left.clone()));
            }
            if is_zero(right) {
                return Some(Rvalue::Use(*left.clone()));
            }
            if is_zero(left) {
                return Some(Rvalue::Use(*right.clone()));
            }
        },
        BinOp::BitXor => {
            if is_zero(right) {
                return Some(Rvalue::Use(*left.clone()));
            }
            if is_zero(left) {
                return Some(Rvalue::Use(*right.clone()));
            }
            if eq_place(left, right) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Int,
                    kind: ConstantKind::Int(0),
                })));
            }
        },
        BinOp::Shl | BinOp::Shr => {
            if is_zero(right) {
                return Some(Rvalue::Use(*left.clone()));
            }
            if is_zero(left) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Int,
                    kind: ConstantKind::Int(0),
                })));
            }
        },
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            if eq_place(left, right) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Bool,
                    kind: ConstantKind::Bool(simp_cmp_eq(op)),
                })));
            }
        },
        _ => {},
    }
    None
}

fn simp_cmp_eq(op: BinOp) -> bool {
    match op {
        BinOp::Eq | BinOp::Le | BinOp::Ge => true,
        _ => false,
    }
}

fn simp_un(op: UnOp, operand: &Box<Operand>) -> Option<Rvalue> {
    if let UnOp::Not = op {
        if let Operand::Constant(c) = operand.as_ref() {
            if let ConstantKind::Int(n) = c.kind {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: c.ty.clone(),
                    kind: ConstantKind::Int(!n),
                })));
            }
        }
    }
    None
}

fn is_zero(op: &Box<Operand>) -> bool {
    matches!(op.as_ref(), Operand::Constant(c) if matches!(c.kind, ConstantKind::Int(0) | ConstantKind::Float(0.0)))
}

fn is_one(op: &Box<Operand>) -> bool {
    matches!(op.as_ref(), Operand::Constant(c) if matches!(c.kind, ConstantKind::Int(1) | ConstantKind::Float(1.0)))
}

fn is_all_ones(op: &Box<Operand>) -> bool {
    matches!(op.as_ref(), Operand::Constant(c) if matches!(c.kind, ConstantKind::Int(i) if i == -1))
}

fn eq_place(a: &Box<Operand>, b: &Box<Operand>) -> bool {
    match (a.as_ref(), b.as_ref()) {
        (Operand::Copy(p1), Operand::Copy(p2)) | (Operand::Move(p1), Operand::Move(p2)) => p1 == p2,
        _ => false,
    }
}

fn fold(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        let mut i = 0;
        while i < block.statements.len() {
            if let Statement::Assign(place, rvalue) = &block.statements[i] {
                let folded = match rvalue {
                    Rvalue::BinaryOp(op, left, right) => fold_bin(*op, left, right),
                    Rvalue::UnaryOp(op, operand) => fold_un(*op, operand),
                    _ => None,
                };
                if let Some(r) = folded {
                    block.statements[i] = Statement::Assign(place.clone(), r);
                }
            }
            i += 1;
        }
    }
}

fn fold_bin(op: BinOp, left: &Box<Operand>, right: &Box<Operand>) -> Option<Rvalue> {
    match (op, left.as_ref(), right.as_ref()) {
        (BinOp::Add, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                return const_int(li.wrapping_add(*ri));
            }
            if let (ConstantKind::Float(lf), ConstantKind::Float(rf)) = (&l.kind, &r.kind) {
                return const_float(lf + rf);
            }
        },
        (BinOp::Sub, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                return const_int(li.wrapping_sub(*ri));
            }
            if let (ConstantKind::Float(lf), ConstantKind::Float(rf)) = (&l.kind, &r.kind) {
                return const_float(lf - rf);
            }
        },
        (BinOp::Mul, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                return const_int(li.wrapping_mul(*ri));
            }
            if let (ConstantKind::Float(lf), ConstantKind::Float(rf)) = (&l.kind, &r.kind) {
                return const_float(lf * rf);
            }
        },
        (BinOp::Div, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                if *ri != 0 {
                    return const_int(li.wrapping_div(*ri));
                }
            }
            if let (ConstantKind::Float(lf), ConstantKind::Float(rf)) = (&l.kind, &r.kind) {
                if *rf != 0.0 {
                    return const_float(lf / rf);
                }
            }
        },
        (BinOp::Rem, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                if *ri != 0 {
                    return const_int(li.wrapping_rem(*ri));
                }
            }
        },
        (BinOp::BitAnd, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                return const_int(li & ri);
            }
        },
        (BinOp::BitOr, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                return const_int(li | ri);
            }
        },
        (BinOp::BitXor, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                return const_int(li ^ ri);
            }
        },
        (BinOp::Shl, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                if *ri >= 0 && *ri < 64 {
                    return const_int(li.wrapping_shl(*ri as u32));
                }
            }
        },
        (BinOp::Shr, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                if *ri >= 0 && *ri < 64 {
                    return const_int(li.wrapping_shr(*ri as u32));
                }
            }
        },
        (BinOp::Eq, Operand::Constant(l), Operand::Constant(r)) => {
            return Some(Rvalue::Use(Operand::Constant(Constant {
                ty: Type::Bool,
                kind: ConstantKind::Bool(l.kind == r.kind),
            })));
        },
        (BinOp::Ne, Operand::Constant(l), Operand::Constant(r)) => {
            return Some(Rvalue::Use(Operand::Constant(Constant {
                ty: Type::Bool,
                kind: ConstantKind::Bool(l.kind != r.kind),
            })));
        },
        (BinOp::Lt, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Bool,
                    kind: ConstantKind::Bool(li < ri),
                })));
            }
        },
        (BinOp::Le, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Bool,
                    kind: ConstantKind::Bool(li <= ri),
                })));
            }
        },
        (BinOp::Gt, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Bool,
                    kind: ConstantKind::Bool(li > ri),
                })));
            }
        },
        (BinOp::Ge, Operand::Constant(l), Operand::Constant(r)) => {
            if let (ConstantKind::Int(li), ConstantKind::Int(ri)) = (&l.kind, &r.kind) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Bool,
                    kind: ConstantKind::Bool(li >= ri),
                })));
            }
            if let (ConstantKind::Float(lf), ConstantKind::Float(rf)) = (&l.kind, &r.kind) {
                return Some(Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::Bool,
                    kind: ConstantKind::Bool(lf >= rf),
                })));
            }
        },
        _ => {},
    }
    None
}

fn fold_un(op: UnOp, operand: &Box<Operand>) -> Option<Rvalue> {
    if let Operand::Constant(c) = operand.as_ref() {
        match op {
            UnOp::Neg => {
                if let ConstantKind::Int(n) = c.kind {
                    return const_int(n.wrapping_neg());
                }
                if let ConstantKind::Float(f) = c.kind {
                    return const_float(-f);
                }
            },
            UnOp::Not => {
                if let ConstantKind::Int(n) = c.kind {
                    return const_int(!n);
                }
                if let ConstantKind::Bool(b) = c.kind {
                    return Some(Rvalue::Use(Operand::Constant(Constant {
                        ty: Type::Bool,
                        kind: ConstantKind::Bool(!b),
                    })));
                }
            },
        }
    }
    None
}

fn const_int(n: i64) -> Option<Rvalue> {
    Some(Rvalue::Use(Operand::Constant(Constant {
        ty: Type::Int,
        kind: ConstantKind::Int(n),
    })))
}

fn const_float(f: f64) -> Option<Rvalue> {
    Some(Rvalue::Use(Operand::Constant(Constant {
        ty: Type::Float,
        kind: ConstantKind::Float(f),
    })))
}

fn propagate(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        let mut copies: HashMap<LocalId, Operand> = HashMap::new();

        for stmt in block.statements.iter_mut() {
            if let Statement::Assign(place, rvalue) = stmt {
                let new_rvalue = propagate_rvalue(rvalue, &copies);
                *rvalue = new_rvalue;

                if let (Place::Local(dest), Rvalue::Use(src)) = (&*place, &*rvalue) {
                    if let Operand::Copy(s) | Operand::Move(s) = src {
                        if let Place::Local(_sid) = s {
                            copies.insert(*dest, src.clone());
                        }
                    }
                }
                if let Place::Local(d) = &*place {
                    copies.remove(d);
                }
            }
        }

        if let Terminator::If { cond, .. } = &mut block.terminator {
            *cond = propagate_operand(cond, &copies);
        }
    }
}

fn propagate_rvalue(rvalue: &Rvalue, copies: &HashMap<LocalId, Operand>) -> Rvalue {
    match rvalue {
        Rvalue::Use(op) => Rvalue::Use(propagate_operand(op, copies)),
        Rvalue::UnaryOp(uop, op) => Rvalue::UnaryOp(*uop, Box::new(propagate_operand(op, copies))),
        Rvalue::BinaryOp(bop, l, r) => Rvalue::BinaryOp(
            *bop,
            Box::new(propagate_operand(l, copies)),
            Box::new(propagate_operand(r, copies)),
        ),
        Rvalue::Cast(kind, op, ty) => {
            Rvalue::Cast(*kind, propagate_operand(op, copies), ty.clone())
        },
        _ => rvalue.clone(),
    }
}

fn propagate_operand(op: &Operand, copies: &HashMap<LocalId, Operand>) -> Operand {
    if let Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) = op {
        if let Some(cached) = copies.get(id) {
            return cached.clone();
        }
    }
    op.clone()
}

fn reduce(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        let mut i = 0;
        while i < block.statements.len() {
            if let Statement::Assign(place, Rvalue::BinaryOp(BinOp::Mul, left, right)) =
                &block.statements[i]
            {
                if let Operand::Constant(c) = right.as_ref() {
                    if let ConstantKind::Int(n) = c.kind {
                        if n > 0 && (n as u64).is_power_of_two() {
                            let shift = n.trailing_zeros();
                            block.statements[i] = Statement::Assign(
                                place.clone(),
                                Rvalue::BinaryOp(
                                    BinOp::Shl,
                                    left.clone(),
                                    Box::new(Operand::Constant(Constant {
                                        ty: Type::Int,
                                        kind: ConstantKind::Int(shift as i64),
                                    })),
                                ),
                            );
                        }
                    }
                }
            }
            if let Statement::Assign(place, Rvalue::BinaryOp(BinOp::Div, left, right)) =
                &block.statements[i]
            {
                if let Operand::Constant(c) = right.as_ref() {
                    if let ConstantKind::Int(n) = c.kind {
                        if n > 0 && (n as u64).is_power_of_two() {
                            let shift = n.trailing_zeros();
                            block.statements[i] = Statement::Assign(
                                place.clone(),
                                Rvalue::BinaryOp(
                                    BinOp::Shr,
                                    left.clone(),
                                    Box::new(Operand::Constant(Constant {
                                        ty: Type::Int,
                                        kind: ConstantKind::Int(shift as i64),
                                    })),
                                ),
                            );
                        }
                    }
                }
            }
            i += 1;
        }
    }
}

fn cse(func: &mut Function) {
    let mut seen: HashMap<(BinOp, LocalId, LocalId), Place> = HashMap::new();

    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        seen.clear();

        let mut i = 0;
        while i < block.statements.len() {
            if let Statement::Assign(place, Rvalue::BinaryOp(op, left, right)) =
                &block.statements[i]
            {
                let l_id = get_local_id(left);
                let r_id = get_local_id(right);
                if let (Some(l), Some(r)) = (l_id, r_id) {
                    let key = (*op, l, r);
                    if let Some(prev_place) = seen.get(&key) {
                        block.statements[i] = Statement::Assign(
                            place.clone(),
                            Rvalue::Use(Operand::Copy(prev_place.clone())),
                        );
                    } else {
                        if let Place::Local(dest) = place {
                            seen.insert(key, Place::Local(*dest));
                        }
                    }
                }
            }
            i += 1;
        }
    }
}

fn get_local_id(op: &Box<Operand>) -> Option<LocalId> {
    match &**op {
        Operand::Copy(p) | Operand::Move(p) => {
            if let Place::Local(id) = p {
                Some(*id)
            } else {
                None
            }
        },
        _ => None,
    }
}
fn licm(func: &mut Function) {
    let mut loop_headers: Vec<BlockId> = Vec::new();

    for block_idx in 0..func.blocks.len() {
        let block_id = BlockId(block_idx as u32);
        let block = &func.blocks[block_id];
        if let Terminator::If {
            then_block,
            else_block,
            ..
        } = &block.terminator
        {
            if *then_block == block_id || *else_block == block_id {
                loop_headers.push(block_id);
            }
        }
    }

    for header in loop_headers {
        move_invariants(func, header);
    }
}

fn move_invariants(func: &mut Function, header: BlockId) {
    let header_block = &func.blocks[header];
    let mut invariants: Vec<(usize, Rvalue)> = Vec::new();

    for (idx, stmt) in header_block.statements.iter().enumerate() {
        if let Statement::Assign(_, rvalue) = stmt {
            if is_loop_invariant(rvalue, header) {
                invariants.push((idx, rvalue.clone()));
            }
        }
    }

    if !invariants.is_empty() {
        if let Some(first_block) = find_predecessor(func, header) {
            let local_count = func.local_count();
            let first = &mut func.blocks[first_block];
            for (_, inv) in invariants.iter().rev() {
                let new_place = Place::Local(LocalId(local_count as u32));
                first
                    .statements
                    .push(Statement::Assign(new_place.clone(), inv.clone()));
            }
        }
    }
}

fn is_loop_invariant(rvalue: &Rvalue, _header: BlockId) -> bool {
    match rvalue {
        Rvalue::Use(op) => !uses_local(op),
        Rvalue::UnaryOp(_, op) => !uses_local(op),
        Rvalue::BinaryOp(_, l, r) => !uses_local(l) && !uses_local(r),
        Rvalue::NullaryOp(..) => true,
        Rvalue::Cast(_, op, _) => !uses_local(op),
        _ => false,
    }
}

fn uses_local(op: &Operand) -> bool {
    match op {
        Operand::Copy(p) | Operand::Move(p) => matches!(p, Place::Local(..)),
        Operand::Constant(_) => false,
    }
}

fn find_predecessor(func: &Function, target: BlockId) -> Option<BlockId> {
    for block_idx in 0..func.blocks.len() {
        let bid = BlockId(block_idx as u32);
        if bid == target {
            continue;
        }
        let block = &func.blocks[bid];
        match &block.terminator {
            Terminator::Goto { target: tgt } => {
                if *tgt == target {
                    return Some(bid);
                }
            },
            Terminator::If {
                then_block,
                else_block,
                ..
            } => {
                if *then_block == target || *else_block == target {
                    return Some(bid);
                }
            },
            _ => {},
        }
    }

    None
}

fn optimize_cond(func: &mut Function) -> bool {
    let mut changed = false;
    let mut replacements: Vec<(BlockId, Terminator)> = Vec::new();

    for block_idx in 0..func.blocks.len() {
        let block_id = BlockId(block_idx as u32);
        let block = &func.blocks[block_id];

        if let Terminator::If {
            cond,
            then_block,
            else_block,
        } = &block.terminator
        {
            if let Operand::Constant(c) = cond {
                if let ConstantKind::Bool(b) = c.kind {
                    let target = if b { *then_block } else { *else_block };
                    replacements.push((block_id, Terminator::Goto { target }));
                    changed = true;
                }
            }
        }

        if let Terminator::SwitchInt {
            discr,
            switch_ty: _,
            targets,
            otherwise,
        } = &block.terminator
        {
            if let Operand::Constant(c) = discr {
                if let ConstantKind::Int(v) = c.kind {
                    let mut target = *otherwise;
                    for (val, t) in targets {
                        if *val == v as u128 {
                            target = *t;
                            break;
                        }
                    }
                    replacements.push((block_id, Terminator::Goto { target }));
                    changed = true;
                }
            }
        }
    }

    for (block_id, new_term) in replacements {
        func.blocks[block_id].terminator = new_term;
    }

    changed
}

fn simplify_br(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        if let Terminator::If {
            cond,
            then_block,
            else_block,
        } = &block.terminator
        {
            if let Operand::Constant(c) = cond {
                if let ConstantKind::Int(v) = c.kind {
                    let target = if v != 0 { then_block } else { else_block };
                    block.terminator = Terminator::Goto { target: *target };
                }
            }
        }
    }
}

fn eliminate_phi(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        block.statements.retain(|s| {
            if let Statement::Assign(p, Rvalue::Use(Operand::Copy(s))) = s {
                if let (Place::Local(d), Place::Local(sid)) = (p, s) {
                    return *d != *sid;
                }
            }
            true
        });
    }
}

pub fn dead_code(func: &mut Function) -> bool {
    let mut used = vec![true; func.local_count()];
    used[0] = true;

    for block_idx in 0..func.blocks.len() {
        let block = &func.blocks[BlockId(block_idx as u32)];
        for stmt in &block.statements {
            if let Statement::Assign(_, rvalue) = stmt {
                mark_use(rvalue, &mut used);
            }
            mark_term_use(&block.terminator, &mut used);
        }
    }

    let mut changed = false;
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        for stmt in block.statements.iter_mut() {
            if let Statement::Assign(place, _) = stmt {
                if let Place::Local(id) = place {
                    if used[id.0 as usize] == false {
                        *stmt = Statement::Nop;
                        changed = true;
                    }
                }
            }
        }
    }
    changed
}

fn mark_use(rvalue: &Rvalue, used: &mut Vec<bool>) {
    match rvalue {
        Rvalue::Use(op) => mark_op_use(op, used),
        Rvalue::UnaryOp(_, op) => mark_op_use(op, used),
        Rvalue::BinaryOp(_, l, r) => {
            mark_op_use(l, used);
            mark_op_use(r, used);
        },
        Rvalue::Aggregate(_, ops) => {
            for op in ops {
                mark_op_use(op, used);
            }
        },
        _ => {},
    }
}

fn mark_op_use(op: &Operand, used: &mut Vec<bool>) {
    if let Operand::Copy(id) | Operand::Move(id) = op {
        if let Place::Local(i) = id {
            if (i.0 as usize) < used.len() {
                used[i.0 as usize] = true;
            }
        }
    }
}

fn mark_term_use(term: &Terminator, used: &mut Vec<bool>) {
    match term {
        Terminator::If { cond, .. } => mark_op_use(cond, used),
        Terminator::SwitchInt { discr, .. } => mark_op_use(discr, used),
        Terminator::Call { args, .. } => {
            for arg in args {
                mark_op_use(arg, used);
            }
        },
        _ => {},
    }
}

fn jump_threading(func: &mut Function) -> bool {
    let mut changed = false;

    let mut replacements: Vec<(BlockId, Terminator)> = Vec::new();

    for block_idx in 0..func.blocks.len() {
        let block_id = BlockId(block_idx as u32);
        let block = &func.blocks[block_id];

        if let Terminator::Goto { target } = &block.terminator {
            let target_block = &func.blocks[*target];

            if let Terminator::Goto {
                target: next_target,
            } = &target_block.terminator
            {
                replacements.push((
                    block_id,
                    Terminator::Goto {
                        target: *next_target,
                    },
                ));
                changed = true;
            }

            if let Terminator::If {
                cond,
                then_block,
                else_block,
            } = &target_block.terminator
            {
                if *then_block == *target || *else_block == *target {
                    replacements.push((
                        block_id,
                        Terminator::If {
                            cond: cond.clone(),
                            then_block: *then_block,
                            else_block: *else_block,
                        },
                    ));
                    changed = true;
                }
            }
        }
    }

    for (block_id, new_term) in replacements {
        func.blocks[block_id].terminator = new_term;
    }

    let mut i = 0;
    while i < func.blocks.len() {
        let block_id = BlockId(i as u32);
        let block = &func.blocks[block_id].clone();

        if let Terminator::If {
            cond: _,
            then_block,
            else_block,
        } = &block.terminator
        {
            let then_target = func.blocks[*then_block].terminator.clone();
            let else_target = func.blocks[*else_block].terminator.clone();

            if let (Terminator::Goto { target: t_tgt }, Terminator::Goto { target: e_tgt }) =
                (then_target, else_target)
            {
                if t_tgt == e_tgt {
                    func.blocks[block_id].terminator = Terminator::Goto { target: t_tgt };
                    changed = true;
                }
            }
        }

        i += 1;
    }

    changed
}

fn merge_blocks(func: &mut Function) -> bool {
    let mut changed = false;

    let mut merge_candidates: Vec<(BlockId, BlockId, Terminator)> = Vec::new();

    for block_idx in 0..func.blocks.len() {
        let block_id = BlockId(block_idx as u32);
        let block = &func.blocks[block_id];

        if let Terminator::Goto { target } = &block.terminator {
            if *target != block_id {
                let target_block = &func.blocks[*target];
                if target_block.statements.is_empty() {
                    if let Terminator::Goto {
                        target: next_target,
                    } = &target_block.terminator
                    {
                        merge_candidates.push((
                            block_id,
                            *target,
                            Terminator::Goto {
                                target: *next_target,
                            },
                        ));
                    }
                }
            }
        }
    }

    for (from, _, new_term) in merge_candidates {
        func.blocks[from].terminator = new_term;
        changed = true;
    }

    changed
}

fn eliminate_unreachable(func: &mut Function) -> bool {
    let mut reachable = vec![false; func.blocks.len()];
    let mut queue = Vec::new();

    reachable[func.entry_block.0 as usize] = true;
    queue.push(func.entry_block);

    while let Some(block_id) = queue.pop() {
        let block = &func.blocks[block_id];
        let targets = get_terminator_targets(&block.terminator);

        for target in targets {
            let idx = target.0 as usize;
            if idx < reachable.len() && !reachable[idx] {
                reachable[idx] = true;
                queue.push(target);
            }
        }
    }

    let mut changed = false;
    let mut new_blocks: Vec<(BlockId, BasicBlock)> = Vec::new();
    let mut old_to_new: HashMap<BlockId, BlockId> = HashMap::new();

    for (idx, is_reachable) in reachable.iter().enumerate() {
        if *is_reachable {
            let old_id = BlockId(idx as u32);
            let new_id = BlockId(new_blocks.len() as u32);
            old_to_new.insert(old_id, new_id);
            new_blocks.push((new_id, func.blocks[old_id].clone()));
        } else {
            changed = true;
        }
    }

    if changed {
        func.blocks.clear();
        for (_, mut block) in new_blocks {
            block.id = BlockId(func.blocks.len() as u32);
            update_terminator_targets(&mut block.terminator, &old_to_new);
            func.blocks.push(block);
        }

        func.entry_block = BlockId(0);
    }

    changed
}

fn get_terminator_targets(term: &Terminator) -> Vec<BlockId> {
    match term {
        Terminator::Goto { target } => vec![*target],
        Terminator::If {
            then_block,
            else_block,
            ..
        } => vec![*then_block, *else_block],
        Terminator::SwitchInt {
            targets, otherwise, ..
        } => {
            let mut result = vec![*otherwise];
            for (_, t) in targets {
                result.push(*t);
            }
            result
        },
        Terminator::Call {
            target, cleanup, ..
        } => {
            let mut result = Vec::new();
            if let Some(t) = target {
                result.push(*t);
            }
            if let Some(c) = cleanup {
                result.push(*c);
            }
            result
        },
        Terminator::Resume => vec![],
        Terminator::Abort => vec![],
        Terminator::Return => vec![],
        Terminator::Unreachable => vec![],
    }
}

fn update_terminator_targets(term: &mut Terminator, mapping: &HashMap<BlockId, BlockId>) {
    match term {
        Terminator::Goto { target } => {
            if let Some(new_target) = mapping.get(target) {
                *target = *new_target;
            }
        },
        Terminator::If {
            then_block,
            else_block,
            ..
        } => {
            if let Some(new_then) = mapping.get(then_block) {
                *then_block = *new_then;
            }
            if let Some(new_else) = mapping.get(else_block) {
                *else_block = *new_else;
            }
        },
        Terminator::SwitchInt {
            targets, otherwise, ..
        } => {
            for (_, t) in targets.iter_mut() {
                if let Some(new_t) = mapping.get(t) {
                    *t = *new_t;
                }
            }
            if let Some(new_o) = mapping.get(otherwise) {
                *otherwise = *new_o;
            }
        },
        Terminator::Call {
            target, cleanup, ..
        } => {
            if let Some(t) = target {
                if let Some(new_t) = mapping.get(t) {
                    *target = Some(*new_t);
                }
            }
            if let Some(c) = cleanup {
                if let Some(new_c) = mapping.get(c) {
                    *cleanup = Some(*new_c);
                }
            }
        },
        _ => {},
    }
}
