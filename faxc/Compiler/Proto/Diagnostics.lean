/-
Error handling and diagnostics for Fax compiler via protobuf
-/

import Compiler.Proto.Messages
import Compiler.Proto.Binary

namespace Compiler.Proto.Diagnostics

-- Diagnostic severity (mirrors compiler.proto)
inductive Severity
  | error | warning | info | hint
  deriving Repr, BEq

def Severity.toNat : Severity → Nat
| .error => 0 | .warning => 1 | .info => 2 | .hint => 3

def Severity.fromNat : Nat → Severity
| 1 => .warning | 2 => .info | 3 => .hint | _ => .error

-- Source location
def Severity.toString : Severity → String
| .error => "error"
| .warning => "warning"
| .info => "info"
| .hint => "hint"

-- Diagnostic message
structure Diagnostic where
  severity : Severity
  code : String
  message : String
  file : String
  line : Nat
  column : Nat
  length : Nat  -- Length of error span
  related : List (String × Messages.SourceRange)  -- Related information
  suggestions : List String  -- Fix suggestions
  deriving Repr

-- Diagnostics collection
structure Diagnostics where
  items : List Diagnostic
  fileName : String
  source : String
  deriving Repr

def Diagnostics.empty (file : String := "") (src : String := "") : Diagnostics :=
  { items := [], fileName := file, source := src }

def Diagnostics.add (d : Diagnostics) (diag : Diagnostic) : Diagnostics :=
  { d with items := diag :: d.items }

def Diagnostics.hasErrors (d : Diagnostics) : Bool :=
  d.items.any (λ diag => diag.severity == .error)

def Diagnostics.errorCount (d : Diagnostics) : Nat :=
  d.items.filter (λ diag => diag.severity == .error) |>.length

def Diagnostics.warningCount (d : Diagnostics) : Nat :=
  d.items.filter (λ diag => diag.severity == .warning) |>.length

-- Create formatted error message
def Diagnostic.format (d : Diagnostic) : String :=
  let severityStr := d.severity.toString
  let locationStr := s!"{d.file}:{d.line}:{d.column}"
  s!"{locationStr}: {severityStr}[{d.code}]: {d.message}"

-- Format all diagnostics
def Diagnostics.format (d : Diagnostics) : String :=
  let header := s!"Diagnostics for {d.fileName}:"
  let items := d.items.reverse.map Diagnostic.format
  String.intercalate "\n" (header :: items)

-- Protobuf encoding for Diagnostics
def encodeSeverity (s : Severity) : Binary.Serializer Unit :=
  Binary.encodeFieldVarint 1 s.toNat.toUInt64

def encodeDiagnostic (d : Diagnostic) : Binary.Serializer Unit := do
  encodeSeverity d.severity
  Binary.encodeFieldString 2 d.code
  Binary.encodeFieldString 3 d.message
  Binary.encodeFieldString 4 d.file
  Binary.encodeFieldVarint 5 d.line.toUInt64
  Binary.encodeFieldVarint 6 d.column.toUInt64
  Binary.encodeFieldVarint 7 d.length.toUInt64

def serializeDiagnostics (d : Diagnostics) : ByteArray :=
  let header := Binary.runSerializer do
    Binary.encodeFieldString 1 d.fileName
    Binary.encodeFieldString 2 d.source
  
  let items := d.items.map (λ diag =>
    Binary.runSerializer (encodeDiagnostic diag))
  
  header ++ items.foldl ByteArray.append ByteArray.empty

-- Error reporter monad
structure ReporterM (α : Type) where
  run : Diagnostics → (α × Diagnostics)

def ReporterM.pure (v : α) : ReporterM α :=
  { run := λ d => (v, d) }

def ReporterM.bind {α β : Type} (m : ReporterM α) (f : α → ReporterM β) : ReporterM β :=
  { run := λ d =>
    let (v, d') := m.run d
    (f v).run d' }

instance : Monad ReporterM where
  pure := ReporterM.pure
  bind := ReporterM.bind

def report (diag : Diagnostic) : ReporterM Unit :=
  { run := λ d => ((), d.add diag) }

def getDiagnostics : ReporterM Diagnostics :=
  { run := λ d => (d, d) }

def runReporter {α : Type} (m : ReporterM α) : (α × Diagnostics) :=
  m.run (Diagnostics.empty)

-- Convenience functions for reporting
def reportError (code : String) (message : String) (location : Messages.SourceRange) 
    : ReporterM Unit :=
  report {
    severity := .error
    code := code
    message := message
    file := location.start.filename
    line := location.start.line
    column := location.start.column
    length := 0
    related := []
    suggestions := []
  }

def reportWarning (code : String) (message : String) (location : Messages.SourceRange)
    : ReporterM Unit :=
  report {
    severity := .warning
    code := code
    message := message
    file := location.start.filename
    line := location.start.line
    column := location.start.column
    length := 0
    related := []
    suggestions := []
  }

def reportInfo (message : String) : ReporterM Unit :=
  report {
    severity := .info
    code := "INFO"
    message := message
    file := ""
    line := 0
    column := 0
    length := 0
    related := []
    suggestions := []
  }

-- Error codes (following SPEC.md)
namespace ErrorCodes

def LEX_INVALID_CHAR := "E0001"
def LEX_UNTERMINATED_STRING := "E0002"
def PARSE_UNEXPECTED_TOKEN := "E0100"
def PARSE_EXPECTED_SEMICOLON := "E0101"
def SEM_TYPE_MISMATCH := "E0200"
def SEM_UNDEFINED_VAR := "E0201"
def SEM_UNDEFINED_FUNC := "E0202"
def SEM_DUPLICATE_DEF := "E0203"
def CODEGEN_UNSUPPORTED := "E0300"

end ErrorCodes

-- Integration with protobuf services
def createErrorResponse (diags : Diagnostics) : Messages.CompilerError :=
  let firstError := diags.items.find? (λ d => d.severity == .error)
  match firstError with
  | some e =>
    { severity := Messages.ErrorSeverity.error
      message := e.message
      code := e.code
      location := {
        start := { filename := e.file, line := e.line, column := e.column, offset := 0 }
        «end» := { filename := e.file, line := e.line, column := e.column + e.length, offset := 0 }
      }
      notes := diags.items.filter (λ d => d.severity == .info) |>.map Diagnostic.message
    }
  | none =>
    { severity := Messages.ErrorSeverity.error
      message := "Unknown error"
      code := "E9999"
      location := Messages.SourceRange.default
      notes := []
    }

end Compiler.Proto.Diagnostics
