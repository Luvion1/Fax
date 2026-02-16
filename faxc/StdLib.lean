-- Fax Standard Library
-- Provides basic I/O and utility functions

namespace Fax.StdLib

open Compiler.AST
open Compiler.AST.Types
open Compiler.Codegen
open Compiler.Codegen.IR
open Compiler.Codegen.Expr

-- Standard library function definitions as AST declarations
def printlnInt : Decl :=
  .funDecl false "println" [("x", .int32)] .unit 
    (.lit (.intLit 0)) -- Placeholder body, will be replaced with intrinsic

def printlnString : Decl :=
  .funDecl false "println_str" [("s", .string)] .unit
    (.lit (.intLit 0))

def printInt : Decl :=
  .funDecl false "print" [("x", .int32)] .unit
    (.lit (.intLit 0))

def readInt : Decl :=
  .funDecl false "read_int" [] .int32
    (.lit (.intLit 0))

-- List of all standard library functions
def allStdLib : List Decl :=
  [printlnInt, printlnString, printInt, readInt]

-- Generate standard library runtime code
def generateStdLibRuntime : String :=
  "; Fax Standard Library Runtime

; Print integer with newline
define void @println(i32 %x) {
entry:
  %fmt = getelementptr [4 x i8], [4 x i8]* @.str_int, i64 0, i64 0
  call i32 (i8*, ...) @printf(i8* %fmt, i32 %x)
  ret void
}

; Print string with newline
define void @println_str(i8* %s) {
entry:
  call i32 @puts(i8* %s)
  ret void
}

; Print integer without newline
define void @print(i32 %x) {
entry:
  %fmt = getelementptr [4 x i8], [4 x i8]* @.str_int, i64 0, i64 0
  call i32 (i8*, ...) @printf(i8* %fmt, i32 %x)
  ret void
}

; Read integer from stdin (simplified)
define i32 @read_int() {
entry:
  ; Returns 0 as placeholder
  ; In real implementation, would use scanf
  ret i32 0
}

"

-- Add standard library declarations to module
def addStdLibToModule (module : Module) : Module :=
  { module with decls := allStdLib ++ module.decls }

-- Check if a function is a standard library function
def isStdLibFunc (name : String) : Bool :=
  ["println", "println_str", "print", "read_int"].contains name

end Fax.StdLib
