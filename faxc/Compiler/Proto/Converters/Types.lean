/-
Converter between AST types and Proto types
-/

import Compiler.AST.Types
import Compiler.Proto.Messages

namespace Compiler.Proto.Converters

open AST.Types
open Messages

-- Convert AST.Ty to Proto.Ty
def AST.Ty.toProto : AST.Types.Ty → Messages.Ty
  | .unit => .primitive .unit
  | .int32 => .primitive .i32
  | .int64 => .primitive .i64
  | .float64 => .primitive .f64
  | .boolTy => .primitive .bool
  | .char => .primitive .char
  | .string => .primitive .string
  | .array elem size => .array elem.toProto size
  | .tuple elems => .tuple (elems.map AST.Ty.toProto)
  | .structTy name fields =>
    .structTy name (fields.map (λ (n, t) => (n, t.toProto)))
  | .enumTy name variants =>
    .enumTy name (variants.map (λ (n, ts) => (n, ts.map AST.Ty.toProto)))
  | .fun args ret =>
    .fun (args.map AST.Ty.toProto) ret.toProto
  | .inferred => .primitive .inferred

-- Convert Proto.Ty back to AST.Ty
def Ty.toAST : Messages.Ty → AST.Types.Ty
  | .primitive .unit => .unit
  | .primitive .i32 => .int32
  | .primitive .i64 => .int64
  | .primitive .f64 => .float64
  | .primitive .bool => .boolTy
  | .primitive .char => .char
  | .primitive .string => .string
  | .primitive .i8 => .int32 -- Map to closest
  | .primitive .i16 => .int32
  | .primitive .u8 => .int32
  | .primitive .u16 => .int32
  | .primitive .u32 => .int32
  | .primitive .u64 => .int64
  | .primitive .f32 => .float64
  | .primitive .inferred => .inferred
  | .array elem size => .array elem.toAST size
  | .tuple elems => .tuple (elems.map Ty.toAST)
  | .structTy name fields =>
    .structTy name (fields.map (λ (n, t) => (n, t.toAST)))
  | .enumTy name variants =>
    .enumTy name (variants.map (λ (n, ts) => (n, ts.map Ty.toAST)))
  | .fun args ret =>
    .fun (args.map Ty.toAST) ret.toAST
  | .named n => .structTy n []

-- Convert AST.Literal to Proto.Literal
def AST.Literal.toProto : AST.Types.Literal → Messages.Literal
  | .int val => .int val
  | .float val => .float val
  | .bool val => .bool val
  | .string val => .string val
  | .char val => .char val.val

-- Convert Proto.Literal back to AST.Literal
def Literal.toAST : Messages.Literal → AST.Types.Literal
  | .int val => .int val
  | .float val => .float val
  | .bool val => .bool val
  | .string val => .string val
  | .char val => .char (Char.ofNat val.toNat)

end Compiler.Proto.Converters
