module Sema.ControlFlow where

import Sema.Types
import Sema.ASTUtils (lookupField, getJStr, getLoc)
import Sema.Diag (Diag(..), Severity(..), toDiag, SemanticError(..), Type(..))
import Sema.Errors
import qualified Data.Map as Map

data TerminationStatus
    = AlwaysTerminates
    | MayTerminate
    | NeverTerminates
    deriving (Show, Eq)

-- Check if a statement or block always returns/is terminated
checkTermination :: [JVal] -> TerminationStatus
checkTermination [] = NeverTerminates
checkTermination (stmt:rest) =
    case lookupField "type" stmt of
        Just (JStr "ReturnStatement") -> AlwaysTerminates
        Just (JStr "BreakStatement") -> AlwaysTerminates
        Just (JStr "ContinueStatement") -> AlwaysTerminates
        Just (JStr "IfStatement") -> 
            let thenBranch = maybe [] (\(JArr a) -> a) (lookupField "then_branch" stmt)
                elseBranch = maybe [] (\(JArr a) -> a) (lookupField "else_branch" stmt)
                thenTerm = checkTermination thenBranch
                elseTerm = checkTermination elseBranch
            in case (thenTerm, elseTerm) of
                (AlwaysTerminates, AlwaysTerminates) -> AlwaysTerminates
                _ -> checkTermination rest
        Just (JStr "WhileStatement") ->
            let cond = lookupField "condition" stmt
                body = maybe [] (\(JArr a) -> a) (lookupField "body" stmt)
            in if isAlwaysTrue cond
               then case checkTermination body of
                   AlwaysTerminates -> AlwaysTerminates
                   _ -> NeverTerminates
               else checkTermination rest
        Just (JStr "ForStatement") -> checkTermination rest
        Just (JStr "Block") ->
            let body = maybe [] (\(JArr a) -> a) (lookupField "body" stmt)
            in case checkTermination body of
                AlwaysTerminates -> AlwaysTerminates
                _ -> checkTermination rest
        _ -> checkTermination rest

isAlwaysTrue :: Maybe JVal -> Bool
isAlwaysTrue Nothing = False
isAlwaysTrue (Just node) =
    case lookupField "type" node of
        Just (JStr "Boolean") -> 
            case lookupField "value" node of
                Just (JBool True) -> True
                _ -> False
        Just (JStr "NumberLiteral") ->
            case lookupField "value" node of
                Just (JNum n) -> n /= "0"
                _ -> False
        _ -> False

-- Check function body for missing return
analyzeFunction :: JVal -> Context -> Context
analyzeFunction funcNode ctx =
    case lookupField "returnType" funcNode of
        Just (JStr "void") -> ctx  -- void functions don't need returns
        _ -> 
            let body = maybe [] (\(JArr a) -> a) (lookupField "body" funcNode)
            in case checkTermination body of
                NeverTerminates -> 
                    let (l, c) = getLoc funcNode
                        name = maybe "anonymous" getJStr (lookupField "name" funcNode)
                    in addDiag (toDiag l c (MissingReturn name)) ctx
                MayTerminate ->
                    let (l, c) = getLoc funcNode
                        name = maybe "anonymous" getJStr (lookupField "name" funcNode)
                    in addDiag (toDiag l c (PossibleMissingReturn name)) ctx
                AlwaysTerminates -> ctx

-- Find unreachable code after return/break/continue
findUnreachableCode :: [JVal] -> [(Int, Int, String)]
findUnreachableCode stmts = findUnreachable stmts False
  where
    findUnreachable [] _ = []
    findUnreachable (stmt:rest) terminated =
        case lookupField "type" stmt of
            Just (JStr "ReturnStatement") -> 
                findUnreachableAfter rest
            Just (JStr "BreakStatement") ->
                findUnreachableAfter rest
            Just (JStr "ContinueStatement") ->
                findUnreachableAfter rest
            Just (JStr "IfStatement") ->
                let thenBranch = maybe [] (\(JArr a) -> a) (lookupField "then_branch" stmt)
                    elseBranch = maybe [] (\(JArr a) -> a) (lookupField "else_branch" stmt)
                    unreachableInThen = findUnreachableCode thenBranch
                    unreachableInElse = findUnreachableCode elseBranch
                    (l, c) = getLoc stmt
                in if allTerminates thenBranch && allTerminates elseBranch
                   then findUnreachableAfter rest
                   else unreachableInThen ++ unreachableInElse ++ findUnreachable rest terminated
            Just (JStr "WhileStatement") ->
                let body = maybe [] (\(JArr a) -> a) (lookupField "body" stmt)
                in findUnreachableCode body ++ findUnreachable rest terminated
            Just (JStr "ForStatement") ->
                let body = maybe [] (\(JArr a) -> a) (lookupField "body" stmt)
                in findUnreachableCode body ++ findUnreachable rest terminated
            Just (JStr "Block") ->
                let body = maybe [] (\(JArr a) -> a) (lookupField "body" stmt)
                    unreachableInBlock = findUnreachableCode body
                in unreachableInBlock ++ findUnreachable rest terminated
            _ -> findUnreachable rest terminated
      where
        findUnreachableAfter [] = []
        findUnreachableAfter (s:ss) =
            let (l, c) = getLoc s
                stmtType = maybe "statement" getJStr (lookupField "type" s)
            in (l, c, stmtType) : findUnreachableAfter ss

allTerminates :: [JVal] -> Bool
allTerminates stmts = checkTermination stmts == AlwaysTerminates

-- Check for unused imports/modules (placeholder for future module system)
checkUnusedImports :: Context -> Context
checkUnusedImports ctx = ctx  -- Placeholder

-- Detect suspicious code patterns
detectSuspiciousPatterns :: JVal -> Context -> Context
detectSuspiciousPatterns node ctx =
    case lookupField "type" node of
        Just (JStr "BinaryExpression") ->
            let op = maybe "" getJStr (lookupField "op" node)
                left = lookupField "left" node
                right = lookupField "right" node
            in checkSuspiciousBinOp op left right node ctx
        Just (JStr "IfStatement") ->
            let cond = lookupField "condition" node
            in checkSuspiciousCondition cond node ctx
        _ -> ctx
  where
    checkSuspiciousBinOp "eq" (Just left) (Just right) parent c =
        case (lookupField "type" left, lookupField "type" right) of
            (Just (JStr "ArrayLiteral"), _) ->
                let (l, col) = getLoc parent
                in addDiag (toDiag l col (SuspiciousPattern "Comparing arrays with == uses reference equality, not content comparison")) c
            (_, Just (JStr "ArrayLiteral")) ->
                let (l, col) = getLoc parent
                in addDiag (toDiag l col (SuspiciousPattern "Comparing arrays with == uses reference equality, not content comparison")) c
            _ -> c
    checkSuspiciousBinOp _ _ _ _ c = c

    checkSuspiciousCondition Nothing _ c = c
    checkSuspiciousCondition (Just cond) parent c =
        case lookupField "type" cond of
            Just (JStr "Assignment") ->
                let (l, col) = getLoc parent
                in addDiag (toDiag l col (SuspiciousPattern "Assignment used as condition - did you mean to use '=='?")) c
            _ -> c

-- Pattern exhaustiveness checking (merged from PatternExhaustiveness.hs)
data Pattern
    = PWild       -- Wildcard (_)
    | PVar String -- Variable
    | PCon String -- Constructor/Literal
    | PInt Integer
    | PBool Bool
    | PChar Char
    | PNull
    | POr [Pattern] -- Pattern alternatives
    deriving (Show, Eq)

checkExhaustiveness :: JVal -> Context -> Context
checkExhaustiveness matchNode ctx =
    let target = lookupField "target" matchNode
        cases = maybe [] (\(JArr a) -> a) (lookupField "cases" matchNode)
        defaultCase = lookupField "default" matchNode
        (l, c) = getLoc matchNode
    in case target of
        Nothing -> ctx
        Just t ->
            let targetTy = getTypeFromAST t
                patterns = map extractPattern cases
            in if hasDefault defaultCase
               then ctx  -- Has default case, always exhaustive
               else case checkPatterns targetTy patterns of
                   Just uncovered ->
                       let msg = "Non-exhaustive pattern match. Missing: " ++ show uncovered
                       in addDiag (toDiag l c (NonExhaustivePattern msg)) ctx
                   Nothing -> ctx

extractPattern :: JVal -> Pattern
extractPattern caseNode =
    case lookupField "pattern" caseNode of
        Just pat -> parsePattern pat
        Nothing -> PWild

parsePattern :: JVal -> Pattern
parsePattern node =
    case lookupField "type" node of
        Just (JStr "Identifier") ->
            case lookupField "value" node of
                Just (JStr "_") -> PWild
                Just (JStr name) -> PVar name
                _ -> PWild
        Just (JStr "NumberLiteral") ->
            case lookupField "value" node of
                Just (JNum n) -> case reads n of
                    [(v, "")] -> PInt v
                    _ -> PWild
                _ -> PWild
        Just (JStr "Boolean") ->
            case lookupField "value" node of
                Just (JBool b) -> PBool b
                _ -> PWild
        Just (JStr "StringLiteral") ->
            case lookupField "value" node of
                Just (JStr s) -> if length s == 1 then PChar (head s) else PCon s
                _ -> PWild
        Just (JStr "NullLiteral") -> PNull
        Just (JStr "Constructor") ->
            case lookupField "name" node of
                Just (JStr name) -> PCon name
                _ -> PWild
        _ -> PWild

getTypeFromAST :: JVal -> Type
getTypeFromAST node =
    case lookupField "type" node of
        Just (JStr "Identifier") -> TUnk  -- Need type inference
        Just (JStr "NumberLiteral") -> TI64
        Just (JStr "Boolean") -> TBool
        Just (JStr "StringLiteral") -> TStr
        Just (JStr "NullLiteral") -> TNull
        _ -> TUnk

hasDefault :: Maybe JVal -> Bool
hasDefault Nothing = False
hasDefault (Just _) = True

checkPatterns :: Type -> [Pattern] -> Maybe [Pattern]
checkPatterns ty patterns =
    case ty of
        TBool -> checkBoolPatterns patterns
        TI64 -> Nothing
        TStr -> Nothing
        TNull -> Nothing
        TUnk -> Nothing
        TStruct _ -> Nothing
        TArr _ -> Nothing
        _ -> Nothing

checkBoolPatterns :: [Pattern] -> Maybe [Pattern]
checkBoolPatterns patterns =
    let hasTrue = any isTruePattern patterns
        hasFalse = any isFalsePattern patterns
        hasWild = any isWildPattern patterns
    in if hasWild || (hasTrue && hasFalse)
       then Nothing
       else Just $ concat [
           if hasTrue then [] else [PBool True],
           if hasFalse then [] else [PBool False]
       ]

isTruePattern :: Pattern -> Bool
isTruePattern (PBool True) = True
isTruePattern PWild = True
isTruePattern _ = False

isFalsePattern :: Pattern -> Bool
isFalsePattern (PBool False) = True
isFalsePattern PWild = True
isFalsePattern _ = False

isWildPattern :: Pattern -> Bool
isWildPattern PWild = True
isWildPattern _ = False

checkRedundantPatterns :: [JVal] -> Context -> Context
checkRedundantPatterns cases ctx =
    let patterns = map extractPattern cases
        (_, redundant) = foldl checkRedundant ([], []) (zip [0..] patterns)
    in foldl addRedundantError ctx redundant
  where
    checkRedundant (seen, redundant) (idx, pat) =
        if isCovered pat seen
        then (seen, redundant ++ [(idx, pat)])
        else (seen ++ [pat], redundant)

    isCovered _ [] = False
    isCovered PWild _ = True
    isCovered (PBool True) seen = PBool True `elem` seen || PWild `elem` seen
    isCovered (PBool False) seen = PBool False `elem` seen || PWild `elem` seen
    isCovered (PInt n) seen = PInt n `elem` seen || PWild `elem` seen
    isCovered (PVar _) _ = False
    isCovered _ _ = False

    addRedundantError c (idx, pat) =
        let (l, col) = getLoc (cases !! idx)
            msg = "Redundant pattern: " ++ show pat
        in addDiag (toDiag l col (RedundantPattern msg)) c
