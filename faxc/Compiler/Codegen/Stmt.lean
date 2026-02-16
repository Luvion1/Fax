import Compiler.AST
import Compiler.AST.Types
import Compiler.Codegen.Expr

namespace Compiler.Codegen.Stmt

open Compiler.AST
open Compiler.AST.Types

-- Statement code generation with context
def CodegenContext := List (String × String) -- Variable name -> LLVM register

partial def generate (s : Stmt) (ctx : CodegenContext) (counter : Nat) : String × CodegenContext × Nat :=
  match s with
  | .decl mut pat value => 
    -- Generate expression code
    let (exprCode, exprReg, newCtx, newCounter) := Expr.generate value ctx counter
    
    -- Extract variable name from pattern
    let varName := match pat with
    | .varPat name => name
    | _ => s!"tmp_{counter}"
    
    -- Allocate stack space and store value
    let allocReg := s!"%{newCounter}"
    let storeCode := s!"
  {allocReg} = alloca i32
  store i32 {exprReg}, i32* {allocReg}"
    
    let finalCtx := (varName, allocReg) :: newCtx
    (exprCode ++ storeCode, finalCtx, newCounter + 1)
  
  | .assign lhs rhs => 
    -- For simple variable assignment
    match lhs with
    | .var name => 
      match ctx.find? (fun (n, _) => n == name) with
      | some (_, ptrReg) => 
        let (rhsCode, rhsReg, ctx1, c1) := Expr.generate rhs ctx counter
        let storeCode := s!"
  store i32 {rhsReg}, i32* {ptrReg}"
        (rhsCode ++ storeCode, ctx1, c1)
      | none => 
        -- Variable not found, treat as global
        let (rhsCode, rhsReg, ctx1, c1) := Expr.generate rhs ctx counter
        let storeCode := s!"
  store i32 {rhsReg}, i32* @{name}"
        (rhsCode ++ storeCode, ctx1, c1)
    | _ => 
      -- Complex assignment (field access, etc.) - placeholder
      ("", ctx, counter)
  
  | .exprStmt e => 
    let (code, _, newCtx, newCounter) := Expr.generate e ctx counter
    (code, newCtx, newCounter)
  
  | .return e => 
    let (code, reg, newCtx, newCounter) := Expr.generate e ctx counter
    let retCode := s!"
  ret i32 {reg}"
    (code ++ retCode, newCtx, newCounter)
  
  | .break => 
    -- Will be handled with loop labels
    ("  br label %break", ctx, counter)
  
  | .continue => 
    -- Will be handled with loop labels
    ("  br label %continue", ctx, counter)

-- Generate code for a list of statements
def generateStmts (stmts : List Stmt) (ctx : CodegenContext) (counter : Nat) : String × CodegenContext × Nat :=
  stmts.foldl (fun (accCode, accCtx, accCounter) stmt =>
    let (code, newCtx, newCounter) := generate stmt accCtx accCounter
    (accCode ++ code, newCtx, newCounter)
  ) ("", ctx, counter)

-- Generate code for let binding (used in expressions)
def generateLet (pat : Pattern) (value : Expr) (body : Expr) (ctx : CodegenContext) (counter : Nat) : String × String × CodegenContext × Nat :=
  -- Generate value
  let (valCode, valReg, ctx1, c1) := Expr.generate value ctx counter
  
  -- Extract variable name
  let varName := match pat with
  | .varPat name => name
  | _ => s!"let_{counter}"
  
  -- Allocate and store
  let ptrReg := s!"%{c1}"
  let storeCode := s!"
  {ptrReg} = alloca i32
  store i32 {valReg}, i32* {ptrReg}"
  
  let ctx2 := (varName, ptrReg) :: ctx1
  
  -- Generate body with new context
  let (bodyCode, bodyReg, ctx3, c2) := Expr.generate body ctx2 (c1 + 1)
  
  (valCode ++ storeCode ++ bodyCode, bodyReg, ctx3, c2)

end Compiler.Codegen.Stmt
