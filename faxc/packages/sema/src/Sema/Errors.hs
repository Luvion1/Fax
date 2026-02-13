module Sema.Errors where

import Sema.Diagnostics
import {-# SOURCE #-} Sema.Types (Type)

data SemanticError
    = UndefinedSymbol String
    | TypeMismatch String Type Type
    | NotAFunction String
    | ArgCountMismatch Int Int
    | ConditionMustBeBool String
    | RangeBoundsMustBeI64
    | ImmutableAssignment String
    | FieldNotFound String String
    | StructNotFound String
    | NotAStruct Type
    | NotAnArray Type
    | ArrayTypeMismatch
    | DuplicateSymbol String
    | MissingField String
    | ExtraField String
    | UnusedVariable String
    | ReturnTypeMismatch Type Type
    | BreakOutsideLoop
    | ContinueOutsideLoop
    deriving (Show, Eq)

getErrorInfo :: SemanticError -> (String, String)
getErrorInfo err = case err of
    UndefinedSymbol s -> ("E001", "Undefined symbol: " ++ s)
    TypeMismatch n exp act -> ("E002", "Type mismatch for '" ++ n ++ "': expected " ++ show exp ++ ", got " ++ show act)
    NotAFunction s -> ("E003", "'" ++ s ++ "' is not a function")
    ArgCountMismatch exp act -> ("E004", "Argument count mismatch: expected " ++ show exp ++ ", got " ++ show act)
    ConditionMustBeBool ctx -> ("E005", "Condition in " ++ ctx ++ " must be boolean")
    RangeBoundsMustBeI64 -> ("E006", "Range bounds must be i64")
    ImmutableAssignment s -> ("E007", "Cannot assign to immutable variable: " ++ s)
    FieldNotFound f s -> ("E008", "Field '" ++ f ++ "' not found in struct '" ++ s ++ "'")
    StructNotFound s -> ("E009", "Struct definition not found: " ++ s)
    NotAStruct t -> ("E010", "Value of type " ++ show t ++ " is not a struct")
    NotAnArray t -> ("E011", "Value of type " ++ show t ++ " is not an array")
    ArrayTypeMismatch -> ("E012", "Array elements must have the same type")
    DuplicateSymbol s -> ("E013", "Duplicate definition of symbol: " ++ s)
    MissingField f -> ("E014", "Missing required field: " ++ f)
    ExtraField f -> ("E015", "Extra field in struct literal: " ++ f)
    UnusedVariable s -> ("W001", "Variable '" ++ s ++ "' is unused")
    ReturnTypeMismatch exp act -> ("E016", "Return type mismatch: expected " ++ show exp ++ ", got " ++ show act)
    BreakOutsideLoop -> ("E017", "Break statement used outside of loop")
    ContinueOutsideLoop -> ("E018", "Continue statement used outside of loop")

toDiag :: Int -> Int -> SemanticError -> Diag
toDiag l c err = 
    let (code, msg) = getErrorInfo err
        sev = if head code == 'W' then Warning else Error
    in Diag sev code msg l c