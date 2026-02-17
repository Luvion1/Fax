//! faxc-mir - Mid-Level Intermediate Representation (MIR)
//!
//! ============================================================================
//! MIR - MID-LEVEL INTERMEDIATE REPRESENTATION
//! ============================================================================
//!
//! MIR adalah representasi intermediate yang berada di tengah-tengah
//! compiler pipeline. MIR dirancang untuk:
//!
//! 1. CONTROL FLOW EXPLICIT
//!    Semua branching dan control flow direpresentasikan secara eksplisit
//!    dalam bentuk Control Flow Graph (CFG).
//!
//! 2. STATIC SINGLE ASSIGNMENT (SSA) FORM
//!    Setiap variable hanya di-assign sekali, memudahkan analisis dan optimasi.
//!
//! 3. TYPE-EXPLICIT
//!    Setiap value memiliki type yang jelas dan tercatat.
//!
//! 4. SIMPLIFIED SEMANTICS
//!    Konstruksi bahasa tingkat tinggi di-lower ke operasi primitif.
//!
//! ============================================================================
//! CONTROL FLOW GRAPH (CFG)
//! ============================================================================
//!
//! CFG adalah directed graph yang merepresentasikan alur kontrol program:
//! - Node = Basic Block (BB)
//! - Edge = Possible control flow between blocks
//!
//! BASIC BLOCK (BB):
//! -----------------
//! Sebuah basic block adalah sequence of instructions dengan:
//! - Single entry point (hanya bisa masuk dari instruction pertama)
//! - Single exit point (hanya bisa keluar dari instruction terakhir)
//! - No internal branching
//!
//! Contoh:
//! ```
//! BB0:
//!   x = 5
//!   y = 10
//!   if x > y goto BB1 else BB2
//!
//! BB1:  // then block
//!   z = x + y
//!   goto BB3
//!
//! BB2:  // else block
//!   z = x - y
//!   goto BB3
//!
//! BB3:  // merge block
//!   return z
//! ```
//!
//! SSA FORM:
//! ---------
//! Static Single Assignment: Setiap variable hanya di-assign sekali.
//!
//! Non-SSA:
//! ```
//! x = 5
//! x = x + 1
//! y = x * 2
//! ```
//!
//! SSA:
//! ```
//! x1 = 5
//! x2 = x1 + 1
//! y1 = x2 * 2
//! ```
//!
//! Keuntungan SSA:
//! - Clear def-use chains
//! - Easy constant propagation
//! - Dead code elimination sederhana
//! - Clear data flow analysis
//!
//! PHI NODES:
//! ----------
//! PHI nodes adalah mekanisme untuk merge values dari multiple predecessors
//! dalam SSA form.
//!
//! ```
//! BB1:          BB2:
//!   x1 = 1        x2 = 2
//!   goto BB3      goto BB3
//!
//! BB3:
//!   x3 = φ(x1: BB1, x2: BB2)  // PHI node
//!   use(x3)
//! ```
//!
//! PHI node memilih value berdasarkan basic block mana yang baru saja
//! dieksekusi.
//!
//! ============================================================================
//! MIR INSTRUCTIONS
//! ============================================================================
//!
//! MIR menggunakan 3-address code format:
//!   result = operand1 op operand2
//!
//! INSTRUCTION TYPES:
//!
//! 1. ASSIGNMENT
//!    dest = source
//!    dest = constant
//!    dest = φ(values...)
//!
//! 2. ARITHMETIC
//!    dest = lhs + rhs
//!    dest = lhs - rhs
//!    dest = lhs * rhs
//!    dest = lhs / rhs
//!    dest = lhs % rhs
//!
//! 3. COMPARISON
//!    dest = lhs == rhs
//!    dest = lhs != rhs
//!    dest = lhs < rhs
//!    dest = lhs > rhs
//!    dest = lhs <= rhs
//!    dest = lhs >= rhs
//!
//! 4. LOGICAL
//!    dest = !operand
//!    dest = lhs && rhs
//!    dest = lhs || rhs
//!
//! 5. MEMORY OPERATIONS
//!    dest = *address           // Dereference
//!    *address = value          // Store
//!    dest = &place             // Reference
//!    dest = &raw place         // Raw pointer
//!
//! 6. AGGREGATE OPERATIONS
//!    dest = (v1, v2, v3)      // Tuple construction
//!    dest = [v1, v2, v3]      // Array construction
//!    dest = aggregate.field    // Field access
//!
//! 7. CALLS
//!    dest = call func(args...)
//!
//! 8. TERMINATORS (basic block endings)
//!    goto block
//!    if cond then block1 else block2
//!    switch value [case1: block1, ...]
//!    return value
//!    unreachable
//!
//! ============================================================================
//! LOWERING HIR KE MIR
//! ============================================================================
//!
//! Proses mengubah High-level IR (HIR) ke MIR:
//!
//! 1. Create CFG structure
//! 2. Lower expressions ke MIR instructions
//! 3. Insert PHI nodes where needed
//! 4. Generate terminators
//!
//! Example Lowering:
//!
//! HIR:
//! ```
//! let x = if condition { a } else { b };
//! ```
//!
//! MIR:
//! ```
//! BB0:
//!   // evaluate condition
//!   cond_val = ...
//!   if cond_val then BB1 else BB2
//!
//! BB1:
//!   // then branch
//!   a_val = ...
//!   goto BB3
//!
//! BB2:
//!   // else branch
//!   b_val = ...
//!   goto BB3
//!
//! BB3:
//!   // merge
//!   x = φ(a_val: BB1, b_val: BB2)
//! ```
//!
//! ============================================================================
//! OPTIMIZATIONS ON MIR
//! ============================================================================
//!
//! MIR adalah level yang ideal untuk banyak optimasi:
//!
//! 1. CONSTANT FOLDING
//!    Mengganti operasi dengan hasilnya jika operands adalah constant.
//!
//!    Before: x = 2 + 3
//!    After:  x = 5
//!
//! 2. DEAD CODE ELIMINATION
//!    Menghapus instructions yang tidak digunakan.
//!
//!    Before:
//!      x = 5
//!      y = x + 1
//!      z = 10
//!      return y
//!
//!    After:
//!      x = 5
//!      y = x + 1
//!      return y
//!    (z dihapus karena tidak digunakan)
//!
//! 3. CONSTANT PROPAGATION
//!    Mensubstitusi variable dengan nilai constant-nya.
//!
//!    Before:
//!      x = 5
//!      y = x + 1
//!
//!    After:
//!      x = 5
//!      y = 6
//!
//! 4. COPY PROPAGATION
//!    Mengganti variable dengan copy-nya.
//!
//!    Before:
//!      x = y
//!      z = x + 1
//!
//!    After:
//!      x = y
//!      z = y + 1
//!
//! 5. COMMON SUBEXPRESSION ELIMINATION (CSE)
//!    Mengeliminasi perhitungan yang redundan.
//!
//!    Before:
//!      x = a + b
//!      y = a + b
//!
//!    After:
//!      x = a + b
//!      y = x
//!
//! 6. INLINING
//!    Mengganti function call dengan body function.
//!
//! 7. LOOP OPTIMIZATIONS
//!    - Loop invariant code motion
//!    - Induction variable analysis
//!    - Loop unrolling
//!
//! ============================================================================
//! ASYNC/AWAIT LOWERING
//! ============================================================================
//!
//! Async/await di Fax diimplementasikan menggunakan state machine transformation.
//! Setiap async function atau async block di-transform menjadi state machine
//! yang mengimplementasikan trait Future.
//!
//! ASYNC FUNCTION LOWERING:
//! -------------------------
//!
//! HIR:
//! ```
//! async fn foo(x: int) -> int {
//!     let y = await bar(x);
//!     y + 1
//! }
//! ```
//!
//! Lowering steps:
//! 1. Create state machine struct
//! 2. Transform async fn menjadi regular fn yang return impl Future
//! 3. Generate poll() method untuk state machine
//! 4. Split async block di setiap await point menjadi states
//!
//! MIR untuk poll method:
//! ```
//! poll(self, cx: &mut Context) -> Poll<T> {
//!   match self.state {
//!     State0:
//!       // Entry point
//!       self.x = param_x
//!       self.state = State1
//!       
//!     State1:
//!       // let y = await bar(x);
//!       match bar(self.x).poll(cx) {
//!         Pending => return Pending
//!         Ready(v) => {
//!           self.y = v
//!           self.state = State2
//!         }
//!       }
//!       
//!     State2:
//!       // y + 1
//!       self.result = self.y + 1
//!       return Ready(self.result)
//!   }
//! }
//! ```
//!
//! AWAIT LOWERING:
//! ---------------
//! Await expression diubah menjadi poll() call dengan state transition.
//! Setiap await point adalah suspend point dimana function bisa yield.
//!
//! ASYNC BLOCK:
//! ------------
//! Async block `async { ... }` diperlakukan seperti async function tanpa parameter,
//! di-transform menjadi anonymous Future type.

use faxc_sem::{hir, DefId, Type};
use faxc_util::{Idx, IndexVec, Span, Symbol};

/// MIR Function
#[derive(Debug, Clone)]
pub struct Function {
    /// Function name
    pub name: Symbol,

    /// Local variables (includes parameters, temporaries, return value)
    pub locals: IndexVec<LocalId, Local>,

    /// Basic blocks
    pub blocks: IndexVec<BlockId, BasicBlock>,

    /// Entry block
    pub entry_block: BlockId,

    /// Return type
    pub return_ty: Type,
}

/// Local variable
#[derive(Debug, Clone)]
pub struct Local {
    /// Type of local
    pub ty: Type,

    /// Source span (for debug info)
    pub span: Span,

    /// Name (if any, for debugging)
    pub name: Option<Symbol>,
}

/// Local ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

impl Idx for LocalId {
    fn from_usize(idx: usize) -> Self {
        LocalId(idx as u32)
    }
    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Block ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl Idx for BlockId {
    fn from_usize(idx: usize) -> Self {
        BlockId(idx as u32)
    }
    fn index(self) -> usize {
        self.0 as usize
    }
}

/// Basic Block
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Block ID
    pub id: BlockId,

    /// Statements in the block
    pub statements: Vec<Statement>,

    /// Terminator (always the last "instruction")
    pub terminator: Terminator,
}

/// Statement
#[derive(Debug, Clone)]
pub enum Statement {
    /// Assignment to a place
    Assign(Place, Rvalue),

    /// Storage live (for stack allocation tracking)
    StorageLive(LocalId),

    /// Storage dead (for stack allocation tracking)
    StorageDead(LocalId),

    /// No operation
    Nop,
}

/// Place - a memory location
#[derive(Debug, Clone)]
pub enum Place {
    /// Local variable
    Local(LocalId),

    /// Projection (field, index, deref)
    Projection(Box<Place>, Projection),
}

/// Projection onto a place
#[derive(Debug, Clone)]
pub enum Projection {
    /// Field access
    Field(u32), // Field index

    /// Index access
    Index(LocalId),

    /// Constant index
    ConstantIndex {
        offset: u64,
        min_length: u64,
        from_end: bool,
    },

    /// Dereference
    Deref,

    /// Subslice
    Subslice { from: u64, to: u64, from_end: bool },
}

/// Rvalue - a value that can be assigned
#[derive(Debug, Clone)]
pub enum Rvalue {
    /// Use a value
    Use(Operand),

    /// Reference
    Ref(Place, Mutability),

    /// Raw pointer
    AddressOf(Place, Mutability),

    /// Unary operation
    UnaryOp(UnOp, Operand),

    /// Binary operation
    BinaryOp(BinOp, Box<Operand>, Box<Operand>),

    /// Checked binary operation (returns (result, overflow_flag))
    CheckedBinaryOp(BinOp, Box<Operand>, Box<Operand>),

    /// Nullary operation
    NullaryOp(NullOp, Type),

    /// Cast
    Cast(CastKind, Operand, Type),

    /// Discriminant (for enums)
    Discriminant(Place),

    /// Aggregate (tuple, array, struct construction)
    Aggregate(AggregateKind, Vec<Operand>),
}

/// Operand - an argument to an operation
#[derive(Debug, Clone)]
pub enum Operand {
    /// Copy value from place
    Copy(Place),

    /// Move value from place
    Move(Place),

    /// Constant value
    Constant(Constant),
}

/// Constant value
#[derive(Debug, Clone)]
pub struct Constant {
    pub ty: Type,
    pub kind: ConstantKind,
}

/// Kind of constant
#[derive(Debug, Clone)]
pub enum ConstantKind {
    Int(i64),
    Float(f64),
    String(Symbol),
    Bool(bool),
    Unit,
}

/// Mutability
#[derive(Debug, Clone, Copy)]
pub enum Mutability {
    Mutable,
    Immutable,
}

/// Unary operations
#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg, // Negation
    Not, // Logical/bitwise not
}

/// Binary operations
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Offset,
}

/// Nullary operations
#[derive(Debug, Clone, Copy)]
pub enum NullOp {
    SizeOf,
    AlignOf,
}

/// Cast kinds
#[derive(Debug, Clone, Copy)]
pub enum CastKind {
    IntToInt,
    IntToFloat,
    FloatToInt,
    FloatToFloat,
    PtrToPtr,
    PtrToInt,
    IntToPtr,
}

/// Aggregate kinds
#[derive(Debug, Clone)]
pub enum AggregateKind {
    Tuple,
    Array(Type),
    Struct(DefId),
    Closure(DefId),
}

/// Terminator - ends a basic block
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Unconditional goto
    Goto { target: BlockId },

    /// Conditional branch
    If {
        cond: Operand,
        then_block: BlockId,
        else_block: BlockId,
    },

    /// Switch (match on integer or enum)
    SwitchInt {
        discr: Operand,
        switch_ty: Type,
        targets: Vec<(u128, BlockId)>,
        otherwise: BlockId,
    },

    /// Return from function
    Return,

    /// Unreachable
    Unreachable,

    /// Call a function
    Call {
        func: Operand,
        args: Vec<Operand>,
        destination: Place,
        target: Option<BlockId>,
        cleanup: Option<BlockId>,
    },

    /// Resume from unwind
    Resume,

    /// Abort
    Abort,
}

/// Lower HIR function to MIR
pub fn lower_hir_function(hir_fn: &hir::FnItem) -> Function {
    let mut builder = Builder::new(hir_fn.name.clone(), hir_fn.ret_type.clone());

    // Add parameters as locals
    for param in &hir_fn.params {
        let local = builder.add_local(param.ty.clone(), Some(param.pat.clone()));
        // TODO: Store parameter local
    }

    // Create entry block
    let entry = builder.new_block();
    builder.set_current_block(entry);

    // Lower function body
    builder.lower_expr(&hir_fn.body);

    builder.build()
}

/// MIR Builder
pub struct Builder {
    function: Function,
    current_block: BlockId,
    local_counter: u32,
    block_counter: u32,
}

impl Builder {
    /// Create new builder
    pub fn new(name: Symbol, return_ty: Type) -> Self {
        let mut function = Function {
            name,
            locals: IndexVec::new(),
            blocks: IndexVec::new(),
            entry_block: BlockId(0),
            return_ty,
        };

        // Add return place (local 0)
        let return_local = function.locals.push(Local {
            ty: function.return_ty.clone(),
            span: Span::DUMMY,
            name: Some(Symbol::intern("return")),
        });

        assert_eq!(return_local, LocalId(0));

        Self {
            function,
            current_block: BlockId(0),
            local_counter: 1,
            block_counter: 0,
        }
    }

    /// Add new local variable
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

    /// Create new basic block
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

    /// Set current block
    pub fn set_current_block(&mut self, block: BlockId) {
        self.current_block = block;
    }

    /// Get current block
    pub fn current_block(&self) -> BlockId {
        self.current_block
    }

    /// Add statement to current block
    pub fn statement(&mut self, stmt: Statement) {
        self.function.blocks[self.current_block]
            .statements
            .push(stmt);
    }

    /// Assign to place
    pub fn assign(&mut self, place: Place, rvalue: Rvalue) {
        self.statement(Statement::Assign(place, rvalue));
    }

    /// Set terminator for current block
    pub fn terminator(&mut self, terminator: Terminator) {
        self.function.blocks[self.current_block].terminator = terminator;
    }

    /// Lower HIR expression to MIR
    pub fn lower_expr(&mut self, expr: &hir::Expr) -> Place {
        match expr {
            hir::Expr::Literal { lit, ty } => {
                let constant = match lit {
                    hir::Literal::Int(n) => ConstantKind::Int(*n),
                    hir::Literal::Float(f) => ConstantKind::Float(*f),
                    hir::Literal::String(s) => ConstantKind::String(*s),
                    hir::Literal::Bool(b) => ConstantKind::Bool(*b),
                    hir::Literal::Unit => ConstantKind::Unit,
                };

                let temp = self.add_local(ty.clone(), None);
                let place = Place::Local(temp);

                self.assign(
                    place.clone(),
                    Rvalue::Use(Operand::Constant(Constant {
                        ty: ty.clone(),
                        kind: constant,
                    })),
                );

                place
            }

            hir::Expr::Var { def_id, ty } => {
                // Look up local for this def_id
                Place::Local(LocalId(0)) // Placeholder
            }

            hir::Expr::Binary {
                op,
                left,
                right,
                ty,
            } => {
                let left_place = self.lower_expr(left);
                let right_place = self.lower_expr(right);

                let left_op = self.place_to_operand(left_place);
                let right_op = self.place_to_operand(right_place);

                let temp = self.add_local(ty.clone(), None);
                let place = Place::Local(temp);

                self.assign(
                    place.clone(),
                    Rvalue::BinaryOp(convert_binop(*op), Box::new(left_op), Box::new(right_op)),
                );

                place
            }

            hir::Expr::Call { func, args, ty } => {
                let func_place = self.lower_expr(func);
                let func_op = self.place_to_operand(func_place);

                let arg_ops: Vec<_> = args
                    .iter()
                    .map(|arg| {
                        let place = self.lower_expr(arg);
                        self.place_to_operand(place)
                    })
                    .collect();

                let temp = self.add_local(ty.clone(), None);
                let dest = Place::Local(temp);

                let after_call = self.new_block();

                self.terminator(Terminator::Call {
                    func: func_op,
                    args: arg_ops,
                    destination: dest.clone(),
                    target: Some(after_call),
                    cleanup: None,
                });

                self.set_current_block(after_call);
                dest
            }

            _ => unimplemented!("Expression lowering not fully implemented"),
        }
    }

    /// Convert place to operand
    fn place_to_operand(&self, place: Place) -> Operand {
        match &place {
            Place::Local(_) => Operand::Copy(place),
            _ => Operand::Copy(place),
        }
    }

    /// Build the function
    pub fn build(mut self) -> Function {
        // Ensure entry block is set
        if self.function.blocks.is_empty() {
            let entry = self.new_block();
            self.function.entry_block = entry;
        }

        self.function
    }
}

/// Convert HIR binary op to MIR binary op
fn convert_binop(op: hir::BinOp) -> BinOp {
    match op {
        hir::BinOp::Add => BinOp::Add,
        hir::BinOp::Sub => BinOp::Sub,
        hir::BinOp::Mul => BinOp::Mul,
        hir::BinOp::Div => BinOp::Div,
        hir::BinOp::Mod => BinOp::Rem,
        hir::BinOp::Eq => BinOp::Eq,
        hir::BinOp::Ne => BinOp::Ne,
        hir::BinOp::Lt => BinOp::Lt,
        hir::BinOp::Gt => BinOp::Gt,
        hir::BinOp::Le => BinOp::Le,
        hir::BinOp::Ge => BinOp::Ge,
        hir::BinOp::And => BinOp::BitAnd,
        hir::BinOp::Or => BinOp::BitOr,
    }
}
