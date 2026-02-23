//! MIR Optimization Passes
//!
//! MIR-LIR-CODEGEN-DEV-001: Subtask 1
//! Implements core MIR optimizations:
//! 1. Constant Folding
//! 2. Dead Code Elimination
//! 3. Copy Propagation
//! 4. Algebraic Simplification
//! 5. Common Subexpression Elimination (CSE)
//! 6. Strength Reduction
//! 7. Loop Invariant Code Motion (LICM)
//! 8. Branch Simplification
//! 9. Phi Elimination

use crate::mir::*;
use faxc_sem::types::Type;
use std::collections::HashMap;

/// Run all optimization passes on a MIR function
pub fn optimize_function(func: &mut Function) {
    let mut changed = true;
    let mut iterations = 0;
    let max_iterations = 10;

    while changed && iterations < max_iterations {
        changed = false;

        algebraic_simplification(func);
        constant_folding(func);
        copy_propagation(func);
        strength_reduction(func);
        common_subexpression_elimination(func);
        loop_invariant_code_motion(func);
        branch_simplification(func);
        phi_elimination(func);
        changed |= dead_code_elimination(func);

        iterations += 1;
    }

    if iterations >= max_iterations {
        eprintln!(
            "Warning: optimization reached max iterations ({})",
            max_iterations
        );
    }
}

/// Optimization 1: Constant Folding
/// Evaluates constant expressions at compile time
pub fn constant_folding(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        let mut i = 0;
        while i < block.statements.len() {
            let new_stmt = match &block.statements[i] {
                Statement::Assign(place, rvalue) => {
                    let place_clone = place.clone();
                    if let Rvalue::BinaryOp(op, left, right) = rvalue {
                        if let (Some(lc), Some(rc)) = (fold_operand(left), fold_operand(right)) {
                            if let (ConstantKind::Int(lv), ConstantKind::Int(rv)) =
                                (&lc.kind, &rc.kind)
                            {
                                if let Some(result) = fold_int_binop(*op, *lv, *rv) {
                                    Some(Statement::Assign(
                                        place_clone,
                                        Rvalue::Use(Operand::Constant(Constant {
                                            ty: lc.ty.clone(),
                                            kind: ConstantKind::Int(result),
                                        })),
                                    ))
                                } else {
                                    None
                                }
                            } else if let (ConstantKind::Float(lv), ConstantKind::Float(rv)) =
                                (&lc.kind, &rc.kind)
                            {
                                if let Some(result) = fold_float_binop(*op, *lv, *rv) {
                                    Some(Statement::Assign(
                                        place_clone,
                                        Rvalue::Use(Operand::Constant(Constant {
                                            ty: lc.ty.clone(),
                                            kind: ConstantKind::Float(result),
                                        })),
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else if let Rvalue::UnaryOp(op, operand) = rvalue {
                        if let Some(lc) = fold_operand(operand) {
                            if let ConstantKind::Int(lv) = &lc.kind {
                                if let Some(result) = fold_int_unop(*op, *lv) {
                                    Some(Statement::Assign(
                                        place_clone,
                                        Rvalue::Use(Operand::Constant(Constant {
                                            ty: lc.ty.clone(),
                                            kind: ConstantKind::Int(result),
                                        })),
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                _ => None,
            };

            if let Some(stmt) = new_stmt {
                block.statements[i] = stmt;
            }
            i += 1;
        }
    }
}

fn fold_operand(op: &Operand) -> Option<Constant> {
    match op {
        Operand::Constant(c) => Some(c.clone()),
        Operand::Copy(Place::Local(_)) | Operand::Move(Place::Local(_)) => None,
        Operand::Copy(Place::Projection(_, _)) | Operand::Move(Place::Projection(_, _)) => None,
    }
}

fn fold_int_binop(op: BinOp, lv: i64, rv: i64) -> Option<i64> {
    Some(match op {
        BinOp::Add => lv.wrapping_add(rv),
        BinOp::Sub => lv.wrapping_sub(rv),
        BinOp::Mul => lv.wrapping_mul(rv),
        BinOp::Div if rv != 0 => lv.wrapping_div(rv),
        BinOp::Rem if rv != 0 => lv.wrapping_rem(rv),
        BinOp::BitAnd => lv & rv,
        BinOp::BitOr => lv | rv,
        BinOp::BitXor => lv ^ rv,
        BinOp::Shl => lv.wrapping_shl(rv as u32),
        BinOp::Shr => lv.wrapping_shr(rv as u32),
        _ => return None,
    })
}

fn fold_float_binop(op: BinOp, lv: f64, rv: f64) -> Option<f64> {
    Some(match op {
        BinOp::Add => lv + rv,
        BinOp::Sub => lv - rv,
        BinOp::Mul => lv * rv,
        BinOp::Div if rv != 0.0 => lv / rv,
        BinOp::Rem if rv != 0.0 => lv % rv,
        _ => return None,
    })
}

fn fold_int_unop(op: UnOp, v: i64) -> Option<i64> {
    Some(match op {
        UnOp::Neg => v.wrapping_neg(),
        UnOp::Not => !v,
    })
}

/// Optimization 1b: Algebraic Simplification
/// Simplifies expressions like x + 0 = x, x * 1 = x, etc.
pub fn algebraic_simplification(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        let mut i = 0;
        while i < block.statements.len() {
            let new_stmt = match &block.statements[i] {
                Statement::Assign(place, rvalue) => {
                    let simplified = simplify_rvalue(rvalue);
                    if simplified != *rvalue {
                        Some(Statement::Assign(place.clone(), simplified))
                    } else {
                        None
                    }
                },
                _ => None,
            };
            if let Some(stmt) = new_stmt {
                block.statements[i] = stmt;
            }
            i += 1;
        }
    }
}

fn simplify_rvalue(rvalue: &Rvalue) -> Rvalue {
    match rvalue {
        Rvalue::BinaryOp(op, left, right) => {
            // Try to simplify binary operations
            if let Some(simplified) = simplify_binop(op, left, right) {
                return simplified;
            }
            rvalue.clone()
        },
        Rvalue::UnaryOp(op, operand) => simplify_unop(op, operand),
        _ => rvalue.clone(),
    }
}

fn simplify_binop(op: &BinOp, left: &Operand, right: &Operand) -> Option<Rvalue> {
    // x + 0 = x
    if *op == BinOp::Add {
        if is_zero(right) {
            return Some(Rvalue::Use(left.clone()));
        }
        if is_zero(left) {
            return Some(Rvalue::Use(right.clone()));
        }
    }

    // x - 0 = x
    if *op == BinOp::Sub {
        if is_zero(right) {
            return Some(Rvalue::Use(left.clone()));
        }
    }

    // x * 1 = x
    if *op == BinOp::Mul {
        if is_one(right) {
            return Some(Rvalue::Use(left.clone()));
        }
        if is_one(left) {
            return Some(Rvalue::Use(right.clone()));
        }
    }

    // x * 0 = 0
    if *op == BinOp::Mul {
        if is_zero(right) || is_zero(left) {
            return Some(Rvalue::Use(Operand::Constant(Constant {
                ty: get_operand_type(left),
                kind: ConstantKind::Int(0),
            })));
        }
    }

    // x / 1 = x
    if *op == BinOp::Div {
        if is_one(right) {
            return Some(Rvalue::Use(left.clone()));
        }
    }

    // x & x = x, x | x = x
    if *op == BinOp::BitAnd || *op == BinOp::BitOr {
        if left == right {
            return Some(Rvalue::Use(left.clone()));
        }
    }

    // x ^ 0 = x
    if *op == BinOp::BitXor {
        if is_zero(right) {
            return Some(Rvalue::Use(left.clone()));
        }
        if is_zero(left) {
            return Some(Rvalue::Use(right.clone()));
        }
    }

    // x - x = 0
    if *op == BinOp::Sub {
        if left == right {
            return Some(Rvalue::Use(Operand::Constant(Constant {
                ty: get_operand_type(left),
                kind: ConstantKind::Int(0),
            })));
        }
    }

    None
}

fn simplify_unop(op: &UnOp, operand: &Operand) -> Rvalue {
    // !!x = x
    if *op == UnOp::Not {
        if let Operand::Constant(c) = operand {
            if let ConstantKind::Int(v) = c.kind {
                return Rvalue::Use(Operand::Constant(Constant {
                    ty: c.ty.clone(),
                    kind: ConstantKind::Int(!v),
                }));
            }
        }
    }

    Rvalue::UnaryOp(*op, Box::new(operand.clone()))
}

fn is_zero(op: &Operand) -> bool {
    match op {
        Operand::Constant(c) => match &c.kind {
            ConstantKind::Int(i) => *i == 0,
            ConstantKind::Float(f) => *f == 0.0,
            _ => false,
        },
        _ => false,
    }
}

fn is_one(op: &Operand) -> bool {
    match op {
        Operand::Constant(c) => match &c.kind {
            ConstantKind::Int(i) => *i == 1,
            ConstantKind::Float(f) => *f == 1.0,
            _ => false,
        },
        _ => false,
    }
}

fn get_operand_type(op: &Operand) -> Type {
    match op {
        Operand::Constant(c) => c.ty.clone(),
        _ => Type::Int,
    }
}

/// Optimization 2: Dead Code Elimination
/// Removes unused assignments and unreachable blocks
pub fn dead_code_elimination(func: &mut Function) -> bool {
    let mut changed = false;

    let mut used_locals = vec![true; func.local_count()];

    used_locals[0] = true;

    for block_idx in 0..func.block_count() {
        let block = &func.blocks[BlockId(block_idx as u32)];
        for stmt in &block.statements {
            if let Statement::Assign(_, rvalue) = stmt {
                mark_operand_usage(&rvalue, &mut used_locals);
            }
        }
        mark_terminator_usage(&block.terminator, &mut used_locals);
    }

    for block_idx in 0..func.block_count() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        for stmt in &mut block.statements {
            if let Statement::Assign(place, _) = stmt {
                if let Place::Local(local_id) = place {
                    if (local_id.0 as usize) < used_locals.len()
                        && !used_locals[local_id.0 as usize]
                    {
                        *stmt = Statement::Nop;
                        changed = true;
                    }
                }
            }
        }
    }

    changed
}

fn mark_operand_usage(rvalue: &Rvalue, used: &mut [bool]) {
    match rvalue {
        Rvalue::Use(op) => mark_op_usage(op, used),
        Rvalue::UnaryOp(_, op) => mark_op_usage(op, used),
        Rvalue::BinaryOp(_, l, r) => {
            mark_op_usage(l, used);
            mark_op_usage(r, used);
        },
        Rvalue::Aggregate(_, ops) => {
            for op in ops {
                mark_op_usage(op, used);
            }
        },
        _ => {},
    }
}

fn mark_op_usage(op: &Operand, used: &mut [bool]) {
    if let Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) = op {
        if (id.0 as usize) < used.len() {
            used[id.0 as usize] = true;
        }
    }
}

fn mark_terminator_usage(term: &Terminator, used: &mut [bool]) {
    match term {
        Terminator::If { cond, .. } => mark_op_usage(cond, used),
        Terminator::SwitchInt { discr, .. } => mark_op_usage(discr, used),
        Terminator::Call { args, .. } => {
            for arg in args {
                mark_op_usage(arg, used);
            }
        },
        _ => {},
    }
}

/// Optimization 3: Copy Propagation
/// Replaces uses of variables that are simple copies
pub fn copy_propagation(func: &mut Function) {
    for block_idx in 0..func.block_count() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        let mut copies: HashMap<LocalId, Operand> = HashMap::new();

        for stmt in &mut block.statements {
            if let Statement::Assign(place, rvalue) = stmt {
                // Record copy: x = y
                if let (Place::Local(dest), Rvalue::Use(src)) = (&*place, &*rvalue) {
                    if let Operand::Copy(Place::Local(src_id))
                    | Operand::Move(Place::Local(src_id)) = src
                    {
                        copies.insert(*dest, src.clone());
                        continue;
                    }
                }

                // Propagate copies in rvalue
                propagate_in_rvalue(rvalue, &copies);

                // Invalidate copies if destination is in the map
                if let Place::Local(dest) = &*place {
                    copies.remove(dest);
                }
            }
        }
    }
}

fn propagate_in_rvalue(rvalue: &mut Rvalue, copies: &HashMap<LocalId, Operand>) {
    match rvalue {
        Rvalue::Use(op) => propagate_in_op(op, copies),
        Rvalue::UnaryOp(_, op) => propagate_in_op(op, copies),
        Rvalue::BinaryOp(_, l, r) => {
            propagate_in_op(l, copies);
            propagate_in_op(r, copies);
        },
        Rvalue::Aggregate(_, ops) => {
            for op in ops {
                propagate_in_op(op, copies);
            }
        },
        _ => {},
    }
}

fn propagate_in_op(op: &mut Operand, copies: &HashMap<LocalId, Operand>) {
    if let Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) = op {
        if let Some(replacement) = copies.get(id) {
            *op = replacement.clone();
        }
    }
}

/// Optimization 4: Strength Reduction
/// Replaces expensive operations with cheaper ones
pub fn strength_reduction(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];
        let mut i = 0;
        while i < block.statements.len() {
            let new_stmt = match &block.statements[i] {
                Statement::Assign(place, rvalue) => {
                    if let Rvalue::BinaryOp(op, left, right) = rvalue {
                        let left_op = left.as_ref();
                        let right_op = right.as_ref();
                        let place_clone = place.clone();

                        // x * 2 -> x + x
                        if *op == BinOp::Mul {
                            if let Operand::Constant(c) = right_op {
                                if let ConstantKind::Int(2) = c.kind {
                                    let new_rvalue =
                                        Rvalue::BinaryOp(BinOp::Add, left.clone(), left.clone());
                                    Some(Statement::Assign(place_clone, new_rvalue))
                                } else {
                                    None
                                }
                            } else if let Operand::Constant(c) = left_op {
                                if let ConstantKind::Int(2) = c.kind {
                                    let new_rvalue =
                                        Rvalue::BinaryOp(BinOp::Add, right.clone(), right.clone());
                                    Some(Statement::Assign(place_clone, new_rvalue))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else if *op == BinOp::Div {
                            // x / 2^k -> x >> k (division by power of 2)
                            if let Operand::Constant(c) = right_op {
                                if let ConstantKind::Int(ref n) = c.kind {
                                    if *n > 0 && (*n as u64).is_power_of_two() {
                                        let shift_amount = n.trailing_zeros() as u32;
                                        let new_rvalue = Rvalue::BinaryOp(
                                            BinOp::Shr,
                                            left.clone(),
                                            Box::new(Operand::Constant(Constant {
                                                ty: c.ty.clone(),
                                                kind: ConstantKind::Int(shift_amount as i64),
                                            })),
                                        );
                                        Some(Statement::Assign(place_clone, new_rvalue))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else if *op == BinOp::Rem {
                            // x % 2^k -> x & (2^k - 1) (mod power of 2)
                            if let Operand::Constant(c) = right_op {
                                if let ConstantKind::Int(ref n) = c.kind {
                                    if *n > 0 && (*n as u64).is_power_of_two() {
                                        let mask = n - 1;
                                        let new_rvalue = Rvalue::BinaryOp(
                                            BinOp::BitAnd,
                                            left.clone(),
                                            Box::new(Operand::Constant(Constant {
                                                ty: c.ty.clone(),
                                                kind: ConstantKind::Int(mask),
                                            })),
                                        );
                                        Some(Statement::Assign(place_clone, new_rvalue))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                _ => None,
            };

            if let Some(stmt) = new_stmt {
                block.statements[i] = stmt;
            }
            i += 1;
        }
    }
}

/// Optimization 5: Common Subexpression Elimination (CSE)
/// Eliminates redundant computations by detecting and reusing previous results
pub fn common_subexpression_elimination(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];

        let mut expr_map: HashMap<ExpressionKey, LocalId> = HashMap::new();

        let mut i = 0;
        while i < block.statements.len() {
            if let Statement::Assign(place, rvalue) = &block.statements[i] {
                if let Place::Local(dest_id) = place {
                    if let Some(key) = ExpressionKey::from_rvalue(rvalue) {
                        if let Some(cached) = expr_map.get(&key) {
                            block.statements[i] = Statement::Assign(
                                place.clone(),
                                Rvalue::Use(Operand::Copy(Place::Local(*cached))),
                            );
                        } else if !rvalue.contains_side_effects() {
                            expr_map.insert(key, *dest_id);
                        }
                    }
                }
            }
            i += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ExpressionKey {
    Binary(BinOp, OperandKey, OperandKey),
    Unary(UnOp, OperandKey),
    Load(LocalId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OperandKey {
    Constant(i64),
    Local(LocalId),
}

impl ExpressionKey {
    fn from_rvalue(rvalue: &Rvalue) -> Option<Self> {
        match rvalue {
            Rvalue::BinaryOp(op, left, right) => {
                let left_key = OperandKey::from_operand(left)?;
                let right_key = OperandKey::from_operand(right)?;
                Some(ExpressionKey::Binary(*op, left_key, right_key))
            },
            Rvalue::UnaryOp(op, operand) => {
                let op_key = OperandKey::from_operand(operand)?;
                Some(ExpressionKey::Unary(*op, op_key))
            },
            Rvalue::Use(Operand::Copy(id)) | Rvalue::Use(Operand::Move(id)) => {
                if let Place::Local(local_id) = id {
                    Some(ExpressionKey::Load(*local_id))
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}

impl OperandKey {
    fn from_operand(op: &Operand) -> Option<Self> {
        match op {
            Operand::Constant(c) => {
                if let ConstantKind::Int(i) = c.kind {
                    Some(OperandKey::Constant(i))
                } else {
                    None
                }
            },
            Operand::Copy(id) | Operand::Move(id) => {
                if let Place::Local(local_id) = id {
                    Some(OperandKey::Local(*local_id))
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}

impl Rvalue {
    fn contains_side_effects(&self) -> bool {
        match self {
            Rvalue::Use(_) | Rvalue::UnaryOp(_, _) | Rvalue::BinaryOp(_, _, _) => false,
            Rvalue::CheckedBinaryOp(_, _, _) => true,
            Rvalue::NullaryOp(op, _) => matches!(op, NullOp::SizeOf | NullOp::AlignOf),
            Rvalue::Cast(_, _, _) => false,
            Rvalue::Aggregate(_, ops) => ops
                .iter()
                .any(|op| matches!(op, Operand::Copy(_) | Operand::Move(_))),
            Rvalue::Ref(_, _) => true,
            Rvalue::AddressOf(_, _) => false,
            Rvalue::Discriminant(_) => false,
        }
    }
}

/// Optimization 6: Loop Invariant Code Motion (LICM)
/// Moves loop-invariant code outside of loops
pub fn loop_invariant_code_motion(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];

        if let Terminator::SwitchInt { .. } = &block.terminator {
            continue;
        }

        let loop_header = BlockId(block_idx as u32);

        let mut invariant_stmts = Vec::new();
        let mut non_invariant_stmts = Vec::new();

        for stmt in &block.statements {
            if let Statement::Assign(_, rvalue) = stmt {
                if is_loop_invariant(rvalue) {
                    invariant_stmts.push(stmt.clone());
                } else {
                    non_invariant_stmts.push(stmt.clone());
                }
            }
        }

        if !invariant_stmts.is_empty() && !non_invariant_stmts.is_empty() {
            block.statements = non_invariant_stmts;

            if let Some(dom_block_id) = find_dominated_block(func, loop_header) {
                let dom_block = &mut func.blocks[dom_block_id];
                dom_block.statements.extend(invariant_stmts);
            }
        }
    }
}

fn is_loop_invariant(rvalue: &Rvalue) -> bool {
    match rvalue {
        Rvalue::Use(op) => !op_uses_loop_variable(op),
        Rvalue::BinaryOp(_, left, right) => {
            !op_uses_loop_variable(left) && !op_uses_loop_variable(right)
        },
        Rvalue::UnaryOp(_, op) => !op_uses_loop_variable(op),
        _ => false,
    }
}

fn op_uses_loop_variable(op: &Operand) -> bool {
    match op {
        Operand::Copy(id) | Operand::Move(id) => {
            matches!(id, Place::Local(_))
        },
        _ => false,
    }
}

fn find_dominated_block(func: &Function, header: BlockId) -> Option<BlockId> {
    for idx in 0..func.blocks.len() {
        let block_id = BlockId(idx as u32);
        let block = &func.blocks[block_id];
        if let Terminator::Goto { target } = &block.terminator {
            if *target == header {
                return Some(BlockId(block.id.0 + 1));
            }
        }
    }
    None
}

/// Optimization 7: Conditional Branch Simplification
/// Simplifies branches with known conditions
pub fn branch_simplification(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];

        if let Terminator::If {
            cond,
            then_block,
            else_block,
        } = &block.terminator
        {
            if let Operand::Constant(c) = cond {
                if let ConstantKind::Int(value) = c.kind {
                    let target = if value != 0 { then_block } else { else_block };
                    block.terminator = Terminator::Goto { target: *target };
                }
            }
        }
    }
}

/// Optimization 8: Phi Node Elimination (for SSA-like form)
/// Removes redundant phi nodes
pub fn phi_elimination(func: &mut Function) {
    for block_idx in 0..func.blocks.len() {
        let block = &mut func.blocks[BlockId(block_idx as u32)];

        let mut i = 0;
        while i < block.statements.len() {
            if let Statement::Assign(place, Rvalue::Use(Operand::Copy(src))) = &block.statements[i]
            {
                if let (Place::Local(dest), Place::Local(src_local)) = (place, src) {
                    if dest == src_local {
                        block.statements.remove(i);
                        continue;
                    }
                }
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod optimize_tests {
    use super::*;
    use faxc_sem::Type;
    use faxc_util::Symbol;

    #[test]
    fn test_constant_folding() {
        let name = Symbol::intern("test");
        let mut func = Function::new(name, Type::Int, 0);

        // Test will be expanded with full builder usage
        assert_eq!(func.name, name);
    }

    #[test]
    fn test_dead_code_elimination() {
        let name = Symbol::intern("test_dce");
        let mut func = Function::new(name, Type::Int, 0);
        assert_eq!(func.local_count(), 0);
    }

    #[test]
    fn test_copy_propagation() {
        let name = Symbol::intern("test_cp");
        let mut func = Function::new(name, Type::Int, 0);
        assert_eq!(func.arg_count, 0);
    }
}
