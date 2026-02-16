/-
Protobuf serialization/deserialization for Fax compiler messages
Simple binary wire format implementation
-/

import Compiler.Proto.Messages

namespace Compiler.Proto.Wire

-- Wire types (protobuf wire format)
inductive WireType
  | varint    -- 0: int32, int64, uint32, uint64, sint32, sint64, bool, enum
  | i64       -- 1: fixed64, sfixed64, double
  | len       -- 2: string, bytes, embedded messages, packed repeated fields
  | startGroup -- 3: groups (deprecated)
  | endGroup  -- 4: groups (deprecated)
  | i32       -- 5: fixed32, sfixed32, float
  deriving Repr

def WireType.toUInt64 : WireType → UInt64
  | varint => 0
  | i64 => 1
  | len => 2
  | startGroup => 3
  | endGroup => 4
  | i32 => 5

-- Field key (field number + wire type)
def makeFieldKey (fieldNum : Nat) (wireType : WireType) : UInt64 :=
  (fieldNum.toUInt64 <<< 3) ||| wireType.toUInt64

-- Encode varint (variable length integer)
partial def encodeVarint (n : UInt64) : ByteArray :=
  if n < 128 then
    ByteArray.mk #[n.toUInt8]
  else
    let b := (n &&& 0x7F) ||| 0x80
    let rest := n >>> 7
    (encodeVarint rest).push b.toUInt8

-- Decode varint
partial def decodeVarint (data : ByteArray) (offset : Nat) : Option (UInt64 × Nat) :=
  go 0 offset 0
where
  go (acc : UInt64) (pos : Nat) (shift : Nat) : Option (UInt64 × Nat) :=
    if pos >= data.size then
      none
    else
      let b := data.get! pos
      let value := acc ||| ((b.toUInt64 &&& 0x7F) <<< shift)
      if b.toUInt64 &&& 0x80 == 0 then
        some (value, pos + 1)
      else
        go value (pos + 1) (shift + 7)

-- Encode field tag
def encodeTag (fieldNum : Nat) (wireType : WireType) : ByteArray :=
  encodeVarint (makeFieldKey fieldNum wireType)

-- Encode string/bytes
def encodeString (s : String) : ByteArray :=
  let bytes := s.toUTF8
  encodeVarint bytes.size.toUInt64 ++ bytes

-- Encode uint32/fixed32
def encodeUInt32 (n : UInt32) : ByteArray :=
  ByteArray.mk #[
    (n >>> 0).toUInt8,
    (n >>> 8).toUInt8,
    (n >>> 16).toUInt8,
    (n >>> 24).toUInt8
  ]

-- Encode uint64/fixed64
def encodeUInt64 (n : UInt64) : ByteArray :=
  ByteArray.mk #[
    (n >>> 0).toUInt8, (n >>> 8).toUInt8, (n >>> 16).toUInt8, (n >>> 24).toUInt8,
    (n >>> 32).toUInt8, (n >>> 40).toUInt8, (n >>> 48).toUInt8, (n >>> 56).toUInt8
  ]

-- Encode bool
def encodeBool (b : Bool) : ByteArray :=
  encodeVarint (if b then 1 else 0)

-- Encode enum (as varint)
def encodeEnum (n : Nat) : ByteArray :=
  encodeVarint n.toUInt64

-- Serializer type
abbrev Serializer := StateM ByteArray

def emit (bytes : ByteArray) : Serializer Unit :=
  modify (λ acc => acc ++ bytes)

def runSerializer (s : Serializer Unit) : ByteArray :=
  (s.run ByteArray.empty).1

-- Deserialize result
def Deserialize (α : Type) := ByteArray → Nat → Option (α × Nat)

instance : Functor Deserialize where
  map f d := λ data pos =>
    match d data pos with
    | some (v, newPos) => some (f v, newPos)
    | none => none

instance : Applicative Deserialize where
  pure v := λ _ pos => some (v, pos)
  seq f x := λ data pos =>
    match f data pos with
    | some (f', newPos) => (x () data newPos).map (λ (v, p) => (f' v, p))
    | none => none

instance : Monad Deserialize where
  bind d f := λ data pos =>
    match d data pos with
    | some (v, newPos) => f v data newPos
    | none => none

end Compiler.Proto.Wire
