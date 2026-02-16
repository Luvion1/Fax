/-
Protobuf encoder/decoder for Token types
-/

import Compiler.Proto.Binary
import Compiler.Proto.Messages

namespace Compiler.Proto.Codec.Token

open Binary Messages

-- TokenType encoding
private def tokenTypeToNat : TokenType → Nat
  | .litInt => 0 | .litFloat => 1 | .litString => 2 | .litChar => 3 | .litBool => 4
  | .kwFn => 10 | .kwLet => 11 | .kwMut => 12 | .kwIf => 13 | .kwElse => 14
  | .kwMatch => 15 | .kwStruct => 16 | .kwEnum => 17 | .kwReturn => 18
  | .kwWhile => 19 | .kwLoop => 20 | .kwBreak => 21 | .kwContinue => 22
  | .kwPub => 23 | .kwMod => 24 | .kwUse => 25 | .kwAs => 26 | .kwTrue => 27
  | .kwFalse => 28
  | .opAdd => 50 | .opSub => 51 | .opMul => 52 | .opDiv => 53 | .opMod => 54
  | .opAnd => 55 | .opOr => 56 | .opNot => 57 | .opEq => 58 | .opNe => 59
  | .opLt => 60 | .opLe => 61 | .opGt => 62 | .opGe => 63 | .opAssign => 64
  | .opAddAssign => 65 | .opSubAssign => 66 | .opMulAssign => 67 | .opDivAssign => 68
  | .opModAssign => 69 | .opShl => 70 | .opShr => 71 | .opBitAnd => 72
  | .opBitOr => 73 | .opBitXor => 74 | .opBitNot => 75
  | .lparen => 100 | .rparen => 101 | .lbrace => 102 | .rbrace => 103
  | .lbracket => 104 | .rbracket => 105 | .comma => 106 | .colon => 107
  | .semicolon => 108 | .dot => 109 | .arrow => 110 | .arrowFat => 111
  | .pipe => 112 | .underscore => 113 | .doubleColon => 114
  | .ident => 200 | .eof => 255

private def natToTokenType : Nat → Option TokenType
  | 0 => some .litInt | 1 => some .litFloat | 2 => some .litString
  | 3 => some .litChar | 4 => some .litBool
  | 10 => some .kwFn | 11 => some .kwLet | 12 => some .kwMut
  | 13 => some .kwIf | 14 => some .kwElse | 15 => some .kwMatch
  | 16 => some .kwStruct | 17 => some .kwEnum | 18 => some .kwReturn
  | 19 => some .kwWhile | 20 => some .kwLoop | 21 => some .kwBreak
  | 22 => some .kwContinue | 23 => some .kwPub | 24 => some .kwMod
  | 25 => some .kwUse | 26 => some .kwAs | 27 => some .kwTrue
  | 28 => some .kwFalse
  | 50 => some .opAdd | 51 => some .opSub | 52 => some .opMul
  | 53 => some .opDiv | 54 => some .opMod | 55 => some .opAnd
  | 56 => some .opOr | 57 => some .opNot | 58 => some .opEq
  | 59 => some .opNe | 60 => some .opLt | 61 => some .opLe
  | 62 => some .opGt | 63 => some .opGe | 64 => some .opAssign
  | 65 => some .opAddAssign | 66 => some .opSubAssign
  | 67 => some .opMulAssign | 68 => some .opDivAssign
  | 69 => some .opModAssign | 70 => some .opShl | 71 => some .opShr
  | 72 => some .opBitAnd | 73 => some .opBitOr | 74 => some .opBitXor
  | 75 => some .opBitNot
  | 100 => some .lparen | 101 => some .rparen | 102 => some .lbrace
  | 103 => some .rbrace | 104 => some .lbracket | 105 => some .rbracket
  | 106 => some .comma | 107 => some .colon | 108 => some .semicolon
  | 109 => some .dot | 110 => some .arrow | 111 => some .arrowFat
  | 112 => some .pipe | 113 => some .underscore | 114 => some .doubleColon
  | 200 => some .ident | 255 => some .eof
  | _ => none

-- SourcePos encoding/decoding
def encodeSourcePos (pos : SourcePos) : Serializer Unit := do
  encodeFieldString 1 pos.filename
  encodeFieldVarint 2 pos.line.toUInt64
  encodeFieldVarint 3 pos.column.toUInt64
  encodeFieldVarint 4 pos.offset.toUInt64

def decodeSourcePos : Deserializer SourcePos := do
  let mut filename := ""
  let mut line : Nat := 0
  let mut column : Nat := 0
  let mut offset : Nat := 0
  
  let rec loop : Deserializer Unit := do
    if (← λ data pos => if pos >= data.size then Except.ok ((), pos) else Except.error "continue") then
      return ()
    
    let tag ← readVarint
    let fieldNum := (tag >>> 3).toNat
    let wireTypeNum := tag &&& 0x7
    
    match wireTypeNum with
    | 0 =>  -- varint
      let v ← readVarint
      match fieldNum with
      | 1 => filename := ""
      | 2 => line := v.toNat
      | 3 => column := v.toNat
      | 4 => offset := v.toNat
      | _ => pure ()
      loop
    | 2 =>  -- length-delimited
      let bytes ← readBytes
      if fieldNum == 1 then
        match String.fromUTF8? bytes with
        | some s => filename := s
        | none => pure ()
      loop
    | _ => skipField (match wireTypeNum with | 0 => .varint | 1 => .i64 | 5 => .i32 | _ => .len) *> loop
  
  loop
  return { filename, line, column, offset }

-- SourceRange encoding/decoding
def encodeSourceRange (range : SourceRange) : Serializer Unit := do
  encodeFieldMessage 1 (encodeSourcePos range.start)
  encodeFieldMessage 2 (encodeSourcePos range.end)

def decodeSourceRange : Deserializer SourceRange := do
  let mut startPos := SourcePos.default
  let mut endPos := SourcePos.default
  -- Simplified - actual implementation would parse fields
  return { start := startPos, «end» := endPos }

-- Token encoding/decoding
def encodeToken (tok : Token) : Serializer Unit := do
  encodeFieldVarint 1 (tokenTypeToNat tok.type).toUInt64
  encodeFieldString 2 tok.text
  encodeFieldMessage 3 (encodeSourceRange tok.span)

def decodeToken : Deserializer Token := do
  let mut typeVal := TokenType.ident
  let mut textVal := ""
  let mut spanVal := SourceRange.default
  
  -- Simplified decoder
  return { type := typeVal, text := textVal, span := spanVal }

-- TokenStream encoding/decoding
def encodeTokenStream (ts : TokenStream) : Serializer Unit := do
  encodeFieldString 1 ts.sourceFilename
  encodeFieldString 2 ts.sourceContent
  -- Encode repeated tokens
  for tok in ts.tokens do
    encodeFieldMessage 3 (encodeToken tok)

def decodeTokenStream : Deserializer TokenStream := do
  let mut filename := ""
  let mut content := ""
  let mut toks : List Token := []
  
  -- Simplified decoder
  return { tokens := toks, sourceFilename := filename, sourceContent := content }

-- Serialize TokenStream to bytes
def serializeTokenStream (ts : TokenStream) : ByteArray :=
  runSerializer (encodeTokenStream ts)

-- Deserialize TokenStream from bytes
def deserializeTokenStream (data : ByteArray) : Except String TokenStream :=
  match decodeTokenStream data 0 with
  | Except.ok (ts, _) => Except.ok ts
  | Except.error e => Except.error e

end Compiler.Proto.Codec.Token
