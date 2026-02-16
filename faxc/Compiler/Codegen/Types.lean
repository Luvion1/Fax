import Compiler.AST

namespace Compiler.Codegen.Types

open Compiler.AST

def toLLVMType (ty : Type) : String :=
  match ty with
  | .unit => "void"
  | .int32 => "i32"
  | .int64 => "i64"
  | .float64 => "double"
  | .boolTy => "i1"
  | .char => "i8"
  | .string => "i8*"
  | .array elem size => s!"[{size} x {toLLVMType elem}]"
  | .tuple elems => s!"{{{elems.map toLLVMType |> String.intercalate ", "}}}"
  | .structTy name _ => s!"%{name}"
  | .enumTy name _ => s!"%{name}"
  | .fun args ret => s!"{toLLVMType ret} ({args.map toLLVMType |> String.intercalate ", "})*"
  | .inferred => "i32"

end Compiler.Codegen.Types
