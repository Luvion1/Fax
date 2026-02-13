module Sema.Types where

import qualified Data.Map as Map
import Sema.Diag (Diag(..), Severity(..), Type(..), SemanticError(..), toDiag)
import Data.List (isPrefixOf)

data JVal = JStr String | JNum String | JBool Bool | JNull | JObj [(String, JVal)] | JArr [JVal] deriving (Show, Eq)

data SymInfo = SymInfo 
    { sType :: Type
    , sMut :: Bool
    , sUsed :: Bool 
    , sLine :: Int
    , sCol :: Int
    } deriving (Show, Eq)

type Sym = String
type Scopes = [Map.Map Sym SymInfo]
type StructFields = Map.Map String Type
type StructDefs = Map.Map String StructFields

data Context = Context 
    { scopes :: Scopes
    , structs :: StructDefs
    , diagnostics :: [Diag]
    , retType :: Type 
    , inLoop :: Bool
    , terminated :: Bool
    }

emptyCtx :: Context
emptyCtx = Context [Map.empty] Map.empty [] TVoid False False

addDiag :: Diag -> Context -> Context
addDiag d c = c { diagnostics = diagnostics c ++ [d] }

enter :: Context -> Context
enter c = c { scopes = Map.empty : scopes c }

exitScopes :: Context -> (Context, [(Sym, Int, Int)])
exitScopes c =
    let (s:ss) = scopes c
        unused = Map.toList $ Map.filter (\i -> not (sUsed i) && sType i /= TUnk) s
        unusedInfo = map (\(n, i) -> (n, sLine i, sCol i)) unused
    in (c { scopes = ss }, unusedInfo)

defSym :: Sym -> Type -> Bool -> Int -> Int -> Context -> Context
defSym s t m l col c = 
    let (h:t') = scopes c 
        isDuplicate = Map.member s h
        isShadowing = any (Map.member s) t'
        c' = if isDuplicate 
             then addDiag (toDiag l col (DuplicateSymbol s)) c
             else if isShadowing
                  then addDiag (toDiag l col (ShadowingWarning s)) c { scopes = Map.insert s (SymInfo t m False l col) h : t' }
                  else c { scopes = Map.insert s (SymInfo t m False l col) h : t' }
    in c'

defStruct :: String -> StructFields -> Context -> Context
defStruct n fs c = 
    if Map.member n (structs c)
    then addDiag (toDiag 0 0 (DuplicateSymbol n)) c
    else c { structs = Map.insert n fs (structs c) }

markUsed :: Sym -> Context -> Context
markUsed s c =
    let upd [] = []
        upd (h:t) = if Map.member s h then Map.adjust (\i -> i { sUsed = True }) s h : t else h : upd t
    in c { scopes = upd (scopes c) }

lookupSym :: Sym -> Context -> Maybe SymInfo
lookupSym s c = foldr (\m acc -> Map.lookup s m `orElse` acc) Nothing (scopes c)
  where orElse (Just x) _ = Just x
        orElse Nothing y = y

lookupStruct :: String -> Context -> Maybe StructFields
lookupStruct n c = Map.lookup n (structs c)

isPointerTy :: Type -> Bool
isPointerTy (TStruct _) = True
isPointerTy (TArr _) = True
isPointerTy TStr = True
isPointerTy (TPtr _) = True
isPointerTy _ = False

typesCompatible :: Type -> Type -> Bool
typesCompatible TUnk _ = True
typesCompatible _ TUnk = True
typesCompatible TNull t2 = isPointerTy t2
typesCompatible t1 TNull = isPointerTy t1
typesCompatible (TArr t1) (TArr t2) = typesCompatible t1 t2
typesCompatible (TPtr t1) (TPtr t2) = typesCompatible t1 t2
typesCompatible (TTup ts1) (TTup ts2) = length ts1 == length ts2 && all (\(a, b) -> typesCompatible a b) (zip ts1 ts2)
typesCompatible t1 t2 = t1 == t2

strToType :: String -> Type
strToType "i64" = TI64
strToType "int" = TI64
strToType "bool" = TBool
strToType "str" = TStr
strToType "string" = TStr
strToType "void" = TVoid
strToType s | null s = TUnk
            | "&mut " `isPrefixOf` s = TPtr (strToType (drop 5 s))
            | "&" `isPrefixOf` s = TPtr (strToType (drop 1 s))
            | last s == ']' = TArr (strToType (init (init s)))
            | otherwise = TStruct s

-- Type inference functions (merged from TypeInference.hs)
type Subst = Map.Map String Type

applySubst :: Subst -> Type -> Type
applySubst _ TI64 = TI64
applySubst _ TBool = TBool
applySubst _ TStr = TStr
applySubst _ TVoid = TVoid
applySubst _ TNull = TNull
applySubst s (TArr t) = TArr (applySubst s t)
applySubst s (TPtr t) = TPtr (applySubst s t)
applySubst s (TTup ts) = TTup (map (applySubst s) ts)
applySubst s (TFn args ret) = TFn (map (applySubst s) args) (applySubst s ret)
applySubst _ t@(TStruct _) = t
applySubst _ TUnk = TUnk

unify :: Type -> Type -> Either String Subst
unify t1 t2 | t1 == t2 = Right Map.empty
unify (TFn a1 r1) (TFn a2 r2) = do
    if length a1 /= length a2
        then Left "Function arity mismatch"
        else do
            s1 <- unifyList a1 a2
            s2 <- unify (applySubst s1 r1) (applySubst s1 r2)
            return $ composeSubst s2 s1
unify (TArr t1) (TArr t2) = unify t1 t2
unify (TPtr t1) (TPtr t2) = unify t1 t2
unify (TTup ts1) (TTup ts2) = 
    if length ts1 /= length ts2
        then Left "Tuple length mismatch"
        else unifyList ts1 ts2
unify t1 t2 = Left $ "Cannot unify " ++ show t1 ++ " with " ++ show t2

unifyList :: [Type] -> [Type] -> Either String Subst
unifyList [] [] = Right Map.empty
unifyList (t1:ts1) (t2:ts2) = do
    s1 <- unify t1 t2
    s2 <- unifyList (map (applySubst s1) ts1) (map (applySubst s1) ts2)
    return $ composeSubst s2 s1
unifyList _ _ = Left "Type list length mismatch"

composeSubst :: Subst -> Subst -> Subst
composeSubst s1 s2 = Map.map (applySubst s1) s2 `Map.union` s1

isNumeric :: Type -> Bool
isNumeric TI64 = True
isNumeric TBool = True
isNumeric _ = False

inferBin :: String -> Type -> Type -> Either String Type
inferBin op t1 t2
    | op `elem` ["add", "sub", "mul", "sdiv", "srem"] = 
        if isNumeric t1 && isNumeric t2
            then Right $ if t1 == TI64 || t2 == TI64 then TI64 else TBool
            else Left "Arithmetic requires numeric types"
    | op `elem` ["eq", "ne", "slt", "sgt", "sle", "sge"] = 
        if isNumeric t1 && isNumeric t2
            then Right TBool
            else Left "Comparison requires numeric types"
    | op `elem` ["land", "lor"] = 
        if isNumeric t1 && isNumeric t2
            then Right TBool
            else Left "Logic requires boolean types"
    | op == "add" && (t1 == TStr || t2 == TStr) = Right TStr
    | otherwise = Left $ "Unknown operator: " ++ op

canCoerce :: Type -> Type -> Bool
canCoerce TUnk _ = True
canCoerce _ TUnk = True
canCoerce TNull t = isPointerTy t
canCoerce t TNull = isPointerTy t
canCoerce t1 t2 | t1 == t2 = True
canCoerce t1 t2 = isNumeric t1 && isNumeric t2

commonType :: Type -> Type -> Type
commonType t1 t2 
    | t1 == t2 = t1
    | isNumeric t1 && isNumeric t2 = TI64
    | otherwise = TUnk
