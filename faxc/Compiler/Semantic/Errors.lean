/-
Semantic Analysis - Error Handling and Reporting
-/

import Compiler.Semantic.Types

namespace Compiler.Semantic.Errors

open Compiler.Semantic.Types

-- Error severity levels
inductive Severity
  | error
  | warning
  | info
  deriving Repr, BEq

-- Detailed error information
structure ErrorDetail where
  kind : SemanticErrorKind
  severity : Severity
  message : String
  location : Option SourceLocation
  suggestions : List String
  relatedInfo : List (String × Option SourceLocation)
  deriving Repr

-- Error reporter
def ErrorReporter := List ErrorDetail

def ErrorReporter.new : ErrorReporter := []

def ErrorReporter.report (reporter : ErrorReporter) (error : ErrorDetail) : ErrorReporter :=
  error :: reporter

def ErrorReporter.hasErrors (reporter : ErrorReporter) : Bool :=
  reporter.any (λ e => e.severity == .error)

def ErrorReporter.errorCount (reporter : ErrorReporter) : Nat :=
  reporter.filter (λ e => e.severity == .error) |>.length

def ErrorReporter.warningCount (reporter : ErrorReporter) : Nat :=
  reporter.filter (λ e => e.severity == .warning) |>.length

def ErrorReporter.getErrors (reporter : ErrorReporter) : List ErrorDetail :=
  reporter.filter (λ e => e.severity == .error)

def ErrorReporter.getWarnings (reporter : ErrorReporter) : List ErrorDetail :=
  reporter.filter (λ e => e.severity == .warning)

-- Error formatter
def formatError (error : ErrorDetail) : String :=
  let severityStr := match error.severity with
  | .error => "error"
  | .warning => "warning"
  | .info => "info"
  
  let locationStr := match error.location with
  | some loc => s!"{loc.filename}:{loc.line}:{loc.column}: "
  | none => ""
  
  let base := s!"{locationStr}{severityStr}: {error.message}"
  
  let suggestionsStr := if error.suggestions.isEmpty then
    ""
  else
    "\n  help: " ++ String.intercalate "\n       " error.suggestions
  
  let relatedStr := if error.relatedInfo.isEmpty then
    ""
  else
    "\n  note: " ++ String.intercalate "\n       " 
      (error.relatedInfo.map (λ (msg, loc) =>
        match loc with
        | some l => s!"{l.filename}:{l.line}:{l.column}: {msg}"
        | none => msg
      ))
  
  base ++ suggestionsStr ++ relatedStr

-- Pretty print all errors
def formatErrors (errors : List ErrorDetail) : String :=
  errors.reverse.map formatError |> String.intercalate "\n\n"

-- Common error constructors
def typeMismatch (expected : Ty) (actual : Ty) (loc : Option SourceLocation := none) 
    : ErrorDetail :=
  {
    kind := .typeMismatch
    severity := .error
    message := s!"expected type `{Ty.toString expected}`, found `{Ty.toString actual}`"
    location := loc
    suggestions := [s!"consider converting the value to type `{Ty.toString expected}`"]
    relatedInfo := []
  }

def undefinedVariable (name : String) (loc : Option SourceLocation := none)
    : ErrorDetail :=
  {
    kind := .undefinedVariable
    severity := .error
    message := s!"cannot find value `{name}` in this scope"
    location := loc
    suggestions := [s!"check the spelling of `{name}`", "ensure the variable is declared before use"]
    relatedInfo := []
  }

def undefinedFunction (name : String) (loc : Option SourceLocation := none)
    : ErrorDetail :=
  {
    kind := .undefinedFunction
    severity := .error
    message := s!"cannot find function `{name}` in this scope"
    location := loc
    suggestions := [s!"check the spelling of `{name}`", "ensure the function is declared before use"]
    relatedInfo := []
  }

def undefinedType (name : String) (loc : Option SourceLocation := none)
    : ErrorDetail :=
  {
    kind := .undefinedType
    severity := .error
    message := s!"cannot find type `{name}` in this scope"
    location := loc
    suggestions := [s!"check the spelling of `{name}`", "ensure the type is declared before use"]
    relatedInfo := []
  }

def duplicateDefinition (name : String) (existingLoc : Option SourceLocation) 
    (newLoc : Option SourceLocation) : ErrorDetail :=
  {
    kind := .duplicateDefinition
    severity := .error
    message := s!"the name `{name}` is defined multiple times"
    location := newLoc
    suggestions := [s!"rename one of the definitions of `{name}`"]
    relatedInfo := match existingLoc with
    | some loc => [("previous definition is here", some loc)]
    | none => []
  }

def arityMismatch (name : String) (expected : Nat) (actual : Nat) 
    (loc : Option SourceLocation := none) : ErrorDetail :=
  {
    kind := .arityMismatch
    severity := .error
    message := s!"function `{name}` takes {expected} argument(s) but {actual} were supplied"
    location := loc
    suggestions := [s!"provide exactly {expected} argument(s) to `{name}`"]
    relatedInfo := []
  }

def invalidFieldAccess (ty : Ty) (field : String) (loc : Option SourceLocation := none)
    : ErrorDetail :=
  {
    kind := .invalidFieldAccess
    severity := .error
    message := s!"no field `{field}` on type `{Ty.toString ty}`"
    location := loc
    suggestions := ["check the field name", "ensure you're accessing the correct type"]
    relatedInfo := []
  }

def unsupportedPattern (reason : String) (loc : Option SourceLocation := none)
    : ErrorDetail :=
  {
    kind := .unsupportedPattern
    severity := .error
    message := s!"unsupported pattern: {reason}"
    location := loc
    suggestions := ["use a simpler pattern", "split into multiple patterns"]
    relatedInfo := []
  }

def outOfBounds (index : Nat) (size : Nat) (loc : Option SourceLocation := none)
    : ErrorDetail :=
  {
    kind := .outOfBounds
    severity := .error
    message := s!"index {index} is out of bounds (size is {size})"
    location := loc
    suggestions := [s!"use an index between 0 and {size - 1}"]
    relatedInfo := []
  }

-- Warnings
def unusedVariable (name : String) (loc : Option SourceLocation := none)
    : ErrorDetail :=
  {
    kind := .other
    severity := .warning
    message := s!"unused variable: `{name}`"
    location := loc
    suggestions := [s!"if this is intentional, prefix it with an underscore: `_{name}`"]
    relatedInfo := []
  }

def deadCode (loc : Option SourceLocation := none) : ErrorDetail :=
  {
    kind := .other
    severity := .warning
    message := "this code will never be executed"
    location := loc
    suggestions := ["remove the unreachable code"]
    relatedInfo := []
  }

def unnecessaryMutable (name : String) (loc : Option SourceLocation := none)
    : ErrorDetail :=
  {
    kind := .other
    severity := .warning
    message := s!"variable `{name}` does not need to be mutable"
    location := loc
    suggestions := [s!"remove `mut` from the declaration of `{name}`"]
    relatedInfo := []
  }

-- Error recovery strategies
def canRecover (error : ErrorDetail) : Bool :=
  match error.severity with
  | .error => false
  | .warning | .info => true

-- Error grouping
def groupByLocation (errors : List ErrorDetail) : List (Option SourceLocation × List ErrorDetail) :=
  let grouped := errors.groupBy (λ e1 e2 => e1.location == e2.location)
  grouped.map (λ group => (group.head!.location, group))

def groupByKind (errors : List ErrorDetail) : List (SemanticErrorKind × List ErrorDetail) :=
  let grouped := errors.groupBy (λ e1 e2 => e1.kind == e2.kind)
  grouped.map (λ group => (group.head!.kind, group))

-- Statistics
def errorStatistics (errors : List ErrorDetail) : ErrorStatistics :=
  let bySeverity := errors.groupBy (λ e1 e2 => e1.severity == e2.severity)
  {
    totalErrors := errors.filter (λ e => e.severity == .error) |>.length
    totalWarnings := errors.filter (λ e => e.severity == .warning) |>.length
    totalInfo := errors.filter (λ e => e.severity == .info) |>.length
    byKind := groupByKind errors |>.map (λ (k, es) => (k, es.length))
  }

structure ErrorStatistics where
  totalErrors : Nat
  totalWarnings : Nat
  totalInfo : Nat
  byKind : List (SemanticErrorKind × Nat)
  deriving Repr

-- Export error summary
def formatStatistics (stats : ErrorStatistics) : String :=
  s!"errors: {stats.totalErrors}, warnings: {stats.totalWarnings}, info: {stats.totalInfo}"

end Compiler.Semantic.Errors
