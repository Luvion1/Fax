module Sema.ASTUtils where

import Sema.Types

-- Helper functions for working with AST nodes
lookupField :: String -> JVal -> Maybe JVal
lookupField k (JObj fs) = lookup k fs
lookupField _ _ = Nothing

getJStr :: JVal -> String
getJStr (JStr s) = s
getJStr _ = ""

getLoc :: JVal -> (Int, Int)
getLoc v = case lookupField "loc" v of
    Just (JObj o) ->
        let l = case lookup "line" o of { Just (JNum n) -> read n; _ -> 0 }
            c = case lookup "col" o of { Just (JNum n) -> read n; _ -> 0 }
        in (l, c)
    _ -> (0, 0)
