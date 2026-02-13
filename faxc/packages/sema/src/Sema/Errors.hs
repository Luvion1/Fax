module Sema.Errors where

import Sema.Diag (Diag(..), Severity(..), Type(..), SemanticError(..), toDiag)
import Data.List (intercalate)

-- Suggestion system
data Suggestion = Suggestion
    { suggestionTitle :: String
    , suggestionFix :: String
    } deriving (Show, Eq)

getSuggestion :: SemanticError -> Maybe Suggestion
getSuggestion err = case err of
    UndefinedSymbol s -> Just $ Suggestion
        { suggestionTitle = "Undefined variable"
        , suggestionFix = "Did you mean to declare 'let " ++ s ++ " = ...' before using it?"
        }
    TypeMismatch name expected actual -> Just $ Suggestion
        { suggestionTitle = "Type mismatch"
        , suggestionFix = "Expected " ++ show expected ++ " but got " ++ show actual ++ ".\n" ++
            case (expected, actual) of
                (TBool, TI64) -> "Hint: Use comparison operators (==, !=, <, >) to get a boolean"
                (TI64, TBool) -> "Hint: Use 'if' expression or boolean to integer conversion"
                (TStr, TI64) -> "Hint: Use toString() or string interpolation"
                (TPtr _, TNull) -> "Hint: The value is null, check with 'if x != null' before using"
                _ -> "Hint: Check the expression type or add explicit type annotation"
        }
    ImmutableAssignment s -> Just $ Suggestion
        { suggestionTitle = "Cannot assign to immutable variable"
        , suggestionFix = "Change 'let " ++ s ++ "' to 'let mut " ++ s ++ "' to make it mutable"
        }
    ConditionMustBeBool ctx -> Just $ Suggestion
        { suggestionTitle = "Condition must be boolean"
        , suggestionFix = "Use a comparison operator or boolean expression\n" ++
            "Examples: 'x > 0', 'x == y', 'flag && otherFlag'"
        }
    ArgCountMismatch expected actual -> Just $ Suggestion
        { suggestionTitle = "Wrong number of arguments"
        , suggestionFix = "Expected " ++ show expected ++ " arguments but got " ++ show actual ++ ".\n" ++
            if expected > actual
            then "Hint: Add missing arguments"
            else "Hint: Remove extra arguments"
        }
    ArgTypeMismatch idx expected actual -> Just $ Suggestion
        { suggestionTitle = "Wrong argument type"
        , suggestionFix = "Argument " ++ show idx ++ " expected " ++ show expected ++ 
            " but got " ++ show actual
        }
    MissingReturn s -> Just $ Suggestion
        { suggestionTitle = "Missing return statement"
        , suggestionFix = "Add a 'return' statement at the end of function '" ++ s ++ "'\n" ++
            "Example: 'return defaultValue;' or ensure all branches return"
        }
    BreakOutsideLoop -> Just $ Suggestion
        { suggestionTitle = "Break outside loop"
        , suggestionFix = "'break' can only be used inside loops (while, for)\n" ++
            "Check if you're in the correct scope"
        }
    ContinueOutsideLoop -> Just $ Suggestion
        { suggestionTitle = "Continue outside loop"
        , suggestionFix = "'continue' can only be used inside loops (while, for)\n" ++
            "Check if you're in the correct scope"
        }
    UnreachableCode -> Just $ Suggestion
        { suggestionTitle = "Unreachable code"
        , suggestionFix = "Remove this code or fix the control flow before it"
        }
    FieldNotFound field struct -> Just $ Suggestion
        { suggestionTitle = "Field not found"
        , suggestionFix = "Field '" ++ field ++ "' does not exist in struct '" ++ struct ++ "'\n" ++
            "Check the struct definition or use the correct field name"
        }
    StructNotFound s -> Just $ Suggestion
        { suggestionTitle = "Struct not found"
        , suggestionFix = "Struct '" ++ s ++ "' is not defined\n" ++
            "Define it with: 'struct " ++ s ++ " { ... }'"
        }
    DuplicateSymbol s -> Just $ Suggestion
        { suggestionTitle = "Duplicate definition"
        , suggestionFix = "Symbol '" ++ s ++ "' is already defined\n" ++
            "Rename one of the definitions or remove the duplicate"
        }
    IndexMustBeI64 -> Just $ Suggestion
        { suggestionTitle = "Index must be i64"
        , suggestionFix = "Array indices must be of type i64 (integer)\n" ++
            "Use an integer expression or cast with 'as i64'"
        }
    NotAFunction s -> Just $ Suggestion
        { suggestionTitle = "Not a function"
        , suggestionFix = "'" ++ s ++ "' is not a function and cannot be called\n" ++
            "Check the variable type or use the correct function name"
        }
    NotAStruct t -> Just $ Suggestion
        { suggestionTitle = "Not a struct"
        , suggestionFix = "Value of type " ++ show t ++ " does not support field access\n" ++
            "Use '.' operator only on struct or class instances"
        }
    NotAnArray t -> Just $ Suggestion
        { suggestionTitle = "Not an array"
        , suggestionFix = "Value of type " ++ show t ++ " does not support indexing\n" ++
            "Use '[]' operator only on arrays and strings"
        }
    UnusedVariable s -> Just $ Suggestion
        { suggestionTitle = "Unused variable"
        , suggestionFix = "Variable '" ++ s ++ "' is declared but never used\n" ++
            "Remove it or prefix with '_' to silence warning: '_" ++ s ++ "'"
        }
    ShadowingWarning s -> Just $ Suggestion
        { suggestionTitle = "Variable shadowing"
        , suggestionFix = "Variable '" ++ s ++ "' shadows outer variable\n" ++
            "Rename one to avoid confusion, or use same name intentionally"
        }
    ConstantCondition ctx val -> Just $ Suggestion
        { suggestionTitle = "Constant condition"
        , suggestionFix = "Condition in " ++ ctx ++ " is always " ++ show val ++ ".\n" ++
            if val 
            then "Hint: Remove the condition or the 'else' branch"
            else "Hint: Remove the entire " ++ ctx ++ " statement"
        }
    InfiniteLoopDetected -> Just $ Suggestion
        { suggestionTitle = "Infinite loop"
        , suggestionFix = "This loop will run forever\n" ++
            "Add a 'break' statement or modify the condition to eventually become false"
        }
    SuspiciousPattern s -> Just $ Suggestion
        { suggestionTitle = "Suspicious code pattern"
        , suggestionFix = s
        }
    UnreachableCodeAfter s -> Just $ Suggestion
        { suggestionTitle = "Unreachable code"
        , suggestionFix = "Remove unreachable " ++ s ++ " after return/break/continue"
        }
    NonExhaustivePattern s -> Just $ Suggestion
        { suggestionTitle = "Non-exhaustive pattern match"
        , suggestionFix = s ++ "\nAdd a default case 'default => ...' or handle all cases"
        }
    RedundantPattern s -> Just $ Suggestion
        { suggestionTitle = "Redundant pattern"
        , suggestionFix = s ++ "\nRemove this pattern as it will never match"
        }
    _ -> Nothing

formatSuggestion :: Suggestion -> String
formatSuggestion s = "\n\ESC[1;36mhelp:\ESC[0m " ++ suggestionTitle s ++ "\n\n" ++
    unlines (map ("    " ++) (lines (suggestionFix s)))

hasSuggestion :: SemanticError -> Bool
hasSuggestion err = case getSuggestion err of
    Just _ -> True
    Nothing -> False
