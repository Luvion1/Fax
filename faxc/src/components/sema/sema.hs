{-|
Module      : FAX.Semantics
Description : Semantic analysis and type checking for FAX programs
Copyright   : (c) 2024 FAX Team
License     : MIT

This module implements the semantic analysis phase of the FAX compiler.
It performs:
- Symbol table management with scope tracking
- Type inference and checking
- Function signature validation
- Rust-style error reporting with error codes
-}

{-# LANGUAGE OverloadedStrings #-}

-- =============================================================================
-- MINIMAL JSON PARSER
-- =============================================================================
-- A hand-written JSON parser that converts JSON strings into JsonValue objects.
-- This is necessary for language-independent communication in the polyglot pipeline.

data JsonValue = JObj [(String, JsonValue)]
               | JArr [JsonValue]
               | JStr String
               | JNum Double
               | JBool Bool
               | JNull
               deriving (Show, Eq)

parseJson :: String -> (JsonValue, String)
parseJson s = 
    let s' = dropWhile isSpace s in
    case s' of
        ('{':xs) -> parseObject xs
        ('[':xs) -> parseArray xs
        ('"':xs) -> let (str, rest) = parseString xs in (JStr str, rest)
        ('t':'r':'u':'e':xs) -> (JBool True, xs)
        ('f':'a':'l':'s':'e':xs) -> (JBool False, xs)
        ('n':'u':'l':'l':xs) -> (JNull, xs)
        (c:xs) | isDigit c || c == '-' -> parseNumber s'
        _ -> (JNull, s')

parseString :: String -> (String, String)
parseString ('"':xs) = ("", xs)
parseString ('\\':c:xs) = let (s, r) = parseString xs in (c:s, r)
parseString (x:xs) = let (s, r) = parseString xs in (x:s, r)
parseString [] = ("", "")

parseObject :: String -> (JsonValue, String)
parseObject s = 
    let s' = dropWhile isSpace s in
    case s' of
        ('}':xs) -> (JObj [], xs)
        _ -> let (pairs, rest) = parsePairs s' in (JObj pairs, rest)

parsePairs :: String -> ([(String, JsonValue)], String)
parsePairs s =
    let s1 = dropWhile (\c -> isSpace c || c == '"') s
        (key, s2) = parseString s1
        s3 = dropWhile (\c -> isSpace c || c == ':') s2
        (val, s4) = parseJson s3
        s5 = dropWhile isSpace s4
    in case s5 of
        (',':xs) -> let (ps, r) = parsePairs xs in ((key, val):ps, r)
        ('}':xs) -> ([(key, val)], xs)
        _ -> ([(key, val)], s5)

parseArray :: String -> (JsonValue, String)
parseArray s =
    let s' = dropWhile isSpace s in
    case s' of
        (']':xs) -> (JArr [], xs)
        _ -> let (vals, rest) = parseElements s' in (JArr vals, rest)

parseElements :: String -> ([JsonValue], String)
parseElements s =
    let (val, s1) = parseJson s
        s2 = dropWhile isSpace s1
    in case s2 of
        (',':xs) -> let (vs, r) = parseElements xs in (val:vs, r)
        (']':xs) -> ([val], xs)
        _ -> ([val], s2)

parseNumber :: String -> (JsonValue, String)
parseNumber s =
    let (numStr, rest) = span (\c -> isDigit c || (c == '.' || c == 'e' || c == '-') && not (null numStr && c == '.')) s
    in (JNum (if null numStr || numStr == "-" then 0 else read numStr), rest)

getAttr :: String -> JsonValue -> Maybe JsonValue
getAttr key (JObj pairs) = lookup key pairs
getAttr _ _ = Nothing

-- =============================================================================
-- RUST-STYLE ERROR HANDLING
-- =============================================================================
-- Emulates Rust's structured error reporting with error codes and formatted output

data ErrorCode = E0425 -- Cannot find value
               | E0308 -- Type mismatch
               | E0061 -- Argument mismatch
               | E0428 -- Redefinition
               | E0000 -- Unknown
               deriving (Eq)

instance Show ErrorCode where
    show E0425 = "E0425"
    show E0308 = "E0308"
    show E0061 = "E0061"
    show E0428 = "E0428"
    show E0000 = "E0000"

data SemanticError = SemanticError 
    { errCode    :: ErrorCode
    , errMsg     :: String
    , errContext :: String
    , errLabel   :: String
    , errHelp    :: String
    } deriving (Show)

formatRustError :: SemanticError -> String
formatRustError err =
    let codeStr = show (errCode err)
        topLine = "\x1b[1;31merror[" ++ codeStr ++ "]\x1b[0m: \x1b[1m" ++ errMsg err ++ "\x1b[0m"
        midLine = "  |"
        ctxLine = "  | " ++ errContext err
        lblLine = "  | \x1b[1;31m^ " ++ errLabel err ++ "\x1b[0m"
        helpLine = "  |\n  = \x1b[1;36mhelp\x1b[0m: " ++ errHelp err
        footer  = "\x1b[1;34m-->\x1b[0m for more info, see https://doc.fax-lang.org/error-index.html#" ++ codeStr
    in intercalate "\n" ["", topLine, midLine, ctxLine, lblLine, helpLine, "", footer, ""]

-- =============================================================================
-- SEMANTIC ANALYZER LOGIC
-- =============================================================================
-- Performs type checking, symbol resolution, and semantic validation

data SymbolInfo = SymbolInfo 
    { symType   :: String
    , symKind   :: String -- "var", "fn", "type"
    , symArgs   :: Int    -- For functions
    , symArgTys :: [String] -- Argument types
    , symRetTy  :: String -- Return type
    } deriving (Show)

type Scope = Map.Map String SymbolInfo
data SymbolTable = SymbolTable 
    { scopes   :: [Scope] 
    , currentFn :: Maybe String -- To track return types
    } deriving (Show)

initialTable :: SymbolTable
initialTable = SymbolTable [Map.fromList [
    ("print", SymbolInfo "fn" "fn" 1 ["any"] "void"), 
    ("io::print", SymbolInfo "fn" "fn" 1 ["any"] "void"), 
    ("Std::io::collect_gc", SymbolInfo "fn" "fn" 0 [] "void"),
    ("true", SymbolInfo "bool" "var" 0 [] ""), 
    ("false", SymbolInfo "bool" "var" 0 [] ""), 
    ("Result", SymbolInfo "type" "type" 0 [] ""), 
    ("Ok", SymbolInfo "fn" "fn" 1 ["any"] "Result"), 
    ("Err", SymbolInfo "fn" "fn" 1 ["any"] "Result"),
    ("i32", SymbolInfo "type" "type" 0 [] ""),
    ("i64", SymbolInfo "type" "type" 0 [] ""), 
    ("str", SymbolInfo "type" "type" 0 [] ""), 
    ("bool", SymbolInfo "type" "type" 0 [] "")
  ]] Nothing

isDefinedInCurrentScope :: String -> SymbolTable -> Bool
isDefinedInCurrentScope name (SymbolTable (curr:_) _) = Map.member name curr
isDefinedInCurrentScope _ _ = False

isDefinedAnywhere :: String -> SymbolTable -> Bool
isDefinedAnywhere name (SymbolTable ss _) = any (Map.member name) ss

getSymbol :: String -> SymbolTable -> Maybe SymbolInfo
getSymbol name (SymbolTable ss _) = foldr (\scope acc -> case Map.lookup name scope of { Just s -> Just s; Nothing -> acc }) Nothing ss

enterScope :: SymbolTable -> SymbolTable
enterScope (SymbolTable ss fn) = SymbolTable (Map.empty : ss) fn

defineSymbol :: String -> SymbolInfo -> SymbolTable -> Either SemanticError SymbolTable
defineSymbol name info st@(SymbolTable (curr:rest) fn) =
    if isDefinedInCurrentScope name st
        then Left $ SemanticError E0428 ("the name `" ++ name ++ "` is defined multiple times") (name ++ " = ...") ("`" ++ name ++ "` redefined here") ("try using a different name for this identifier")
        else Right $ SymbolTable (Map.insert name info curr : rest) fn

inferType :: SymbolTable -> JsonValue -> String
inferType st (JObj pairs) =
    case lookup "type" pairs of
        Just (JStr "Atomic") ->
            case lookup "value" pairs of
                Just (JStr val) | all (\c -> isDigit c || (c == '-' && length val > 1)) val -> "i64"
                Just (JStr val) | "\"" `isPrefixOf` val -> "str"
                Just (JStr val) -> maybe "unknown" symType (getSymbol val st)
                _ -> "unknown"
        Just (JStr "StringLiteral") -> "str"
        Just (JStr "BinaryExpression") -> 
            let op = case lookup "op" pairs of { Just (JStr o) -> o; _ -> "" }
            in if op `elem` ["eq", "ne", "lt", "gt", "le", "ge"] then "bool" else "i64"
        Just (JStr "ComparisonExpression") -> "bool"
        Just (JStr "CallExpression") -> 
            let name = case lookup "name" pairs of { Just (JStr n) -> n; _ -> "" }
            in case getSymbol name st of { Just info -> symRetTy info; _ -> "unknown" }
        _ -> "unknown"
inferType _ _ = "unknown"

checkArgTypes :: [String] -> [String] -> Either (String, String) ()
checkArgTypes [] [] = Right ()
checkArgTypes (e:es) (a:as) =
    if e == "any" || a == "unknown" || e == a
        then checkArgTypes es as
        else Left (e, a)
checkArgTypes _ _ = Right () -- Count mismatch handled elsewhere

analyzeNode :: SymbolTable -> JsonValue -> Either SemanticError SymbolTable
analyzeNode st (JObj pairs) =
    case lookup "type" pairs of
        Just (JStr "Program") -> 
            case lookup "body" pairs of
                Just (JArr nodes) -> foldM analyzeNode st nodes
                _ -> Right st
        
        Just (JStr "FunctionDeclaration") -> do
            let name = case lookup "name" pairs of { Just (JStr n) -> n; _ -> "" }
            let args = case lookup "args" pairs of { Just (JArr a) -> a; _ -> [] }
            let retTy = case lookup "returnType" pairs of { Just (JStr t) -> t; _ -> "i64" }
            
            -- Extract argument types
            let argTys = map (\a -> case getAttr "type" a of { Just (JStr t) -> t; _ -> "any" }) args
            
            let info = SymbolInfo "fn" "fn" (length args) argTys retTy
            st' <- defineSymbol name info st
            
            -- Enter function scope
            let stInner = (enterScope st') { currentFn = Just retTy }
            stArgs <- foldM (\s a -> case (getAttr "name" a, getAttr "type" a) of 
                                        (Just (JStr n), Just (JStr t)) -> defineSymbol n (SymbolInfo t "var" 0 [] "") s
                                        (Just (JStr n), _) -> defineSymbol n (SymbolInfo "i64" "var" 0 [] "") s
                                        _ -> Right s) stInner args
            
            let body = case lookup "body" pairs of { Just (JArr b) -> b; _ -> [] }
            _ <- foldM analyzeNode stArgs body
            return st'

        Just (JStr "VariableDeclaration") -> do
            let name = case lookup "name" pairs of { Just (JStr n) -> n; _ -> "" }
            -- FIX: Do NOT use "type" field here as it is the node type!
            let declaredTy = case lookup "varType" pairs of { Just (JStr t) -> t; _ -> "unknown" }
            
            let expr = case lookup "expr" pairs of { Just e -> e; _ -> case lookup "value" pairs of { Just v -> v; _ -> JNull } }
            let inferred = inferType st expr
            let finalTy = if declaredTy == "unknown" then inferred else declaredTy
            
            stVar <- defineSymbol name (SymbolInfo finalTy "var" 0 [] "") st
            _ <- analyzeNode stVar expr
            return stVar

        Just (JStr "BinaryExpression") -> do
            let left = case lookup "left" pairs of { Just l -> l; _ -> JNull }
            let right = case lookup "right" pairs of { Just r -> r; _ -> JNull }
            let op = case lookup "op" pairs of { Just (JStr o) -> o; _ -> "" }
            let leftTy = inferType st left
            let rightTy = inferType st right
            if leftTy /= rightTy && leftTy /= "unknown" && rightTy /= "unknown"
                then Left $ SemanticError E0308 ("mismatched types: expected `" ++ leftTy ++ "`, found `" ++ rightTy ++ "`") (leftTy ++ " " ++ op ++ " " ++ rightTy) ("expected `" ++ leftTy ++ "` because of this operator") ("ensure both operands have the same type")
                else do
                    _ <- analyzeNode st left
                    _ <- analyzeNode st right
                    return st

        Just (JStr "ComparisonExpression") -> do
            let left = case lookup "left" pairs of { Just l -> l; _ -> JNull }
            let right = case lookup "right" pairs of { Just r -> r; _ -> JNull }
            let op = case lookup "op" pairs of { Just (JStr o) -> o; _ -> "" }
            let leftTy = inferType st left
            let rightTy = inferType st right
            if leftTy /= rightTy && leftTy /= "unknown" && rightTy /= "unknown"
                then Left $ SemanticError E0308 ("mismatched types in comparison") (leftTy ++ " " ++ op ++ " " ++ rightTy) ("operands must be of the same type") ("ensure both sides of the comparison have the same type")
                else do
                    _ <- analyzeNode st left
                    _ <- analyzeNode st right
                    return st

        Just (JStr "Atomic") ->
            case lookup "value" pairs of
                Just (JStr val) -> 
                    if null val || val == "?" || val == ";" || isDefinedAnywhere val st || "\"" `isPrefixOf` val || all (\c -> isDigit c || (c == '-' && length val > 1)) val
                        then Right st
                        else Left $ SemanticError E0425 ("cannot find value `" ++ val ++ "` in this scope") ("let x = " ++ val ++ ";") ("not found in this scope") ("check the spelling or ensure the variable is defined before use")
                _ -> Right st

        Just (JStr "CallExpression") -> do
            let name = case lookup "name" pairs of { Just (JStr n) -> n; _ -> "" }
            case getSymbol name st of
                Just info -> do
                    let args = case lookup "args" pairs of { Just (JArr a) -> a; _ -> [] }
                    let expectedCount = symArgs info
                    let actualCount = case lookup "expr" pairs of { Just _ -> 1; _ -> length args }
                    
                    if expectedCount /= actualCount && name /= "print" && name /= "io::print"
                        then Left $ SemanticError E0061 ("this function takes " ++ show expectedCount ++ " arguments but " ++ show actualCount ++ " were supplied") (name ++ "(...)") ("expected " ++ show expectedCount ++ " arguments") ("provide the correct number of arguments defined in the function signature")
                        else do
                            -- Check Argument Types
                            let argNodes = case lookup "expr" pairs of { Just e -> [e]; _ -> args }
                            let argTypes = map (inferType st) argNodes
                            
                            case checkArgTypes (symArgTys info) argTypes of
                                Left (expected, found) -> 
                                     Left $ SemanticError E0308 ("mismatched types in function argument") (name ++ "(...)") ("expected `" ++ expected ++ "`, found `" ++ found ++ "`") ("pass an argument of type `" ++ expected ++ "`")
                                Right () -> do
                                    stArgs <- case lookup "args" pairs of
                                        Just (JArr a) -> foldM analyzeNode st a
                                        _ -> return st
                                    case lookup "expr" pairs of
                                        Just e -> analyzeNode stArgs e
                                        _ -> return stArgs

                Nothing -> Left $ SemanticError E0425 ("cannot find function `" ++ name ++ "` in this scope") (name ++ "(...)") ("not found in this scope") ("ensure the function name is correct and accessible")

        Just (JStr "ReturnStatement") -> do
            let expr = case lookup "expr" pairs of { Just e -> e; _ -> JNull }
            let inferred = inferType st expr
            case currentFn st of
                Just expected -> 
                    if inferred /= expected && inferred /= "unknown" && expected /= "void"
                        then Left $ SemanticError E0308 ("mismatched types in return statement") ("return " ++ inferred ++ ";") ("expected `" ++ expected ++ "`, found `" ++ inferred ++ "`") ("change the return expression to match the function's return type")
                        else analyzeNode st expr
                Nothing -> analyzeNode st expr

        _ -> do
            let childNodes = [v | (k, v) <- pairs, k `elem` ["left", "right", "expr", "condition", "args", "body", "value", "then_branch", "else_branch"]]
            foldM analyzeNode st childNodes

analyzeNode st (JArr nodes) = foldM analyzeNode st nodes
analyzeNode st _ = Right st

main :: IO ()
main = do
    args <- getArgs
    case args of
        (path:_) -> do
            content <- readFile path
            let (json, _) = parseJson content
            case analyzeNode initialTable json of
                Left err -> do
                    hPutStrLn stderr $ formatRustError err
                    putStrLn $ "{\"error\": \"" ++ errMsg err ++ "\"}"
                    hFlush stdout
                    exitFailure
                Right _  -> putStrLn content
        _ -> putStrLn "{\"error\": \"No input provided\"}"