use faxc_util::{Idx, IndexVec, Symbol, DefId};
use crate::hir::LabelId;
use std::collections::HashMap;

/// Rib ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RibId(pub u32);

impl Idx for RibId {
    fn from_usize(idx: usize) -> Self {
        RibId(idx as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

/// A single scope (rib)
#[derive(Debug)]
pub struct Rib {
    /// Bindings in this scope
    pub bindings: HashMap<Symbol, DefId>,
    /// Parent rib
    pub parent: Option<RibId>,
    /// Kind of rib
    pub kind: RibKind,
}

/// Kind of rib
#[derive(Debug, Clone, Copy)]
pub enum RibKind {
    Module,
    Function,
    Block,
    Loop(Option<LabelId>),
}

/// Scope tree for name resolution
pub struct ScopeTree {
    /// All ribs (scopes)
    pub ribs: IndexVec<RibId, Rib>,
    /// Current rib stack
    pub current_rib: RibId,
}

impl ScopeTree {
    /// Create new scope tree
    pub fn new() -> Self {
        let mut ribs = IndexVec::new();
        let root = ribs.push(Rib {
            bindings: HashMap::new(),
            parent: None,
            kind: RibKind::Module,
        });

        Self {
            ribs,
            current_rib: root,
        }
    }

    /// Enter new scope
    pub fn enter_scope(&mut self, kind: RibKind) -> RibId {
        let new_rib = self.ribs.push(Rib {
            bindings: HashMap::new(),
            parent: Some(self.current_rib),
            kind,
        });
        self.current_rib = new_rib;
        new_rib
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.ribs[self.current_rib].parent {
            self.current_rib = parent;
        }
    }

    /// Add binding to current scope
    pub fn add_binding(&mut self, name: Symbol, def_id: DefId) {
        self.ribs[self.current_rib].bindings.insert(name, def_id);
    }

    /// Resolve name to definition
    pub fn resolve(&self, name: Symbol) -> Option<DefId> {
        let mut rib_id = self.current_rib;

        loop {
            let rib = &self.ribs[rib_id];

            if let Some(&def_id) = rib.bindings.get(&name) {
                return Some(def_id);
            }

            match rib.parent {
                Some(parent) => rib_id = parent,
                None => return None,
            }
        }
    }
}
