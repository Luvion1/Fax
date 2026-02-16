/-
Protobuf codec for Type system
-/

import Compiler.Proto.Codec.Token
import Compiler.Proto.Messages

namespace Compiler.Proto.Codec.Types

open Binary Messages

-- PrimitiveType encoding
private def primitiveTypeToNat : PrimitiveType → Nat
  | .unit => 0 | .i8 => 1 | .i16 => 2 | .i32 => 3 | .i64 => 4
  | .u8 => 5 | .u16 => 6 | .u32 => 7 | .u64 => 8
  | .f32 => 9 | .f64 => 10 | .bool => 11 | .char => 12
  | .string => 13 | .inferred => 255

private def natToPrimitiveType : Nat → Option PrimitiveType
  | 0 => some .unit | 1 => some .i8 | 2 => some .i16 | 3 => some .i32
  | 4 => some .i64 | 5 => some .u8 | 6 => some .u16 | 7 => some .u32
  | 8 => some .u64 | 9 => some .f32 | 10 => some .f64 | 11 => some .bool
  | 12 => some .char | 13 => some .string | 255 => some .inferred
  | _ => none

-- Forward declarations for recursive types
partial def encodeTy : Ty → Serializer Unit
partial def decodeTy : Deserializer Ty

-- Ty encoding (using oneof pattern)
partial def encodeTy (ty : Ty) : Serializer Unit :=
  match ty with
  | .primitive p =>
    encodeFieldVarint 1 (primitiveTypeToNat p).toUInt64
  | .array elem size =>
    encodeFieldMessage 2 (do
      encodeFieldMessage 1 (encodeTy elem)
      encodeFieldVarint 2 size.toUInt64)
  | .tuple elems =>
    encodeFieldMessage 3 (
      elems.forM (λ elem => encodeFieldMessage 1 (encodeTy elem)))
  | .structTy name fields =>
    encodeFieldMessage 4 (do
      encodeFieldString 1 name
      fields.forM (λ (n, t) =>
        encodeFieldMessage 2 (do
          encodeFieldString 1 n
          encodeFieldMessage 2 (encodeTy t))))
  | .enumTy name variants =>
    encodeFieldMessage 5 (do
      encodeFieldString 1 name
      variants.forM (λ (n, ts) =>
        encodeFieldMessage 2 (do
          encodeFieldString 1 n
          ts.forM (λ t => encodeFieldMessage 2 (encodeTy t)))))
  | .fun args ret =>
    encodeFieldMessage 6 (do
      args.forM (λ arg => encodeFieldMessage 1 (encodeTy arg))
      encodeFieldMessage 2 (encodeTy ret))
  | .named name =>
    encodeFieldString 7 name

-- Simplified Ty decoder (would need proper oneof handling)
partial def decodeTy : Deserializer Ty := do
  -- Simplified implementation
  return .primitive .inferred

-- Literal encoding/decoding
def encodeLiteral (lit : Literal) : Serializer Unit :=
  match lit with
  | .int v => encodeFieldVarint 1 (if v >= 0 then v.toUInt64 else (encodeZigZag v).toUInt64)
  | .float v => encodeFieldFixed64 2 v.toUInt64
  | .bool v => encodeFieldBool 3 v
  | .string v => encodeFieldString 4 v
  | .char v => encodeFieldVarint 5 v.val.toUInt64

def decodeLiteral : Deserializer Literal := do
  -- Simplified
  return .int 0

-- Param encoding/decoding
def encodeParam (p : Param) : Serializer Unit := do
  encodeFieldString 1 p.name
  encodeFieldMessage 2 (encodeTy p.paramType)

def decodeParam : Deserializer Param := do
  return { name := "", paramType := .primitive .inferred }

-- Field encoding/decoding
def encodeField (f : Field) : Serializer Unit := do
  encodeFieldString 1 f.name
  encodeFieldMessage 2 (encodeTy f.fieldType)

def decodeField : Deserializer Field := do
  return { name := "", fieldType := .primitive .inferred }

-- Variant encoding/decoding
def encodeVariant (v : Variant) : Serializer Unit := do
  encodeFieldString 1 v.name
  v.payloadTypes.forM (λ t => encodeFieldMessage 2 (encodeTy t))

def decodeVariant : Deserializer Variant := do
  return { name := "", payloadTypes := [] }

end Compiler.Proto.Codec.Types
