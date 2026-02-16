/-
Complete binary protobuf serialization implementation
Implements protobuf wire format specification
-/

import Compiler.Proto.Messages

namespace Compiler.Proto.Binary

-- Varint encoding (variable-length integers)
def encodeVarint (n : UInt64) : ByteArray :=
  let rec go (acc : ByteArray) (remaining : UInt64) : ByteArray :=
    if remaining < 128 then
      acc.push remaining.toUInt8
    else
      let byte := (remaining &&& 0x7F) ||| 0x80
      go (acc.push byte.toUInt8) (remaining >>> 7)
  go ByteArray.empty n

-- ZigZag encoding for signed integers
def encodeZigZag (n : Int64) : UInt64 :=
  if n >= 0 then
    (2 * n.toUInt64)
  else
    ((-2 * n.toInt - 1)).toUInt64

-- Decode varint
def decodeVarint (data : ByteArray) (offset : Nat) : Option (UInt64 × Nat) :=
  let rec go (acc : UInt64) (shift : Nat) (pos : Nat) : Option (UInt64 × Nat) :=
    if pos >= data.size then
      none
    else
      let b := data.get! pos
      let value := acc ||| ((b.toUInt64 &&& 0x7F) <<< shift)
      if b.toUInt64 &&& 0x80 == 0 then
        some (value, pos + 1)
      else if shift >= 64 then
        none  -- Overflow
      else
        go value (shift + 7) (pos + 1)
  go 0 0 offset

-- Field tag encoding (field number + wire type)
inductive WireType
  | varint | i64 | len | sgroup | egroup | i32
  deriving Repr, BEq

def WireType.toUInt64 : WireType → UInt64
  | varint => 0 | i64 => 1 | len => 2 | sgroup => 3 | egroup => 4 | i32 => 5

-- Encode field tag
def encodeTag (fieldNum : Nat) (wireType : WireType) : ByteArray :=
  encodeVarint ((fieldNum.toUInt64 <<< 3) ||| wireType.toUInt64)

-- Fixed 32-bit encoding (little-endian)
def encodeFixed32 (n : UInt32) : ByteArray :=
  ByteArray.mk #[
    n.toUInt8,
    (n >>> 8).toUInt8,
    (n >>> 16).toUInt8,
    (n >>> 24).toUInt8
  ]

-- Fixed 64-bit encoding (little-endian)
def encodeFixed64 (n : UInt64) : ByteArray :=
  ByteArray.mk #[
    n.toUInt8, (n >>> 8).toUInt8, (n >>> 16).toUInt8, (n >>> 24).toUInt8,
    (n >>> 32).toUInt8, (n >>> 40).toUInt8, (n >>> 48).toUInt8, (n >>> 56).toUInt8
  ]

-- Boolean encoding
def encodeBool (b : Bool) : ByteArray :=
  encodeVarint (if b then 1 else 0)

-- String/bytes encoding (length-delimited)
def encodeString (s : String) : ByteArray :=
  let bytes := s.toUTF8
  encodeVarint bytes.size.toUInt64 ++ bytes

-- Enum encoding (as varint)
def encodeEnum (n : Nat) : ByteArray :=
  encodeVarint n.toUInt64

-- Serializer monad
structure SerializeState where
  buffer : ByteArray
  fieldNumber : Nat := 1

def Serializer := StateM SerializeState

def emit (bytes : ByteArray) : Serializer Unit :=
  modify (λ s => { s with buffer := s.buffer ++ bytes })

def withField (n : Nat) (action : Serializer Unit) : Serializer Unit :=
  modify (λ s => { s with fieldNumber := n }) *> action

def runSerializer (s : Serializer Unit) : ByteArray :=
  (s.run { buffer := ByteArray.empty }).1.buffer

-- Field encoding helpers
def encodeFieldVarint (fieldNum : Nat) (value : UInt64) : Serializer Unit :=
  emit (encodeTag fieldNum .varint)
  emit (encodeVarint value)

def encodeFieldFixed32 (fieldNum : Nat) (value : UInt32) : Serializer Unit :=
  emit (encodeTag fieldNum .i32)
  emit (encodeFixed32 value)

def encodeFieldFixed64 (fieldNum : Nat) (value : UInt64) : Serializer Unit :=
  emit (encodeTag fieldNum .i64)
  emit (encodeFixed64 value)

def encodeFieldBool (fieldNum : Nat) (value : Bool) : Serializer Unit :=
  emit (encodeTag fieldNum .varint)
  emit (encodeBool value)

def encodeFieldString (fieldNum : Nat) (value : String) : Serializer Unit :=
  emit (encodeTag fieldNum .len)
  emit (encodeString value)

def encodeFieldBytes (fieldNum : Nat) (value : ByteArray) : Serializer Unit :=
  emit (encodeTag fieldNum .len)
  emit (encodeVarint value.size.toUInt64)
  emit value

def encodeFieldMessage (fieldNum : Nat) (encode : Serializer Unit) : Serializer Unit :=
  emit (encodeTag fieldNum .len)
  let msgBytes := runSerializer encode
  emit (encodeVarint msgBytes.size.toUInt64)
  emit msgBytes

-- Deserializer
def Deserializer (α : Type) := ByteArray → Nat → Except String (α × Nat)

def fail (msg : String) {α : Type} : Deserializer α :=
  λ _ _ => throw msg

def pure (v : α) : Deserializer α :=
  λ _ pos => Except.ok (v, pos)

def bind {α β : Type} (d : Deserializer α) (f : α → Deserializer β) : Deserializer β :=
  λ data pos =>
    match d data pos with
    | Except.ok (v, newPos) => f v data newPos
    | Except.error e => Except.error e

instance : Monad Deserializer where
  pure := pure
  bind := bind

instance : Alternative Deserializer where
  failure := fail "Alternative failure"
  orElse d1 d2 := λ data pos =>
    match d1 data pos with
    | Except.ok r => Except.ok r
    | Except.error _ => d2 () data pos

-- Deserialization helpers
def readVarint : Deserializer UInt64 :=
  λ data pos =>
    match decodeVarint data pos with
    | some (v, newPos) => Except.ok (v, newPos)
    | none => Except.error "Failed to decode varint"

def readFixed32 : Deserializer UInt32 :=
  λ data pos =>
    if pos + 4 <= data.size then
      let n :=
        data.get! pos.toNat.toUSize |||
        (data.get! (pos + 1).toNat.toUSize <<< 8) |||
        (data.get! (pos + 2).toNat.toUSize <<< 16) |||
        (data.get! (pos + 3).toNat.toUSize <<< 24)
      Except.ok (n.toUInt32, pos + 4)
    else
      Except.error "Not enough bytes for fixed32"

def readFixed64 : Deserializer UInt64 :=
  λ data pos =>
    if pos + 8 <= data.size then
      let n :=
        data.get! pos.toNat.toUSize |||
        (data.get! (pos + 1).toNat.toUSize <<< 8) |||
        (data.get! (pos + 2).toNat.toUSize <<< 16) |||
        (data.get! (pos + 3).toNat.toUSize <<< 24) |||
        (data.get! (pos + 4).toNat.toUSize <<< 32) |||
        (data.get! (pos + 5).toNat.toUSize <<< 40) |||
        (data.get! (pos + 6).toNat.toUSize <<< 48) |||
        (data.get! (pos + 7).toNat.toUSize <<< 56)
      Except.ok (n, pos + 8)
    else
      Except.error "Not enough bytes for fixed64"

def readBool : Deserializer Bool := do
  let v ← readVarint
  return v != 0

def readString : Deserializer String := do
  let len ← readVarint
  let lenNat := len.toNat
  λ data pos =>
    if pos + lenNat <= data.size then
      let bytes := data.extract pos (pos + lenNat)
      match String.fromUTF8? bytes with
      | some s => Except.ok (s, pos + lenNat)
      | none => Except.error "Invalid UTF-8 string"
    else
      Except.error "Not enough bytes for string"

def readBytes : Deserializer ByteArray := do
  let len ← readVarint
  let lenNat := len.toNat
  λ data pos =>
    if pos + lenNat <= data.size then
      Except.ok (data.extract pos (pos + lenNat), pos + lenNat)
    else
      Except.error "Not enough bytes"

-- Skip unknown field
def skipField (wireType : WireType) : Deserializer Unit :=
  match wireType with
  | .varint => readVarint *> pure ()
  | .i64 => readFixed64 *> pure ()
  | .i32 => readFixed32 *> pure ()
  | .len => readBytes *> pure ()
  | _ => fail "Cannot skip group fields"

end Compiler.Proto.Binary
