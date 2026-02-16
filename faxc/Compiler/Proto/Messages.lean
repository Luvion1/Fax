/-
Protobuf message structures for Fax compiler
These structures mirror the .proto definitions and can be serialized/deserialized
-/

namespace Compiler.Proto

-- Source position information
structure SourcePos where
  filename : String
  line : Nat
  column : Nat
  offset : Nat
  deriving Repr

def SourcePos.default : SourcePos :=
  { filename := "", line := 0, column := 0, offset := 0 }

structure SourceRange where
  start : SourcePos
  «end» : SourcePos
  deriving Repr

def SourceRange.default : SourceRange :=
  { start := SourcePos.default, «end» := SourcePos.default }

-- Token types (mirror of TokenType in proto)
inductive TokenType
  | litInt | litFloat | litString | litChar | litBool
  | kwFn | kwLet | kwMut | kwIf | kwElse | kwMatch
  | kwStruct | kwEnum | kwReturn | kwWhile | kwLoop
  | kwBreak | kwContinue | kwPub | kwMod | kwUse
  | kwAs | kwTrue | kwFalse
  | opAdd | opSub | opMul | opDiv | opMod
  | opAnd | opOr | opNot | opEq | opNe
  | opLt | opLe | opGt | opGe
  | opAssign | opAddAssign | opSubAssign | opMulAssign
  | opDivAssign | opModAssign
  | opShl | opShr | opBitAnd | opBitOr | opBitXor | opBitNot
  | lparen | rparen | lbrace | rbrace | lbracket | rbracket
  | comma | colon | semicolon | dot | arrow | arrowFat
  | pipe | underscore | doubleColon
  | ident | eof
  deriving Repr, BEq

-- Token message
structure Token where
  type : TokenType
  text : String
  span : SourceRange
  deriving Repr

-- Token stream
structure TokenStream where
  tokens : List Token
  sourceFilename : String
  sourceContent : String
  deriving Repr

-- Primitive types
inductive PrimitiveType
  | unit | i8 | i16 | i32 | i64
  | u8 | u16 | u32 | u64
  | f32 | f64 | bool | char | string
  | inferred
  deriving Repr, BEq

-- Type representation
inductive Ty
  | primitive (p : PrimitiveType)
  | array (elem : Ty) (size : Nat)
  | tuple (elems : List Ty)
  | structTy (name : String) (fields : List (String × Ty))
  | enumTy (name : String) (variants : List (String × List Ty))
  | fun (args : List Ty) (ret : Ty)
  | named (name : String)
  deriving Repr

-- Literal values
inductive Literal
  | int (val : Int)
  | float (val : Float)
  | bool (val : Bool)
  | string (val : String)
  | char (val : Char)
  deriving Repr, BEq

-- Pattern matching
inductive Pattern
  | wild
  | lit (l : Literal)
  | var (name : String)
  | tuple (pats : List Pattern)
  | structPat (name : String) (fields : List (String × Pattern))
  | enumPat (enumName : String) (variant : String) (pats : List Pattern)
  deriving Repr

-- Unary operators
inductive UnaryOp | neg | not | bitnot
  deriving Repr, BEq

-- Binary operators
inductive BinOp
  | add | sub | mul | div | mod
  | and | or
  | eq | ne | lt | le | gt | ge
  | shl | shr | bitand | bitor | bitxor
  deriving Repr, BEq

-- Expressions
inductive Expr
  | lit (l : Literal)
  | var (name : String)
  | tuple (elems : List Expr)
  | structVal (name : String) (fields : List (String × Expr))
  | enumVal (enumName : String) (variant : String) (args : List Expr)
  | proj (e : Expr) (idx : Nat)
  | field (e : Expr) (fieldName : String)
  | unary (op : UnaryOp) (e : Expr)
  | binary (op : BinOp) (e1 e2 : Expr)
  | call (fnName : String) (args : List Expr)
  | exprIf (cond thenBranch elseBranch : Expr)
  | matchExpr (scrut : Expr) (cases : List (Pattern × Expr))
  | block (stmts : List Stmt) (trailing : Expr)
  | lambda (params : List (String × Ty)) (body : Expr)
  | letExpr (pat : Pattern) (value : Expr) (body : Expr)
  deriving Repr

-- Statements
inductive Stmt
  | decl (isMut : Bool) (pat : Pattern) (value : Expr)
  | assign (lhs rhs : Expr)
  | exprStmt (e : Expr)
  | return (e : Expr)
  | break
  | continue
  deriving Repr

-- Declarations
inductive Decl
  | funDecl (isPub : Bool) (name : String) (params : List (String × Ty)) (ret : Ty) (body : Expr)
  | structDecl (isPub : Bool) (name : String) (fields : List (String × Ty))
  | enumDecl (isPub : Bool) (name : String) (variants : List (String × List Ty))
  deriving Repr

-- Module
structure Module where
  name : String
  decls : List Decl
  deriving Repr

end Compiler.Proto
