/-
Code Generation Module - Main Entry Point
Exports all code generation components
-/

import Compiler.AST
import Compiler.Codegen.Types
import Compiler.Codegen.IR
import Compiler.Codegen.Expr
import Compiler.Codegen.Stmt
import Compiler.Codegen.Proto

namespace Compiler.Codegen

-- Re-export all codegen components
export Types (CodegenContext CodegenError)
export IR (buildModule buildFunction buildDecl)
export Expr (codegenExpr)
export Stmt (codegenStmt)

-- Main code generation function
def generateIR (m : AST.Module) : String :=
  IR.buildModule m.decls

def Module.toIR (m : AST.Module) : String :=
  generateIR m

end Compiler.Codegen
