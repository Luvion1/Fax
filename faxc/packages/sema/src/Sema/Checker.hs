module Sema.Checker where

import qualified Data.Map as Map
import Data.Char (isDigit)
import Sema.Types
import Sema.Diag (Diag(..), Severity(..), toDiag, SemanticError(..), Type(..))
import Sema.Errors (Suggestion, getSuggestion)
import Sema.ControlFlow (analyzeFunction, findUnreachableCode, checkExhaustiveness, checkRedundantPatterns)
import Sema.ASTUtils (lookupField, getJStr, getLoc)

-- isConstantTrue and isConstantFalse moved to ControlFlow module
isConstantTrue :: JVal -> Bool
isConstantTrue node = case lookupField "type" node of
    Just (JStr "Boolean") -> case lookupField "value" node of { Just (JBool b) -> b; _ -> False }
    Just (JStr "NumberLiteral") -> case lookupField "value" node of { Just (JNum n) -> n /= "0"; _ -> False }
    _ -> False

isConstantFalse :: JVal -> Bool
isConstantFalse node = case lookupField "type" node of
    Just (JStr "Boolean") -> case lookupField "value" node of { Just (JBool b) -> not b; _ -> False }
    Just (JStr "NumberLiteral") -> case lookupField "value" node of { Just (JNum n) -> n == "0"; _ -> False }
    Just (JStr "NullLiteral") -> True
    _ -> False

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
            let name = maybe "anon" getJStr (lookupField "name" node)
                fields = maybe [] (\(JArr l') -> l') (lookupField "fields" node)
                fieldMap = Map.fromList $ map (\f -> (maybe "" getJStr (lookupField "name" f), maybe TUnk (strToType . getJStr) (lookupField "type" f))) fields
            in defStruct name fieldMap ctx
        Just (JStr "FunctionDeclaration") ->
            let name = maybe "anon" getJStr (lookupField "name" node)
                retTy = strToType $ maybe "void" getJStr (lookupField "returnType" node)
                args = maybe [] (\(JArr l') -> l') (lookupField "args" node)
                argTys = map (\a -> maybe TUnk (strToType . getJStr) (lookupField "p_type" a)) args
            in defSym name (TFn argTys retTy) False l c ctx
        _ -> ctx

check :: Context -> JVal -> (Type, Context)
check ctx node = 
    if terminated ctx 
    then (TUnk, reportError node UnreachableCode ctx { terminated = False }) 
    else case lookupField "type" node of
    Just (JStr "FunctionDeclaration") -> (TVoid, checkFunc ctx node)
    Just (JStr "VariableDeclaration") -> (TVoid, checkVar ctx node)
    Just (JStr "IfStatement") -> (TVoid, checkIfStmt ctx node)
    Just (JStr "WhileStatement") -> (TVoid, checkWhileStmt ctx node)
    Just (JStr "ForStatement") -> (TVoid, checkForStmt ctx node)
    Just (JStr "ReturnStatement") -> (TVoid, (snd $ checkRet ctx node) { terminated = True })
    Just (JStr "BreakStatement") -> (TVoid, (checkBrk ctx node) { terminated = True })
    Just (JStr "ContinueStatement") -> (TVoid, (checkCont ctx node) { terminated = True })
    Just (JStr "CallExpression") -> checkCall ctx node
    Just (JStr "Assignment") -> (TVoid, checkAssign ctx node)
    Just (JStr "IfExpression") -> checkIfExpr ctx node
    Just (JStr "MatchExpression") -> checkMatch ctx node
    Just (JStr "UnaryExpression") -> checkUnary ctx node
    Just (JStr "ReferenceExpression") -> checkRef ctx node
    Just (JStr "DereferenceExpression") -> checkDeref ctx node
    Just (JStr "BinaryExpression") -> checkBin ctx node
    Just (JStr "TupleLiteral") -> checkTup ctx node
    Just (JStr "MemberAccess") -> checkMem ctx node
    Just (JStr "IndexAccess") -> checkIdx ctx node
    Just (JStr "StructLiteral") -> checkStructLit ctx node
    Just (JStr "ArrayLiteral") -> checkArrLit ctx node
    Just (JStr "ComparisonExpression") -> (TBool, snd $ checkBin ctx node)
    Just (JStr "StringLiteral") -> (TStr, ctx)
    Just (JStr "NumberLiteral") -> (TI64, ctx)
    Just (JStr "Boolean") -> (TBool, ctx)
    Just (JStr "NullLiteral") -> (TNull, ctx)
    Just (JStr "Identifier") ->
        let name = maybe "" getJStr (lookupField "value" node)
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
    let retTy = strToType $ maybe "void" getJStr (lookupField "returnType" node)
        args = maybe [] (\(JArr l') -> l') (lookupField "args" node)
        argTys = map (\a -> maybe TUnk (strToType . getJStr) (lookupField "p_type" a)) args
        ctx' = enter ctx { retType = retTy, inLoop = False, terminated = False }
        ctxWithArgs = foldl (\c (arg, ty) ->
            let (l, col) = getLoc arg
            in maybe c (\nameVal -> defSym (getJStr nameVal) ty False l col c) (lookupField "name" arg)) ctx' (zip args argTys)
        body = maybe [] (\(JArr l') -> l') (lookupField "body" node)
        finalCtx = foldl (\c n -> snd $ check c n) ctxWithArgs body
        -- Check for unreachable code
        unreachable = findUnreachableCode body
        ctxWithUnreachable = foldl (\c (l, col, stmtType) -> 
            addDiag (toDiag l col (UnreachableCodeAfter stmtType)) c) finalCtx unreachable
        -- Check for missing returns
        ctxWithReturnCheck = analyzeFunction node ctxWithUnreachable
    in (exitScope ctxWithReturnCheck) { terminated = False }

checkBlk :: Context -> JVal -> Context
checkBlk ctx node =
    let body = maybe [] (\(JArr l') -> l') (lookupField "body" node)
        ctx' = foldl (\c n -> snd $ check c n) (enter ctx) body
        (ctxAfter, unused) = exitScopes ctx'
        warnings = map (\(n, l, c) -> toDiag l c (UnusedVariable n)) unused
    in (foldl (flip addDiag) ctxAfter warnings) { terminated = terminated ctx' }

checkVar :: Context -> JVal -> Context
checkVar ctx node =
    let name = maybe "unk" getJStr (lookupField "name" node)
        isMut = case lookupField "is_mutable" node of { Just (JBool b) -> b; _ -> False }
        (l, c) = getLoc node
        (ty, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "expr" node)
        varTyStr = maybe "" getJStr (lookupField "var_type" node)
        varTy = if null varTyStr then TUnk else strToType varTyStr
        finalCtx = if varTy /= TUnk && not (typesCompatible varTy ty)
                   then reportError node (TypeMismatch name varTy ty) ctx'
                   else ctx'
    in defSym name (if varTy /= TUnk then varTy else ty) isMut l c finalCtx

checkIfStmt :: Context -> JVal -> Context
checkIfStmt ctx node =
    let condNode = maybe JNull id (lookupField "condition" node)
        (condTy, ctx') = check ctx condNode
        ctx'' = if condTy /= TBool && condTy /= TI64 && condTy /= TUnk then reportError node (ConditionMustBeBool "if") ctx' else ctx'
        
        -- Constant Condition Warning
        ctxWithConstWarn = if isConstantTrue condNode then reportError condNode (ConstantCondition "if" True) ctx''
                           else if isConstantFalse condNode then reportError condNode (ConstantCondition "if" False) ctx''
                           else ctx''
                           
        thenBody = maybe [] (\(JArr l') -> l') (lookupField "then_branch" node)
        elseBody = maybe [] (\(JArr l') -> l') (lookupField "else_branch" node)
        
        ctxAfterThen = if isConstantFalse condNode then enter ctxWithConstWarn else foldl (\c n -> snd $ check c n) (enter ctxWithConstWarn) thenBody
        ctxAfterElse = if isConstantTrue condNode then enter (exitScope ctxAfterThen) { terminated = False } else foldl (\c n -> snd $ check c n) (enter (exitScope ctxAfterThen) { terminated = False }) elseBody
        
        finalTerminated = (isConstantTrue condNode && terminated ctxAfterThen) || 
                          (isConstantFalse condNode && terminated ctxAfterElse) ||
                          (terminated ctxAfterThen && terminated ctxAfterElse)
    in (exitScope ctxAfterElse) { terminated = finalTerminated }

checkWhileStmt :: Context -> JVal -> Context
checkWhileStmt ctx node =
    let condNode = maybe JNull id (lookupField "condition" node)
        (condTy, ctx') = check ctx condNode
        ctx'' = if condTy /= TBool && condTy /= TI64 && condTy /= TUnk then reportError node (ConditionMustBeBool "while") ctx' else ctx'
        
        -- Infinite Loop & Constant Condition Detection
        isAlwaysTrue = isConstantTrue condNode
        isAlwaysFalse = isConstantFalse condNode
        ctxWithWarn = if isAlwaysTrue then reportError condNode InfiniteLoopDetected (reportError condNode (ConstantCondition "while" True) ctx'')
                      else if isAlwaysFalse then reportError condNode (ConstantCondition "while" False) ctx''
                      else ctx''
                      
        body = maybe [] (\(JArr l') -> l') (lookupField "body" node)
        finalCtx = if isAlwaysFalse then enter ctxWithWarn else foldl (\c n -> snd $ check (c { inLoop = True }) n) (enter ctxWithWarn) body
        
        -- If it's an infinite loop without a break, any code after it is unreachable
        -- Note: Simple approximation, doesn't check for 'break' inside body yet
        finalTerminated = isAlwaysTrue 
    in (exitScope finalCtx) { inLoop = inLoop ctx'', terminated = finalTerminated }

checkForStmt :: Context -> JVal -> Context
checkForStmt ctx node =
    let name = maybe "i" getJStr (lookupField "var_name" node)
        (l, c) = getLoc node
        (startTy, ctx') = maybe (TI64, ctx) (check ctx) (lookupField "start" node)
        (endTy, ctx'') = maybe (TI64, ctx') (check ctx') (lookupField "end" node)
        ctx''' = if startTy /= TI64 || endTy /= TI64 then reportError node RangeBoundsMustBeI64 ctx'' else ctx''
        body = maybe [] (\(JArr l') -> l') (lookupField "body" node)
        loopCtx = defSym name TI64 False l c (enter ctx''')
        finalLoopCtx = foldl (\c n -> snd $ check (c { inLoop = True }) n) loopCtx body
    in (exitScope finalLoopCtx) { inLoop = inLoop ctx''', terminated = False }

checkRet :: Context -> JVal -> (Type, Context)
checkRet ctx node =
    let (ty, ctx') = maybe (TVoid, ctx) (check ctx) (lookupField "argument" node)
        expected = retType ctx'
    in if not (typesCompatible expected ty)
       then (ty, reportError node (ReturnTypeMismatch expected ty) ctx')
       else (ty, ctx')

checkBrk :: Context -> JVal -> Context
checkBrk ctx node = if inLoop ctx then ctx else reportError node BreakOutsideLoop ctx

checkCont :: Context -> JVal -> Context
checkCont ctx node = if inLoop ctx then ctx else reportError node ContinueOutsideLoop ctx

checkCall :: Context -> JVal -> (Type, Context)
checkCall ctx node =
    let name = maybe "unk" getJStr (lookupField "name" node)
        args = maybe [] (\(JArr l') -> l') (lookupField "args" node)
        (retTy, ctx', expectedTys) = if name == "len" then (TI64, ctx, [TUnk])
                        else if name `elem` ["print", "collect_gc"] then (TVoid, ctx, [])
                        else case lookupSym name ctx of
                             Just info -> case sType info of
                                 TFn tys r -> (r, ctx, tys)
                                 _ -> (TUnk, reportError node (NotAFunction name) ctx, [])
                             Nothing -> (TUnk, reportError node (UndefinedSymbol name) ctx, [])
        
        ctxWithCountCheck = if length expectedTys > 0 && length expectedTys /= length args && name /= "len"
                            then reportError node (ArgCountMismatch (length expectedTys) (length args)) ctx'
                            else ctx'
                            
        finalCtx = foldl (\c (arg, idx) -> 
            let (argTy, c') = check c arg
            in if idx < length expectedTys && not (typesCompatible (expectedTys !! idx) argTy)
               then reportError arg (ArgTypeMismatch idx (expectedTys !! idx) argTy) c'
               else c'
            ) ctxWithCountCheck (zip args [0..])
    in (retTy, finalCtx)

checkMem :: Context -> JVal -> (Type, Context)
checkMem ctx node =
    let (baseTy, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "base" node)
        field = maybe "" getJStr (lookupField "field" node)
    in case baseTy of
        TStruct name -> case lookupStruct name ctx' of
            Just fields -> case Map.lookup field fields of
                Just ty -> (ty, ctx')
                Nothing -> (TUnk, reportError node (FieldNotFound field name) ctx')
            Nothing -> (TUnk, reportError node (StructNotFound name) ctx')
        TTup tys ->
            if all isDigit field
            then let idx = read field :: Int
                 in if idx < length tys then (tys !! idx, ctx')
                    else (TUnk, reportError node (FieldNotFound field "tuple") ctx')
            else (TUnk, reportError node (FieldNotFound field "tuple") ctx')
        TUnk -> (TUnk, ctx')
        _ -> (TUnk, reportError node (NotAStruct baseTy) ctx')

checkIdx :: Context -> JVal -> (Type, Context)
checkIdx ctx node =
    let (baseTy, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "base" node)
        (idxTy, ctx'') = maybe (TI64, ctx') (check ctx') (lookupField "index" node)
        resCtx = if idxTy /= TI64 && idxTy /= TUnk then reportError node IndexMustBeI64 ctx'' else ctx''
    in case baseTy of
        TArr ty -> (ty, resCtx)
        TStr -> (TI64, resCtx)
        TUnk -> (TUnk, resCtx)
        _ -> (TUnk, reportError node (NotAnArray baseTy) resCtx)

checkStructLit :: Context -> JVal -> (Type, Context)
checkStructLit ctx node =
    let name = maybe "" getJStr (lookupField "name" node)
        fields = maybe [] (\(JArr l') -> l') (lookupField "fields" node)
        (ctxWithFields, actualFields) = foldl (\(c, acc) f ->
            let (ty, c') = check c (maybe JNull id (lookupField "expr" f))
                fieldName = maybe "" getJStr (lookupField "name" f)
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
            let name = maybe "" getJStr (lookup "value" fs)
            in case lookupSym name ctx' of
                Just info -> 
                    let ctx'' = if not (sMut info) then reportError target (ImmutableAssignment name) ctx' else ctx'
                    in if not (typesCompatible (sType info) exprTy)
                       then reportError target (TypeMismatch name (sType info) exprTy) ctx''
                       else markUsed name ctx''
                Nothing -> reportError target (UndefinedSymbol name) ctx'
        JStr name -> case lookupSym name ctx' of
            Just info -> 
                let ctx'' = if not (sMut info) then reportError target (ImmutableAssignment name) ctx' else ctx'
                in if not (typesCompatible (sType info) exprTy)
                   then reportError target (TypeMismatch name (sType info) exprTy) ctx''
                   else markUsed name ctx''
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
        op = maybe "" getJStr (lookupField "op" node)
    in if op == "land" || op == "lor"
       then let ctx1 = if leftTy /= TBool && leftTy /= TI64 && leftTy /= TUnk then reportError node (TypeMismatch op TBool leftTy) ctx'' else ctx''
                ctx2 = if rightTy /= TBool && rightTy /= TI64 && rightTy /= TUnk then reportError node (TypeMismatch op TBool rightTy) ctx1 else ctx1
            in (TBool, ctx2)
       else (if leftTy == TStr && rightTy == TStr && op == "add" then TStr
             else if op == "srem" then TI64
             else leftTy,
             if not (typesCompatible leftTy rightTy)
             then reportError node (TypeMismatch "operator" leftTy rightTy) ctx''
             else ctx'')

checkIfExpr :: Context -> JVal -> (Type, Context)
checkIfExpr ctx node =
    let (condTy, ctx') = maybe (TBool, ctx) (check ctx) (lookupField "condition" node)
        ctx'' = if condTy /= TBool && condTy /= TI64 && condTy /= TUnk then reportError node (ConditionMustBeBool "if") ctx' else ctx'
        thenBody = maybe [] (\(JArr l') -> l') (lookupField "then_branch" node)
        elseBody = maybe [] (\(JArr l') -> l') (lookupField "else_branch" node)
        
        -- Check then branch in separate scope
        (thenTy, ctxAfterThen) = 
            let (t, c) = foldl (\(accT, curC) n -> check curC n) (TUnk, enter ctx'') thenBody
            in (t, exitScope c)
            
        -- Check else branch in separate scope starting from the same context ctx''
        (elseTy, ctxAfterElse) = 
            let (t, c) = foldl (\(accT, curC) n -> check curC n) (TUnk, enter ctxAfterThen { diagnostics = diagnostics ctxAfterThen }) elseBody
            in (t, exitScope c)
            
        -- Final context should have diagnostics from both branches but not their local symbols
        finalCtx = ctxAfterElse { diagnostics = diagnostics ctxAfterElse }
        
    in if not (typesCompatible thenTy elseTy)
       then (TUnk, reportError node (TypeMismatch "if-branch" thenTy elseTy) finalCtx)
       else (thenTy, finalCtx)

checkMatch :: Context -> JVal -> (Type, Context)
checkMatch ctx node =
    let (targetTy, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "target" node)
        cases = maybe [] (\(JArr l') -> l') (lookupField "cases" node)
        (resTy, finalCtx) = foldl (\(accTy, c) cNode ->
            let (patTy, c') = maybe (TUnk, c) (check c) (lookupField "pattern" cNode)
                body = maybe [] (\(JArr l') -> l') (lookupField "body" cNode)
                -- Isolate case body scope
                (bodyTy, c'') = 
                    let (t, bc) = foldl (\(bt, curC) n -> check curC n) (TUnk, enter c') body
                    in (t, exitScope bc)
                newC = if not (typesCompatible targetTy patTy) then reportError cNode (TypeMismatch "match-pattern" targetTy patTy) c'' else c''
            in (if accTy == TUnk then bodyTy else if not (typesCompatible accTy bodyTy) then TUnk else accTy, newC)) (TUnk, ctx') cases
        defaultBody = maybe [] (\(JArr l') -> l') (lookupField "default" node)
        (defTy, finalCtxWithDefault) = if null defaultBody then (resTy, finalCtx) 
            else let (t, c) = foldl (\(accT, curC) n -> check curC n) (TUnk, enter finalCtx) defaultBody
                 in (t, exitScope c)
        
        -- Check pattern exhaustiveness
        ctxWithPatternCheck = checkExhaustiveness node finalCtxWithDefault
        -- Check for redundant patterns
        ctxWithRedundantCheck = checkRedundantPatterns cases ctxWithPatternCheck
    in if resTy /= TUnk && defTy /= TUnk && not (typesCompatible resTy defTy)
       then (TUnk, reportError node (TypeMismatch "match-result" resTy defTy) ctxWithRedundantCheck)
       else (if resTy /= TUnk then resTy else defTy, ctxWithRedundantCheck)

checkUnary :: Context -> JVal -> (Type, Context)
checkUnary ctx node =
    let (operandTy, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "operand" node)
        op = maybe "" getJStr (lookupField "op" node)
    in if op == "not"
       then if operandTy /= TBool && operandTy /= TI64 && operandTy /= TUnk
            then (TBool, reportError node (TypeMismatch "not" TBool operandTy) ctx')
            else (TBool, ctx')
       else if op == "neg"
            then if operandTy /= TI64 && operandTy /= TUnk
                 then (TI64, reportError node (TypeMismatch "neg" TI64 operandTy) ctx')
                 else (TI64, ctx')
            else (TUnk, ctx')

checkRef :: Context -> JVal -> (Type, Context)
checkRef ctx node =
    let (ty, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "operand" node)
    in (TPtr ty, ctx')

checkDeref :: Context -> JVal -> (Type, Context)
checkDeref ctx node =
    let (ty, ctx') = maybe (TUnk, ctx) (check ctx) (lookupField "operand" node)
    in case ty of
        TPtr t -> (t, ctx')
        TUnk -> (TUnk, ctx')
        _ -> (TUnk, reportError node (NotAStruct ty) ctx')

checkTup :: Context -> JVal -> (Type, Context)
checkTup ctx node =
    let elements = maybe [] (\(JArr l') -> l') (lookupField "elements" node)
        (tys, finalCtx) = foldl (\(ts, c) e -> let (t, c') = check c e in (ts ++ [t], c')) ([], ctx) elements
    in (TTup tys, finalCtx)
