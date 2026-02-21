use crate::mir::*;
use faxc_sem::{hir, Type};
use faxc_util::{IndexVec, Span, Symbol};

/// MIR Builder
pub struct Builder {
    pub function: Function,
    pub current_block: BlockId,
    pub block_counter: u32,
}

impl Builder {
    pub fn new(name: Symbol, return_ty: Type) -> Self {
        let mut function = Function {
            name,
            locals: IndexVec::new(),
            blocks: IndexVec::new(),
            entry_block: BlockId(0),
            return_ty: return_ty.clone(),
        };

        // Add return place (local 0)
        function.locals.push(Local {
            ty: return_ty,
            span: Span::DUMMY,
            name: Some(Symbol::intern("return")),
        });

        Self {
            function,
            current_block: BlockId(0),
            block_counter: 0,
        }
    }

    pub fn add_local(&mut self, ty: Type, name: Option<hir::Pattern>) -> LocalId {
        let name_symbol = name.as_ref().map(|p| match p {
            hir::Pattern::Binding { name, .. } => *name,
            _ => Symbol::intern("tmp"),
        });

        self.function.locals.push(Local {
            ty,
            span: Span::DUMMY,
            name: name_symbol,
        })
    }

    pub fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.block_counter);
        self.block_counter += 1;

        self.function.blocks.push(BasicBlock {
            id,
            statements: Vec::new(),
            terminator: Terminator::Unreachable,
        });

        id
    }

    pub fn set_current_block(&mut self, block: BlockId) {
        self.current_block = block;
    }

    pub fn statement(&mut self, stmt: Statement) {
        self.function.blocks[self.current_block].statements.push(stmt);
    }

    pub fn assign(&mut self, place: Place, rvalue: Rvalue) {
        self.statement(Statement::Assign(place, rvalue));
    }

    pub fn terminator(&mut self, terminator: Terminator) {
        self.function.blocks[self.current_block].terminator = terminator;
    }

    pub fn build(mut self) -> Function {
        if self.function.blocks.is_empty() {
            let entry = self.new_block();
            self.function.entry_block = entry;
        }
        self.function
    }
}
