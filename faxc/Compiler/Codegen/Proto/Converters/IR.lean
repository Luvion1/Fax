/-
Codegen IR Protobuf Converters
Converts between Lean Codegen IR types and Protobuf message types
-/

import Compiler.Codegen.Types
import Compiler.Proto.Messages

namespace Compiler.Codegen.Proto.Converters.IR

open Compiler.Codegen.Types
open Compiler.Proto.Messages

-- Convert Lean Codegen Type to Proto Ty
def CodegenType.toProto (ty : CodegenType) : Ty :=
  match ty with
  | .void => .primitive .unit
  | .i1 => .primitive .bool
  | .i8 => .primitive .i8
  | .i16 => .primitive .i16
  | .i32 => .primitive .i32
  | .i64 => .primitive .i64
  | .f32 => .primitive .f32
  | .f64 => .primitive .f64
  | .ptr ty => .primitive .string -- Simplified
  | .array ty n => .array (CodegenType.toProto ty) n
  | .struct name fields => .structTy name (fields.map (fun (n, t) => (n, CodegenType.toProto t)))
  | .function ret args => .fun (args.map CodegenType.toProto) (CodegenType.toProto ret)

-- Convert Proto Ty to Lean Codegen Type
def Ty.toCodegenType (ty : Ty) : CodegenType :=
  match ty with
  | .primitive .unit => .void
  | .primitive .bool => .i1
  | .primitive .i8 => .i8
  | .primitive .i16 => .i16
  | .primitive .i32 => .i32
  | .primitive .i64 => .i64
  | .primitive .f32 => .f32
  | .primitive .f64 => .f64
  | .primitive .string => .ptr .i8
  | .primitive .char => .i8
  | .primitive .inferred => .i32
  | .array elem n => .array (Ty.toCodegenType elem) n
  | .structTy name fields => .struct name (fields.map (fun (n, t) => (n, Ty.toCodegenType t)))
  | .fun args ret => .function (Ty.toCodegenType ret) (args.map Ty.toCodegenType)
  | .tuple elems => .struct "tuple" (elems.mapIdx (fun i t => (s!"_{i}", Ty.toCodegenType t)))
  | .named n => .struct n []
  | .enumTy name _ => .i32 -- Enums as i32

-- Convert LLVMValue to Proto Value representation
def LLVMValue.toProto (val : LLVMValue) : Expr :=
  match val with
  | .constInt n => .lit (.int n)
  | .constFloat f => .lit (.float f)
  | .constBool b => .lit (.bool b)
  | .constString s => .lit (.string s)
  | .register name => .var name
  | .global name => .var (s!"@{name}")
  | .null => .lit (.int 0)

-- Convert LLVMInstruction to Proto Stmt representation  
def LLVMInstruction.toProto (inst : LLVMInstruction) : Stmt :=
  match inst with
  | .alloca ty => .exprStmt (.lit (.int 0)) -- Placeholder
  | .store val ptr => .assign (.var ptr) (LLVMValue.toProto (.register val))
  | .load ptr => .exprStmt (.var ptr)
  | .add ty lhs rhs => .exprStmt (.binary .add (.var lhs) (.var rhs))
  | .sub ty lhs rhs => .exprStmt (.binary .sub (.var lhs) (.var rhs))
  | .mul ty lhs rhs => .exprStmt (.binary .mul (.var lhs) (.var rhs))
  | .sdiv ty lhs rhs => .exprStmt (.binary .div (.var lhs) (.var rhs))
  | .icmp cond ty lhs rhs => 
    let op := match cond with
    | .eq => BinaryOp.eq
    | .ne => BinaryOp.ne
    | .slt => BinaryOp.lt
    | .sle => BinaryOp.le
    | .sgt => BinaryOp.gt
    | .sge => BinaryOp.ge
    .exprStmt (.binary op (.var lhs) (.var rhs))
  | .call retTy name args => 
    .exprStmt (.call name (args.map (fun _ => .lit (.int 0)))) -- Simplified
  | .ret val => .return (LLVMValue.toProto val)
  | .br label => .exprStmt (.lit (.int 0)) -- Placeholder
  | .brCond cond tLabel fLabel => .exprStmt (.lit (.int 0)) -- Placeholder
  | .phi ty vals => .exprStmt (.lit (.int 0)) -- Placeholder
  | .getelementptr ty ptr indices => .exprStmt (.lit (.int 0)) -- Placeholder

-- Convert LLVMFunction to Proto Decl
def LLVMFunction.toProto (func : LLVMFunction) : Decl :=
  .func func.name 
    (func.params.map (fun (n, t) => (n, CodegenType.toProto t)))
    (CodegenType.toProto func.retType)
    (.block func.body.map LLVMInstruction.toProto (.lit (.int 0)))

-- Convert LLVMModule to Proto Module
def LLVMModule.toProto (mod : LLVMModule) : Module :=
  { decls := mod.functions.map LLVMFunction.toProto }

-- Convert Proto Module to Lean LLVMModule (partial)
def Module.toLLVMModule (mod : Module) : LLVMModule :=
  { 
    name := "module",
    functions := mod.decls.filterMap (fun d =>
      match d with
      | .func name params ret body => some {
          name := name,
          params := params.map (fun (n, t) => (n, Ty.toCodegenType t)),
          retType := Ty.toCodegenType ret,
          body := [] -- Body reconstruction would need full parsing
        }
      | _ => none
    )
  }

end Compiler.Codegen.Proto.Converters.IR
