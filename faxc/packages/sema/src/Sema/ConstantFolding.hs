module Sema.ConstantFolding where

import Sema.Types
import Sema.ASTUtils (lookupField, getJStr)

data ConstValue = CInt Integer | CBool Bool | CStr String | CNull | CUnknown
    deriving (Show, Eq)

evaluateConst :: JVal -> ConstValue
evaluateConst node =
    case lookupField "type" node of
        Just (JStr "NumberLiteral") -> 
            case lookupField "value" node of
                Just (JNum n) -> case reads n of
                    [(v, "")] -> CInt v
                    _ -> CUnknown
                _ -> CUnknown
        Just (JStr "Boolean") ->
            case lookupField "value" node of
                Just (JBool b) -> CBool b
                _ -> CUnknown
        Just (JStr "StringLiteral") ->
            case lookupField "value" node of
                Just (JStr s) -> CStr s
                _ -> CUnknown
        Just (JStr "NullLiteral") -> CNull
        Just (JStr "UnaryExpression") ->
            let operand = evaluateConst $ case lookupField "operand" node of
                    Just v -> v
                    Nothing -> JObj []
                op = case lookupField "op" node of
                    Just (JStr o) -> o
                    _ -> ""
            in evalUnary op operand
        Just (JStr "BinaryExpression") ->
            let left = evaluateConst $ case lookupField "left" node of
                    Just v -> v
                    Nothing -> JObj []
                right = evaluateConst $ case lookupField "right" node of
                    Just v -> v
                    Nothing -> JObj []
                op = case lookupField "op" node of
                    Just (JStr o) -> o
                    _ -> ""
            in evalBinary op left right
        Just (JStr "ComparisonExpression") ->
            let left = evaluateConst $ case lookupField "left" node of
                    Just v -> v
                    Nothing -> JObj []
                right = evaluateConst $ case lookupField "right" node of
                    Just v -> v
                    Nothing -> JObj []
                op = case lookupField "op" node of
                    Just (JStr o) -> o
                    _ -> ""
            in evalComparison op left right
        Just (JStr "IfExpression") ->
            let cond = evaluateConst $ case lookupField "condition" node of
                    Just v -> v
                    Nothing -> JObj []
            in case cond of
                CBool True -> evalBranch node "then_branch"
                CBool False -> evalBranch node "else_branch"
                _ -> CUnknown
        _ -> CUnknown

evalUnary :: String -> ConstValue -> ConstValue
evalUnary "neg" (CInt n) = CInt (-n)
evalUnary "not" (CBool b) = CBool (not b)
evalUnary "not" (CInt 0) = CBool True
evalUnary "not" (CInt _) = CBool False
evalUnary _ _ = CUnknown

evalBinary :: String -> ConstValue -> ConstValue -> ConstValue
evalBinary "add" (CInt a) (CInt b) = CInt (a + b)
evalBinary "sub" (CInt a) (CInt b) = CInt (a - b)
evalBinary "mul" (CInt a) (CInt b) = CInt (a * b)
evalBinary "sdiv" (CInt a) (CInt b) 
    | b /= 0 = CInt (a `div` b)
    | otherwise = CUnknown
evalBinary "srem" (CInt a) (CInt b)
    | b /= 0 = CInt (a `mod` b)
    | otherwise = CUnknown
evalBinary "add" (CStr a) (CStr b) = CStr (a ++ b)
evalBinary "add" (CStr a) (CInt b) = CStr (a ++ show b)
evalBinary "add" (CInt a) (CStr b) = CStr (show a ++ b)
evalBinary _ _ _ = CUnknown

evalComparison :: String -> ConstValue -> ConstValue -> ConstValue
evalComparison "eq" (CInt a) (CInt b) = CBool (a == b)
evalComparison "ne" (CInt a) (CInt b) = CBool (a /= b)
evalComparison "slt" (CInt a) (CInt b) = CBool (a < b)
evalComparison "sgt" (CInt a) (CInt b) = CBool (a > b)
evalComparison "sle" (CInt a) (CInt b) = CBool (a <= b)
evalComparison "sge" (CInt a) (CInt b) = CBool (a >= b)
evalComparison "eq" (CBool a) (CBool b) = CBool (a == b)
evalComparison "ne" (CBool a) (CBool b) = CBool (a /= b)
evalComparison "eq" (CStr a) (CStr b) = CBool (a == b)
evalComparison "ne" (CStr a) (CStr b) = CBool (a /= b)
evalComparison _ _ _ = CUnknown

evalBranch :: JVal -> String -> ConstValue
evalBranch node field =
    case lookupField field node of
        Just (JArr stmts) | not (null stmts) -> evaluateConst (last stmts)
        _ -> CUnknown

isConstant :: JVal -> Bool
isConstant node = case evaluateConst node of
    CUnknown -> False
    _ -> True

getConstInt :: JVal -> Maybe Integer
getConstInt node = case evaluateConst node of
    CInt n -> Just n
    _ -> Nothing

getConstBool :: JVal -> Maybe Bool
getConstBool node = case evaluateConst node of
    CBool b -> Just b
    CInt 0 -> Just False
    CInt _ -> Just True
    _ -> Nothing

checkDivZero :: JVal -> Maybe String
checkDivZero node =
    case lookupField "type" node of
        Just (JStr "BinaryExpression") ->
            case lookupField "op" node of
                Just (JStr op) | op `elem` ["sdiv", "srem"] ->
                    case lookupField "right" node >>= getConstInt of
                        Just 0 -> Just "Division by zero"
                        _ -> Nothing
                _ -> Nothing
        _ -> Nothing

checkOverflow :: JVal -> Maybe String
checkOverflow node =
    case lookupField "type" node of
        Just (JStr "BinaryExpression") ->
            let left = getConstInt =<< lookupField "left" node
                right = getConstInt =<< lookupField "right" node
                op = case lookupField "op" node of
                    Just (JStr o) -> o
                    _ -> ""
                maxInt = 9223372036854775807
                minInt = -9223372036854775808
            in case (left, right, op) of
                (Just a, Just b, "add") -> 
                    let r = a + b
                    in if r > maxInt || r < minInt
                        then Just "Integer overflow in addition"
                        else Nothing
                (Just a, Just b, "mul") ->
                    let r = a * b
                    in if r > maxInt || r < minInt
                        then Just "Integer overflow in multiplication"
                        else Nothing
                _ -> Nothing
        _ -> Nothing

simplifyExpr :: JVal -> JVal
simplifyExpr node =
    case lookupField "type" node of
        Just (JStr "BinaryExpression") ->
            case evaluateConst node of
                CInt n -> JObj [("type", JStr "NumberLiteral"), ("value", JNum (show n))]
                CBool b -> JObj [("type", JStr "Boolean"), ("value", JBool b)]
                _ -> node
        Just (JStr "UnaryExpression") ->
            case evaluateConst node of
                CInt n -> JObj [("type", JStr "NumberLiteral"), ("value", JNum (show n))]
                CBool b -> JObj [("type", JStr "Boolean"), ("value", JBool b)]
                _ -> node
        _ -> node

checkConstCondition :: JVal -> Maybe (Bool, String)
checkConstCondition node =
    case getConstBool node of
        Just True -> Just (True, "Always true")
        Just False -> Just (False, "Always false")
        _ -> Nothing
