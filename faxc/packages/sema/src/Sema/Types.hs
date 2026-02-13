module Sema.Types where

import qualified Data.Map as Map
import Sema.Diagnostics
import Sema.Errors

data JVal = JStr String | JNum String | JBool Bool | JNull | JObj [(String, JVal)] | JArr [JVal] deriving (Show, Eq)

data Type = TI64 | TBool | TStr | TVoid | TNull | TUnk | TFn [Type] Type | TStruct String | TArr Type deriving (Show, Eq)

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
    }

emptyCtx :: Context
emptyCtx = Context [Map.empty] Map.empty [] TVoid False

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
    in if Map.member s h 
       then addDiag (toDiag l col (DuplicateSymbol s)) c
       else c { scopes = Map.insert s (SymInfo t m False l col) h : t' }

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
isPointerTy _ = False

typesCompatible :: Type -> Type -> Bool
typesCompatible TUnk _ = True
typesCompatible _ TUnk = True
typesCompatible TNull t2 = isPointerTy t2
typesCompatible t1 TNull = isPointerTy t1
typesCompatible t1 t2 = t1 == t2

strToType :: String -> Type
strToType "i64" = TI64
strToType "bool" = TBool
strToType "str" = TStr
strToType "string" = TStr
strToType "void" = TVoid
strToType s | null s = TUnk
            | last s == ']' = TArr (strToType (init (init s)))
            | otherwise = TStruct s