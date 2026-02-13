module Sema.Checker where

import qualified Data.Map as Map
import Data.Char (isDigit)
import Sema.Types
import Sema.Diagnostics
import Sema.Errors

lookupField :: String -> JVal -> Maybe JVal
lookupField k (JObj fs) = lookup k fs
lookupField _ _ = Nothing

getLoc :: JVal -> (Int, Int)
getLoc v = case lookupField "loc" v of
    Just (JObj o) -> 
        let l = case lookup "line" o of { Just (JNum n) -> read n; _ -> 0 }
            c = case lookup "col" o of { Just (JNum n) -> read n; _ -> 0 }
        in (l, c)
    _ -> (0, 0)

checkProgram :: Context -> [JVal] -> ([Diag], Context)
checkProgram ctx stmts =
    let ctxWithGlobals = foldl registerGlobal ctx stmts
        finalCtx = foldl (\c n -> snd $ check c n) ctxWithGlobals stmts
    in (Sema.Types.diagnostics finalCtx, finalCtx)

registerGlobal :: Context -> JVal -> Context
registerGlobal ctx node = 
    let (l, c) = getLoc node
    in case lookupField "type" node of
        Just (JStr "StructDeclaration") ->
            let name = maybe "anon" (\(JStr s) -> s) (lookupField "name" node)
                fields = maybe [] (\(JArr l') -> l') (lookupField "fields" node)
                fieldMap = Map.fromList $ map (\f -> (maybe "" (\(JStr s) -> s) (lookupField "name" f), maybe TUnk (\(JStr s) -> strToType s) (lookupField "type" f))) fields
            in defStruct name fieldMap ctx
        Just (JStr "FunctionDeclaration") ->
            let name = maybe "anon" (\(JStr s) -> s) (lookupField "name" node)
                retTy = strToType $ maybe "void" (\(JStr s) -> s) (lookupField "returnType" node)
                args = maybe [] (\(JArr l') -> l') (lookupField "args" node)
                argTys = map (\a -> maybe TUnk (\(JStr s) -> strToType s) (lookupField "p_type" a)) args
            in defSym name (TFn argTys retTy) False l c ctx
        _ -> ctx

check :: Context -> JVal -> (Type, Context)
check ctx node = case lookupField "type" node of
    Just (JStr "FunctionDeclaration") -> (TVoid, checkFunc ctx node)
    Just (JStr "VariableDeclaration") -> (TVoid, checkVar ctx node)
    Just (JStr "IfStatement") -> (TVoid, checkIfStmt ctx node)
    Just (JStr "WhileStatement") -> (TVoid, checkWhileStmt ctx node)
    Just (JStr "ForStatement") -> (TVoid, checkForStmt ctx node)
    Just (JStr "ReturnStatement") -> (TVoid, checkRet ctx node)
    Just (JStr "BreakStatement") -> (TVoid, checkBrk ctx node)
    Just (JStr "ContinueStatement") -> (TVoid, checkCont ctx node)
    Just (JStr "CallExpression") -> checkCall ctx node
    Just (JStr "Assignment") -> (TVoid, checkAssign ctx node)
    Just (JStr "BinaryExpression") -> checkBin ctx node
    Just (JStr "MemberAccess") -> checkMem ctx node
    Just (JStr "IndexAccess") -> checkIdx ctx node
    Just (JStr "StructLiteral") -> checkStructLit ctx node
    Just (JStr "ArrayLiteral") -> checkArrLit ctx node
    Just (JStr "ComparisonExpression") -> (TBool, snd $ checkBin ctx node)
    Just (JStr "StringLiteral") -> (TStr, ctx)
    Just (JStr "NumberLiteral") -> (TI64, ctx)
    Just (JStr "NullLiteral") -> (TNull, ctx)
    Just (JStr "Identifier") ->
        let name = maybe "" (\(JStr val) -> val) (lookupField "value" node)
        in case lookupSym name ctx of
            Just info -> (sType info, markUsed name ctx)
            Nothing -> (TUnk, reportError node (UndefinedSymbol name) ctx)
    Just (JStr "Block") -> (TVoid, checkBlk ctx node)
    _ -> (TUnk, ctx)

reportError :: JVal -> SemanticError -> Context -> Context
reportError node err ctx = 
    let (l, c) = getLoc node
    in addDiag (toDiag l c err) ctx

exitScope :: Context -> Context
exitScope ctx = 
    let (ctx', unused) = exitScopes ctx
        warnings = map (\(n, l, c) -> toDiag l c (UnusedVariable n)) unused
    in foldl (flip addDiag) ctx' warnings

checkFunc :: Context -> JVal -> Context
checkFunc ctx node =
    let retTy = strToType $ maybe "void" (\(JStr s) -> s) (lookupField "returnType" node)
        args = maybe [] (\(JArr l') -> l') (lookupField "args" node)
        argTys = map (\a -> maybe TUnk (\(JStr s) -> strToType s) (lookupField "p_type" a)) args
        ctx' = enter ctx { retType = retTy, inLoop = False }
        ctxWithArgs = foldl (\c (arg, ty) ->
            let (l, col) = getLoc arg
            in maybe c (\(JStr name) -> defSym name ty False l col c) (lookupField "name" arg)) ctx' (zip args argTys)
        body = maybe [] (\(JArr l') -> l') (lookupField "body" node)
        finalCtx = foldl (\c n -> snd $ check c n) ctxWithArgs body
    in exitScope finalCtx

checkBlk :: Context -> JVal -> Context
checkBlk ctx node =
    let body = maybe [] (\(JArr l') -> l') (lookupField "body" node)
        ctx' = foldl (\c n -> snd $ check c n) (enter ctx) body
    in exitScope ctx'

checkVar :: Context -> JVal -> Context
checkVar ctx node =
    let name = maybe "unk" (\(JStr s) -> s) (lookupField "name" node)
        isMut = case lookupField "is_mutable" node of { Just (JBool b) -> b; _ -> False }
        (l, c) = getLoc node
        (ty, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "expr" node)
        varTy = maybe TUnk strToType ((\(JStr s) -> if s == "" then Nothing else Just s) =<< lookupField "var_type" node)
        finalCtx = if not (typesCompatible varTy ty)
                   then reportError node (TypeMismatch name varTy ty) ctx'
                   else ctx'
    in defSym name (if varTy /= TUnk then varTy else ty) isMut l c finalCtx

checkIfStmt :: Context -> JVal -> Context
checkIfStmt ctx node =
    let (condTy, ctx') = maybe (TBool, ctx) (check ctx) (lookupField "condition" node)
        ctx'' = if condTy /= TBool && condTy /= TUnk then reportError node (ConditionMustBeBool "if") ctx' else ctx'
        thenBody = maybe [] (\(JArr l') -> l') (lookupField "then_branch" node)
        elseBody = maybe [] (\(JArr l') -> l') (lookupField "else_branch" node)
        ctxAfterThen = foldl (\c n -> snd $ check c n) ctx'' thenBody
        ctxAfterElse = foldl (\c n -> snd $ check c n) ctxAfterThen elseBody
    in ctxAfterElse

checkWhileStmt :: Context -> JVal -> Context
checkWhileStmt ctx node =
    let (condTy, ctx') = maybe (TBool, ctx) (check ctx) (lookupField "condition" node)
        ctx'' = if condTy /= TBool && condTy /= TUnk then reportError node (ConditionMustBeBool "while") ctx' else ctx'
        body = maybe [] (\(JArr l') -> l') (lookupField "body" node)
        finalCtx = foldl (\c n -> snd $ check (c { inLoop = True }) n) ctx'' body
    in finalCtx { inLoop = inLoop ctx'' }

checkForStmt :: Context -> JVal -> Context
checkForStmt ctx node =
    let name = maybe "i" (\(JStr s) -> s) (lookupField "var_name" node)
        (l, c) = getLoc node
        (startTy, ctx') = maybe (TI64, ctx) (check ctx) (lookupField "start" node)
        (endTy, ctx'') = maybe (TI64, ctx') (check ctx') (lookupField "end" node)
        ctx''' = if startTy /= TI64 || endTy /= TI64 then reportError node RangeBoundsMustBeI64 ctx'' else ctx''
        body = maybe [] (\(JArr l') -> l') (lookupField "body" node)
        loopCtx = defSym name TI64 False l c (enter ctx''')
        finalLoopCtx = foldl (\c n -> snd $ check (c { inLoop = True }) n) loopCtx body
        (ctxExit, _) = exitScopes finalLoopCtx
    in ctxExit { inLoop = inLoop ctx''' }

checkRet :: Context -> JVal -> Context
checkRet ctx node =
    let (ty, ctx') = maybe (TVoid, ctx) (check ctx) (lookupField "argument" node)
        expected = retType ctx'
    in if not (typesCompatible expected ty)
       then reportError node (ReturnTypeMismatch expected ty) ctx'
       else ctx'

checkBrk :: Context -> JVal -> Context
checkBrk ctx node = if inLoop ctx then ctx else reportError node BreakOutsideLoop ctx

checkCont :: Context -> JVal -> Context
checkCont ctx node = if inLoop ctx then ctx else reportError node ContinueOutsideLoop ctx

checkCall :: Context -> JVal -> (Type, Context)
checkCall ctx node =
    let name = maybe "unk" (\(JStr s) -> s) (lookupField "name" node)
        args = maybe [] (\(JArr l') -> l') (lookupField "args" node)
        (retTy, ctx') = if name `elem` ["print", "collect_gc"] then (TVoid, ctx)
                        else case lookupSym name ctx of
                             Just info -> case sType info of
                                 TFn tys r -> if length tys /= length args
                                              then (r, reportError node (ArgCountMismatch (length tys) (length args)) ctx)
                                              else (r, ctx)
                                 _ -> (TUnk, reportError node (NotAFunction name) ctx)
                             Nothing -> (TUnk, reportError node (UndefinedSymbol name) ctx)
    in (retTy, foldl (\c a -> snd $ check c a) ctx' args)

checkMem :: Context -> JVal -> (Type, Context)
checkMem ctx node =
    let (baseTy, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "base" node)
        field = maybe "" (\(JStr s) -> s) (lookupField "field" node)
    in case baseTy of
        TStruct name -> case lookupStruct name ctx' of
            Just fields -> case Map.lookup field fields of
                Just ty -> (ty, ctx')
                Nothing -> (TUnk, reportError node (FieldNotFound field name) ctx')
            Nothing -> (TUnk, reportError node (StructNotFound name) ctx')
        TUnk -> (TUnk, ctx')
        _ -> (TUnk, reportError node (NotAStruct baseTy) ctx')

checkIdx :: Context -> JVal -> (Type, Context)
checkIdx ctx node =
    let (baseTy, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "base" node)
        (idxTy, ctx'') = maybe (TI64, ctx') (check ctx') (lookupField "index" node)
    in if idxTy /= TI64 && idxTy /= TUnk then (TUnk, reportError node RangeBoundsMustBeI64 ctx'')
       else case baseTy of
           TArr ty -> (ty, ctx'')
           TUnk -> (TUnk, ctx'')
           _ -> (TUnk, reportError node (NotAnArray baseTy) ctx'')

checkStructLit :: Context -> JVal -> (Type, Context)
checkStructLit ctx node =
    let name = maybe "" (\(JStr s) -> s) (lookupField "name" node)
        fields = maybe [] (\(JArr l') -> l') (lookupField "fields" node)
        (ctxWithFields, actualFields) = foldl (\(c, acc) f ->
            let (ty, c') = check c (maybe JNull id (lookupField "expr" f))
                fieldName = maybe "" (\(JStr s) -> s) (lookupField "name" f)
            in (c', Map.insert fieldName ty acc)) (ctx, Map.empty) fields
    in case lookupStruct name ctxWithFields of
        Just expectedFields ->
            let missing = Map.keys $ Map.difference expectedFields actualFields
                extra = Map.keys $ Map.difference actualFields expectedFields
                mismatched = Map.keys $ Map.filterWithKey (\k t -> case Map.lookup k expectedFields of { Just et -> not (typesCompatible et t); _ -> False }) actualFields
                ctx1 = foldl (\c m -> reportError node (MissingField m) c) ctxWithFields missing
                ctx2 = foldl (\c e -> reportError node (ExtraField e) c) ctx1 extra
                ctx3 = foldl (\c m -> reportError node (TypeMismatch m (expectedFields Map.! m) (actualFields Map.! m)) c) ctx2 mismatched
            in (TStruct name, ctx3)
        Nothing -> (TUnk, reportError node (StructNotFound name) ctxWithFields)

checkArrLit :: Context -> JVal -> (Type, Context)
checkArrLit ctx node =
    let elements = maybe [] (\(JArr l') -> l') (lookupField "elements" node)
        (elemTy, ctx') = if null elements then (TUnk, ctx) else check ctx (head elements)
        finalCtx = foldl (\c e -> let (ty, c') = check c e in if not (typesCompatible elemTy ty) then reportError node ArrayTypeMismatch c' else c') ctx' (if null elements then [] else tail elements)
    in (TArr elemTy, finalCtx)

checkAssign :: Context -> JVal -> Context
checkAssign ctx node =
    let target = maybe JNull id (lookupField "target" node)
        (exprTy, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "expr" node)
    in case target of
        JObj fs | lookup "type" fs == Just (JStr "Identifier") ->
            let name = maybe "" (\(JStr val) -> val) (lookup "value" fs)
            in case lookupSym name ctx' of
                Just info -> if not (sMut info) then reportError target (ImmutableAssignment name) ctx'
                             else if not (typesCompatible (sType info) exprTy)
                                  then reportError target (TypeMismatch name (sType info) exprTy) ctx'
                                  else ctx'
                Nothing -> reportError target (UndefinedSymbol name) ctx'
        JStr name -> case lookupSym name ctx' of
            Just info -> if not (sMut info) then reportError target (ImmutableAssignment name) ctx'
                         else if not (typesCompatible (sType info) exprTy)
                              then reportError target (TypeMismatch name (sType info) exprTy) ctx'
                              else ctx'
            Nothing -> reportError target (UndefinedSymbol name) ctx'
        JObj _ -> let (targetTy, ctx'') = check ctx' target in
                  if not (typesCompatible targetTy exprTy)
                  then reportError target (TypeMismatch "assignment" targetTy exprTy) ctx''
                  else ctx''
        _ -> ctx'

checkBin :: Context -> JVal -> (Type, Context)
checkBin ctx node =
    let (leftTy, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "left" node)
        (rightTy, ctx'') = maybe (TUnk, ctx') (check ctx') (lookupField "right" node)
        op = maybe "" (\(JStr s) -> s) (lookupField "op" node)
    in (if leftTy == TStr && rightTy == TStr && op == "add" then TStr else leftTy,
        if not (typesCompatible leftTy rightTy)
        then reportError node (TypeMismatch "operator" leftTy rightTy) ctx''
        else ctx'')