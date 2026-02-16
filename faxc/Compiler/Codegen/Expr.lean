import Compiler.AST
import Compiler.AST.Types
import Compiler.Codegen.Types

namespace Compiler.Codegen.Expr

open Compiler.AST
open Compiler.AST.Types

-- Code generation state
def CodegenContext := List (String × String) -- Variable name -> LLVM register

def freshVar (ctx : CodegenContext) (base : String) : String × CodegenContext :=
  let counter := ctx.length
  let varName := s!"{base}_{counter}"
  (varName, ctx)

-- Counter for unique string literal names
private def stringLiteralCounter : IO.Ref Nat := unsafePerformIO (IO.mkRef 0)

private def unsafePerformIO {α} (io : IO α) : α :=
  match io.run {} with
  | EStateM.Result.ok a _ => a
  | _ => panic! "unsafePerformIO failed"

-- Generate unique name for string literal
private def freshStringLiteralName (str : String) : String :=
  let counter := stringLiteralCounter.modifyGet (fun n => (n, n + 1))
  s!".str.{counter}_{str.length}"

-- Generate LLVM IR for literals
def emitLiteral (lit : Lit) : String × String :=
  match lit with
  | .intLit val => 
    let reg := s!"i32 {val}"
    ("", reg)
  | .floatLit val => 
    let reg := s!"double {val}"
    ("", reg)
  | .boolLit true => 
    let reg := s!"i1 1"
    ("", reg)
  | .boolLit false => 
    let reg := s!"i1 0"
    ("", reg)
  | .stringLit str => 
    -- String literals need global constant with unique name
    let constName := freshStringLiteralName str
    let strLen := str.length + 1
    -- Escape special characters in string
    let escapedStr := String.join (str.toList.map (fun c =>
      if c == '"' then "\\\""
      else if c == '\\' then "\\\\"
      else if c == '\n' then "\\0A"
      else if c == '\r' then "\\0D"
      else if c == '\t' then "\\09"
      else if c.toNat < 32 then s!"\\{c.toNat.toUInt8.toHex}"
      else String.singleton c
    ))
    let code := s!"{constName} = private constant [{strLen} x i8] c\"{escapedStr}\\00\""
    let reg := s!"i8* getelementptr ([{strLen} x i8], [{strLen} x i8]* {constName}, i64 0, i64 0)"
    (code, reg)
  | .charLit c => 
    let reg := s!"i8 {c.toNat}"
    ("", reg)
  | .floatLit val => 
    let reg := s!"double {val}"
    ("", reg)
  | .boolLit true => 
    let reg := s!"i1 1"
    ("", reg)
  | .boolLit false => 
    let reg := s!"i1 0"
    ("", reg)
  | .stringLit str => 
    -- String literals need global constant
    let constName := s!".str_{str.length}"
    let strLen := str.length + 1
    let code := s!"{constName} = private constant [{strLen} x i8] c\"{str}\\00\""
    let reg := s!"i8* getelementptr ([{strLen} x i8], [{strLen} x i8]* {constName}, i64 0, i64 0)"
    (code, reg)
  | .charLit c => 
    let reg := s!"i8 {c.toNat}"
    ("", reg)

-- Generate LLVM IR for unary operators
def emitUnaryOp (op : UnaryOp) (operandReg : String) (resultReg : String) : String :=
  match op with
  | .neg => 
    s!"  {resultReg} = sub i32 0, {operandReg}"
  | .not => 
    s!"  {resultReg} = xor i1 {operandReg}, 1"
  | .bitnot => 
    s!"  {resultReg} = xor i32 {operandReg}, -1"

-- Generate LLVM IR for binary operators
def emitBinaryOp (op : BinaryOp) (ty : String) (leftReg : String) (rightReg : String) (resultReg : String) : String :=
  match op with
  | .add => s!"  {resultReg} = add {ty} {leftReg}, {rightReg}"
  | .sub => s!"  {resultReg} = sub {ty} {leftReg}, {rightReg}"
  | .mul => s!"  {resultReg} = mul {ty} {leftReg}, {rightReg}"
  | .div => s!"  {resultReg} = sdiv {ty} {leftReg}, {rightReg}"
  | .mod => s!"  {resultReg} = srem {ty} {leftReg}, {rightReg}"
  | .and => s!"  {resultReg} = and i1 {leftReg}, {rightReg}"
  | .or => s!"  {resultReg} = or i1 {leftReg}, {rightReg}"
  | .eq => 
    if ty == "i1" then
      s!"  {resultReg} = icmp eq i1 {leftReg}, {rightReg}"
    else
      s!"  {resultReg} = icmp eq {ty} {leftReg}, {rightReg}"
  | .ne => 
    if ty == "i1" then
      s!"  {resultReg} = icmp ne i1 {leftReg}, {rightReg}"
    else
      s!"  {resultReg} = icmp ne {ty} {leftReg}, {rightReg}"
  | .lt => s!"  {resultReg} = icmp slt {ty} {leftReg}, {rightReg}"
  | .le => s!"  {resultReg} = icmp sle {ty} {leftReg}, {rightReg}"
  | .gt => s!"  {resultReg} = icmp sgt {ty} {leftReg}, {rightReg}"
  | .ge => s!"  {resultReg} = icmp sge {ty} {leftReg}, {rightReg}"
  | .shl => s!"  {resultReg} = shl {ty} {leftReg}, {rightReg}"
  | .shr => s!"  {resultReg} = ashr {ty} {leftReg}, {rightReg}"
  | .band => s!"  {resultReg} = and {ty} {leftReg}, {rightReg}"
  | .bor => s!"  {resultReg} = or {ty} {leftReg}, {rightReg}"
  | .bxor => s!"  {resultReg} = xor {ty} {leftReg}, {rightReg}"

-- Get type string for expression
def exprType (e : Expr) : String :=
  match e with
  | .lit (.intLit _) => "i32"
  | .lit (.floatLit _) => "double"
  | .lit (.boolLit _) => "i1"
  | .lit (.charLit _) => "i8"
  | .lit (.stringLit _) => "i8*"
  | _ => "i32" -- Default for now

-- Recursive expression code generation
partial def generate (e : Expr) (ctx : CodegenContext) (counter : Nat) : String × String × CodegenContext × Nat :=
  match e with
  | .lit lit => 
    let (code, reg) := emitLiteral lit
    (code, reg, ctx, counter)
  
  | .var name => 
    -- Look up variable in context
    match ctx.find? (fun (n, _) => n == name) with
    | some (_, reg) => ("", reg, ctx, counter)
    | none => ("", s!"@{name}", ctx, counter) -- Assume it's a global/function
  
  | .unary op operand => 
    let (code1, reg1, ctx1, c1) := generate operand ctx (counter + 1)
    let resultReg := s!"%{c1}"
    let code2 := emitUnaryOp op reg1 resultReg
    (code1 ++ "\n" ++ code2, resultReg, ctx1, c1 + 1)
  
  | .binary op left right => 
    let ty := exprType left
    let (code1, reg1, ctx1, c1) := generate left ctx (counter + 1)
    let (code2, reg2, ctx2, c2) := generate right ctx1 (c1 + 1)
    let resultReg := s!"%{c2}"
    let code3 := emitBinaryOp op ty reg1 reg2 resultReg
    (code1 ++ "\n" ++ code2 ++ "\n" ++ code3, resultReg, ctx2, c2 + 1)
  
  | .call fnName args => 
    -- Generate code for each argument
    let (argsCode, argsRegs, finalCtx, finalCounter) := 
      args.foldl (fun (accCode, accRegs, accCtx, accCounter) arg =>
        let (c, r, newCtx, newCounter) := generate arg accCtx accCounter
        (accCode ++ c, accRegs ++ [(exprType arg, r)], newCtx, newCounter)
      ) ("", [], ctx, counter)
    
    -- Build call instruction
    let resultReg := s!"%{finalCounter}"
    let argsStr := argsRegs.map (fun (ty, reg) => s!"{ty} {reg}") |> String.intercalate ", "
    let callCode := s!"  {resultReg} = call i32 @{fnName}({argsStr})"
    (argsCode ++ "\n" ++ callCode, resultReg, finalCtx, finalCounter + 1)
  
  | .exprIf cond thenExpr elseExpr => 
    let (code1, condReg, ctx1, c1) := generate cond ctx (counter + 1)
    let (code2, thenReg, ctx2, c2) := generate thenExpr ctx1 (c1 + 1)
    let (code3, elseReg, ctx3, c3) := generate elseExpr ctx2 (c2 + 1)
    
    let ty := exprType thenExpr
    let resultReg := s!"%{c3}"
    let thenLabel := s!"if_then_{c3}"
    let elseLabel := s!"if_else_{c3}"
    let endLabel := s!"if_end_{c3}"
    
    let branchCode := s!"
  br i1 {condReg}, label %{thenLabel}, label %{elseLabel}

{thenLabel}:
{code2}
  br label %{endLabel}

{elseLabel}:
{code3}
  br label %{endLabel}

{endLabel}:
  {resultReg} = phi {ty} [{thenReg}, %{thenLabel}], [{elseReg}, %{elseLabel}]"
    
    (code1 ++ branchCode, resultReg, ctx3, c3 + 1)
  
  | .block stmts expr => 
    -- For now, just generate the final expression
    -- Statements will be handled separately
    let (code, reg, ctx', c') := generate expr ctx (counter + 1)
    (code, reg, ctx', c')
  
  | _ => 
    -- Placeholder for unimplemented expressions
    ("", "i32 0", ctx, counter)

-- Entry point for expression code generation
def generateExpr (e : Expr) : String × String :=
  let (code, reg, _, _) := generate e [] 1
  (code, reg)

end Compiler.Codegen.Expr
