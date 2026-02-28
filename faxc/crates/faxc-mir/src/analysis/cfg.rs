//! Control Flow Analysis for MIR
//!
//! Provides control flow graph construction and analysis for optimizations

use crate::mir::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Control Flow Graph
pub struct ControlFlowGraph {
    /// Basic block graph (block_id -> set of predecessors)
    pub predecessors: HashMap<BlockId, HashSet<BlockId>>,
    /// Basic block graph (block_id -> set of successors)
    pub successors: HashMap<BlockId, HashSet<BlockId>>,
    /// Dominators for each block
    pub dominators: HashMap<BlockId, HashSet<BlockId>>,
    /// Immediate dominators
    pub idom: HashMap<BlockId, BlockId>,
    /// Postorder number for each block
    pub postorder: HashMap<BlockId, u32>,
}

impl ControlFlowGraph {
    /// Build CFG from function
    pub fn new(func: &Function) -> Self {
        let mut predecessors: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
        let mut successors: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();

        // Initialize all blocks
        for (block_id, _) in func.blocks.iter_enumerated() {
            predecessors.insert(block_id, HashSet::new());
            successors.insert(block_id, HashSet::new());
        }

        // Build edges from terminators
        for (block_id, block) in func.blocks.iter_enumerated() {
            let term = &block.terminator;
            let targets = terminator_targets(term);

            for target in targets {
                // Add edge from current block to target
                successors.get_mut(&block_id).unwrap().insert(target);
                predecessors.get_mut(&target).unwrap().insert(block_id);
            }
        }

        let postorder = compute_postorder(&successors, func.entry_block);

        let mut cfg = Self {
            predecessors,
            successors,
            dominators: HashMap::new(),
            idom: HashMap::new(),
            postorder,
        };

        // Compute dominators
        cfg.compute_dominators(func);

        cfg
    }

    /// Compute dominators using algorithm
    fn compute_dominators(&mut self, func: &Function) {
        let blocks: Vec<BlockId> = func.blocks.iter_enumerated().map(|(id, _)| id).collect();

        if blocks.is_empty() {
            return;
        }

        let entry = func.entry_block;

        // Initialize: entry dominates itself, no other block is dominated
        let mut doms: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
        for block_id in &blocks {
            let mut set = HashSet::new();
            if *block_id == entry {
                set.insert(entry);
            } else {
                // Initially, all blocks dominate every block except entry
                for b in &blocks {
                    set.insert(*b);
                }
            }
            doms.insert(*block_id, set);
        }

        // Iterate until fixed point
        let mut changed = true;
        while changed {
            changed = false;

            for block_id in &blocks {
                if *block_id == entry {
                    continue;
                }

                let mut new_dom = HashSet::new();
                // Start with just the block itself
                new_dom.insert(*block_id);

                // Intersect dominators of all predecessors
                if let Some(preds) = self.predecessors.get(block_id) {
                    if !preds.is_empty() {
                        let mut first = true;
                        for pred in preds {
                            if let Some(pred_dom) = doms.get(pred) {
                                if first {
                                    new_dom = pred_dom.clone();
                                    first = false;
                                } else {
                                    new_dom = new_dom.intersection(pred_dom).cloned().collect();
                                }
                            }
                        }
                        new_dom.insert(*block_id);
                    }
                }

                if new_dom != doms[block_id] {
                    doms.insert(*block_id, new_dom.clone());
                    changed = true;
                }
            }
        }

        self.dominators = doms;

        // Compute immediate dominators
        self.compute_immediate_dominators(func);
    }

    /// Compute immediate dominators
    fn compute_immediate_dominators(&mut self, func: &Function) {
        let blocks: Vec<BlockId> = func.blocks.iter_enumerated().map(|(id, _)| id).collect();
        let entry = func.entry_block;

        for block_id in &blocks {
            if *block_id == entry {
                continue;
            }

            if let Some(doms) = self.dominators.get(block_id) {
                // Find immediate dominator: the closest dominator that is not this block
                let mut idom_candidate: Option<BlockId> = None;

                for candidate in &blocks {
                    if *candidate == *block_id {
                        continue;
                    }
                    if doms.contains(candidate) {
                        // Check if candidate dominates all other dominators
                        let mut is_idom = true;
                        for other in &blocks {
                            if *other == *candidate || *other == *block_id {
                                continue;
                            }
                            if doms.contains(other) && !self.dominators[other].contains(candidate) {
                                is_idom = false;
                                break;
                            }
                        }
                        if is_idom {
                            if idom_candidate.is_none()
                                || self.postorder[&candidate]
                                    > self.postorder[&idom_candidate.unwrap()]
                            {
                                idom_candidate = Some(*candidate);
                            }
                        }
                    }
                }

                if let Some(idom) = idom_candidate {
                    self.idom.insert(*block_id, idom);
                }
            }
        }
    }

    /// Check if block is reachable from entry
    pub fn is_reachable(&self, block: BlockId, func: &Function) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(func.entry_block);
        visited.insert(func.entry_block);

        while let Some(current) = queue.pop_front() {
            if current == block {
                return true;
            }

            if let Some(succs) = self.successors.get(&current) {
                for &succ in succs {
                    if !visited.contains(&succ) {
                        visited.insert(succ);
                        queue.push_back(succ);
                    }
                }
            }
        }

        false
    }

    /// Get natural loops in the function
    pub fn find_loops(&self) -> HashMap<BlockId, LoopInfo> {
        let mut loops = HashMap::new();

        for (header, preds) in &self.predecessors {
            // A block is a loop header if it has a predecessor that is a successor
            if preds.contains(header) {
                // Find all blocks in the loop
                let mut loop_blocks = HashSet::new();
                let mut queue = VecDeque::new();

                for pred in preds {
                    if *pred != *header {
                        queue.push_back(*pred);
                    }
                }

                while let Some(block) = queue.pop_front() {
                    if block == *header {
                        continue;
                    }
                    if loop_blocks.contains(&block) {
                        continue;
                    }
                    loop_blocks.insert(block);

                    if let Some(block_preds) = self.predecessors.get(&block) {
                        for pred in block_preds {
                            if !loop_blocks.contains(pred) {
                                queue.push_back(*pred);
                            }
                        }
                    }
                }

                loops.insert(
                    *header,
                    LoopInfo {
                        header: *header,
                        blocks: loop_blocks,
                    },
                );
            }
        }

        loops
    }
}

/// Loop information
#[derive(Debug)]
pub struct LoopInfo {
    pub header: BlockId,
    pub blocks: HashSet<BlockId>,
}

/// Get target blocks from terminator
fn terminator_targets(term: &Terminator) -> Vec<BlockId> {
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
            let mut result: Vec<BlockId> = targets.iter().map(|(_, id)| *id).collect();
            result.push(*otherwise);
            result
        },
        Terminator::Return | Terminator::Unreachable | Terminator::Resume | Terminator::Abort => {
            vec![]
        },
        Terminator::Call { target, .. } => {
            if let Some(t) = target {
                vec![*t]
            } else {
                vec![]
            }
        },
    }
}

/// Compute postorder numbering
fn compute_postorder(
    successors: &HashMap<BlockId, HashSet<BlockId>>,
    entry: BlockId,
) -> HashMap<BlockId, u32> {
    let mut visited = HashSet::new();
    let mut postorder = HashMap::new();
    let mut counter = 0;

    fn dfs(
        node: BlockId,
        successors: &HashMap<BlockId, HashSet<BlockId>>,
        visited: &mut HashSet<BlockId>,
        postorder: &mut HashMap<BlockId, u32>,
        counter: &mut u32,
    ) {
        visited.insert(node);

        if let Some(succs) = successors.get(&node) {
            for &succ in succs {
                if !visited.contains(&succ) {
                    dfs(succ, successors, visited, postorder, counter);
                }
            }
        }

        postorder.insert(node, *counter);
        *counter += 1;
    }

    dfs(
        entry,
        successors,
        &mut visited,
        &mut postorder,
        &mut counter,
    );
    postorder
}

/// Check if a block dominates another
pub fn dominates(cfg: &ControlFlowGraph, a: BlockId, b: BlockId) -> bool {
    if let Some(doms) = cfg.dominators.get(&b) {
        doms.contains(&a)
    } else {
        false
    }
}

/// Get all blocks that a block dominates
pub fn dominated_blocks(cfg: &ControlFlowGraph, block: BlockId) -> Vec<BlockId> {
    cfg.dominators
        .iter()
        .filter(|(_, doms)| doms.contains(&block))
        .map(|(id, _)| *id)
        .collect()
}
