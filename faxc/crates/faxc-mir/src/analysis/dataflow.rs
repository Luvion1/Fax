//! Data Flow Analysis for MIR
//!
//! Provides various data flow analyses for optimization:
//! - Liveness Analysis
//! - Available Expressions Analysis
//! - Reaching Definitions Analysis

use crate::cfg::ControlFlowGraph;
use crate::mir::*;
use std::collections::{HashMap, HashSet};

pub struct LivenessAnalysis {
    pub block_entry: HashMap<BlockId, HashSet<LocalId>>,
    pub block_exit: HashMap<BlockId, HashSet<LocalId>>,
}

impl LivenessAnalysis {
    pub fn new() -> Self {
        Self {
            block_entry: HashMap::new(),
            block_exit: HashMap::new(),
        }
    }
}

impl Default for LivenessAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

pub fn analyze_liveness(func: &Function, cfg: &ControlFlowGraph) -> LivenessAnalysis {
    let block_count = func.blocks.len();
    let mut block_entry: HashMap<BlockId, HashSet<LocalId>> = HashMap::new();
    let mut block_exit: HashMap<BlockId, HashSet<LocalId>> = HashMap::new();

    for (block_id, _) in func.blocks.iter_enumerated() {
        block_entry.insert(block_id, HashSet::new());
        block_exit.insert(block_id, HashSet::new());
    }

    let mut changed = true;
    let max_iterations = block_count * block_count;
    let mut iterations = 0;

    while changed && iterations < max_iterations {
        changed = false;
        iterations += 1;

        for (block_id, block) in func.blocks.iter_enumerated() {
            let mut out = HashSet::new();

            if let Some(succs) = cfg.successors.get(&block_id) {
                for &succ in succs {
                    if let Some(entry) = block_entry.get(&succ) {
                        out.extend(entry.iter());
                    }
                }
            }

            let in_set = compute_block_in(block, &out);

            if let Some(old_in) = block_entry.get(&block_id) {
                if &in_set != old_in {
                    changed = true;
                }
            }
            block_entry.insert(block_id, in_set.clone());
            block_exit.insert(block_id, out);
        }
    }

    LivenessAnalysis {
        block_entry,
        block_exit,
    }
}

fn compute_block_in(block: &BasicBlock, out: &HashSet<LocalId>) -> HashSet<LocalId> {
    let mut uses = HashSet::new();
    let mut defines = HashSet::new();

    for stmt in &block.statements {
        match stmt {
            Statement::Assign(place, rvalue) => {
                if let Place::Local(id) = place {
                    defines.insert(*id);
                }
                rvalue_uses(rvalue, &mut uses);
            },
            Statement::Nop => {},
            Statement::StorageLive(id) | Statement::StorageDead(id) => {
                defines.insert(*id);
            },
        }
    }

    let mut term_uses = HashSet::new();
    collect_terminator_uses(&block.terminator, &mut term_uses);
    uses.extend(term_uses);

    let mut result = uses;
    for id in out {
        if !defines.contains(id) {
            result.insert(*id);
        }
    }
    result
}

fn rvalue_uses(rvalue: &Rvalue, uses: &mut HashSet<LocalId>) {
    match rvalue {
        Rvalue::Use(op) => operand_uses(op, uses),
        Rvalue::BinaryOp(_, left, right) => {
            operand_uses(left, uses);
            operand_uses(right, uses);
        },
        Rvalue::UnaryOp(_, op) => operand_uses(op, uses),
        Rvalue::CheckedBinaryOp(_, left, right) => {
            operand_uses(left, uses);
            operand_uses(right, uses);
        },
        _ => {},
    }
}

fn operand_uses(op: &Operand, uses: &mut HashSet<LocalId>) {
    match op {
        Operand::Copy(place) | Operand::Move(place) => {
            if let Place::Local(id) = place {
                uses.insert(*id);
            }
        },
        Operand::Constant(_) => {},
    }
}

fn collect_terminator_uses(term: &Terminator, uses: &mut HashSet<LocalId>) {
    match term {
        Terminator::If { cond, .. } => operand_uses(cond, uses),
        Terminator::SwitchInt { discr, .. } => operand_uses(discr, uses),
        Terminator::Call { args, .. } => {
            for arg in args {
                operand_uses(arg, uses);
            }
        },
        _ => {},
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ExprKey {
    pub op: u8,
    pub left: LocalId,
    pub right: Option<LocalId>,
}

pub struct AvailableExpressions {
    pub gen: HashMap<BlockId, HashSet<ExprKey>>,
    pub kill: HashMap<BlockId, HashSet<ExprKey>>,
    pub block_entry: HashMap<BlockId, HashSet<ExprKey>>,
    pub block_exit: HashMap<BlockId, HashSet<ExprKey>>,
}

impl Default for AvailableExpressions {
    fn default() -> Self {
        Self::new()
    }
}

impl AvailableExpressions {
    pub fn new() -> Self {
        Self {
            gen: HashMap::new(),
            kill: HashMap::new(),
            block_entry: HashMap::new(),
            block_exit: HashMap::new(),
        }
    }
}

pub fn analyze_available_expressions(
    func: &Function,
    cfg: &ControlFlowGraph,
) -> AvailableExpressions {
    let mut analysis = AvailableExpressions::new();

    for (block_id, block) in func.blocks.iter_enumerated() {
        let mut gen = HashSet::new();
        let mut defined = HashSet::new();

        for stmt in &block.statements {
            if let Statement::Assign(place, rvalue) = stmt {
                if let Place::Local(dest) = place {
                    if !defined.contains(dest) {
                        if let Some(expr) = compute_expr_key(rvalue) {
                            gen.insert(expr);
                        }
                    }
                    defined.insert(*dest);
                }
            }
        }

        analysis.gen.insert(block_id, gen);
        analysis.kill.insert(block_id, HashSet::new());
    }

    analysis
        .block_entry
        .insert(func.entry_block, HashSet::new());

    let mut changed = true;
    let max_iterations = func.blocks.len() * func.blocks.len();
    let mut iterations = 0;

    while changed && iterations < max_iterations {
        changed = false;
        iterations += 1;

        for (block_id, _) in func.blocks.iter_enumerated() {
            let mut in_set = HashSet::new();

            if let Some(preds) = cfg.predecessors.get(&block_id) {
                for &pred in preds {
                    if let Some(pred_out) = analysis.block_exit.get(&pred) {
                        for expr in pred_out {
                            in_set.insert(expr.clone());
                        }
                    }
                }
            }

            let old_entry = analysis.block_entry.get(&block_id).cloned();
            if old_entry != Some(in_set.clone()) {
                changed = true;
                analysis.block_entry.insert(block_id, in_set.clone());
            }

            analysis.block_exit.insert(block_id, in_set);
        }
    }

    analysis
}

fn compute_expr_key(rvalue: &Rvalue) -> Option<ExprKey> {
    match rvalue {
        Rvalue::BinaryOp(op, left, right) => {
            let op_code: u8 = match op {
                BinOp::Add => 1,
                BinOp::Sub => 2,
                BinOp::Mul => 3,
                BinOp::Div => 4,
                BinOp::Rem => 5,
                BinOp::BitAnd => 6,
                BinOp::BitOr => 7,
                BinOp::BitXor => 8,
                BinOp::Shl => 9,
                BinOp::Shr => 10,
                _ => return None,
            };

            if let Operand::Copy(Place::Local(l)) | Operand::Move(Place::Local(l)) = left.as_ref() {
                let r = right.as_ref();
                if let Operand::Copy(Place::Local(rid)) | Operand::Move(Place::Local(rid)) = r {
                    return Some(ExprKey {
                        op: op_code,
                        left: *l,
                        right: Some(*rid),
                    });
                }
            }
            None
        },
        _ => None,
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DefId {
    pub block: BlockId,
    pub local: LocalId,
    pub stmt_idx: usize,
}

pub struct ReachingDefinitions {
    pub block_entry: HashMap<BlockId, HashSet<DefId>>,
    pub block_exit: HashMap<BlockId, HashSet<DefId>>,
}

impl Default for ReachingDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

impl ReachingDefinitions {
    pub fn new() -> Self {
        Self {
            block_entry: HashMap::new(),
            block_exit: HashMap::new(),
        }
    }
}

pub fn analyze_reaching_definitions(
    func: &Function,
    cfg: &ControlFlowGraph,
) -> ReachingDefinitions {
    let mut analysis = ReachingDefinitions::new();

    for (block_id, _) in func.blocks.iter_enumerated() {
        analysis.block_entry.insert(block_id, HashSet::new());
        analysis.block_exit.insert(block_id, HashSet::new());
    }

    analysis
        .block_entry
        .insert(func.entry_block, HashSet::new());

    let mut changed = true;
    while changed {
        changed = false;

        for (block_id, block) in func.blocks.iter_enumerated() {
            let mut in_set = HashSet::new();

            if let Some(preds) = cfg.predecessors.get(&block_id) {
                for &pred in preds {
                    if let Some(pred_out) = analysis.block_exit.get(&pred) {
                        for def in pred_out {
                            in_set.insert(def.clone());
                        }
                    }
                }
            }

            let mut out_set = in_set.clone();

            for (stmt_idx, stmt) in block.statements.iter().enumerate() {
                if let Statement::Assign(Place::Local(local), _) = stmt {
                    out_set.retain(|d| d.local != *local);
                    out_set.insert(DefId {
                        block: block_id,
                        local: *local,
                        stmt_idx,
                    });
                }
            }

            let old_entry = analysis.block_entry.get(&block_id).cloned();
            if old_entry != Some(in_set.clone()) {
                changed = true;
                analysis.block_entry.insert(block_id, in_set);
            }

            analysis.block_exit.insert(block_id, out_set);
        }
    }

    analysis
}
