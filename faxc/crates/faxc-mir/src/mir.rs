use faxc_sem::Type;
use faxc_util::{Idx, IndexVec, Span, Symbol, DefId};

/// MIR Function
#[derive(Clone)]
pub struct Function {
    pub name: Symbol,
    pub locals: IndexVec<LocalId, Local>,
    pub blocks: IndexVec<BlockId, BasicBlock>,
    pub entry_block: BlockId,
    pub return_ty: Type,
}

/// Local variable
#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    pub ty: Type,
    pub span: Span,
    pub name: Option<Symbol>,
}

/// Local ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

impl Idx for LocalId {
    fn from_usize(idx: usize) -> Self { LocalId(idx as u32) }
    fn index(self) -> usize { self.0 as usize }
}

/// Block ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl Idx for BlockId {
    fn from_usize(idx: usize) -> Self { BlockId(idx as u32) }
    fn index(self) -> usize { self.0 as usize }
}

/// Basic Block
#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub id: BlockId,
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

/// Statement
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assign(Place, Rvalue),
    StorageLive(LocalId),
    StorageDead(LocalId),
    Nop,
}

/// Place - a memory location
#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    Local(LocalId),
    Projection(Box<Place>, Projection),
}

/// Projection onto a place
#[derive(Debug, Clone, PartialEq)]
pub enum Projection {
    Field(u32),
    Index(LocalId),
    ConstantIndex { offset: u64, min_length: u64, from_end: bool },
    Deref,
    Subslice { from: u64, to: u64, from_end: bool },
}

/// Rvalue
#[derive(Debug, Clone, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    Ref(Place, Mutability),
    AddressOf(Place, Mutability),
    UnaryOp(UnOp, Operand),
    BinaryOp(BinOp, Box<Operand>, Box<Operand>),
    CheckedBinaryOp(BinOp, Box<Operand>, Box<Operand>),
    NullaryOp(NullOp, Type),
    Cast(CastKind, Operand, Type),
    Discriminant(Place),
    Aggregate(AggregateKind, Vec<Operand>),
}

/// Operand
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
}

/// Constant value
#[derive(Debug, Clone, PartialEq)]
pub struct Constant {
    pub ty: Type,
    pub kind: ConstantKind,
}

/// Kind of constant
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantKind {
    Int(i64),
    Float(f64),
    String(Symbol),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mutability { Mutable, Immutable }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp { Neg, Not }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, Ne, Lt, Le, Gt, Ge,
    BitAnd, BitOr, BitXor,
    Shl, Shr, Offset,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NullOp { SizeOf, AlignOf }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CastKind {
    IntToInt, IntToFloat, FloatToInt, FloatToFloat,
    PtrToPtr, PtrToInt, IntToPtr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateKind {
    Tuple,
    Array(Type),
    Struct(DefId),
    Closure(DefId),
}

/// Terminator
#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    Goto { target: BlockId },
    If { cond: Operand, then_block: BlockId, else_block: BlockId },
    SwitchInt {
        discr: Operand,
        switch_ty: Type,
        targets: Vec<(u128, BlockId)>,
        otherwise: BlockId,
    },
    Return,
    Unreachable,
    Call {
        func: Operand,
        args: Vec<Operand>,
        destination: Place,
        target: Option<BlockId>,
        cleanup: Option<BlockId>,
    },
    Resume,
    Abort,
}
