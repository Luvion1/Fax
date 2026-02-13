module Sema.Pretty where

import Sema.Diagnostics
import Data.List (intercalate)

formatDiag :: String -> String -> Diag -> String
formatDiag filename source d =
    let header = (if sev d == Error then "\ESC[1;31merror\ESC[0m" else "\ESC[1;33mwarning\ESC[0m") 
                 ++ "\ESC[1m[" ++ code d ++ "]: " ++ msg d ++ "\ESC[0m"
        locInfo = "  \ESC[1;34m-->\ESC[0m " ++ filename ++ ":" ++ show (line d) ++ ":" ++ show (col d)
        
        sourceLines = lines source
        padding = replicate (length (show (line d))) ' '
        
        snippet = if line d > 0 && line d <= length sourceLines
                  then padding ++ " \ESC[1;34m|\ESC[0m\n" ++
                       show (line d) ++ " \ESC[1;34m|\ESC[0m " ++ (sourceLines !! (line d - 1)) ++ "\n" ++
                       padding ++ " \ESC[1;34m|\ESC[0m " ++ replicate (col d - 1) ' ' 
                       ++ (if sev d == Error then "\ESC[1;31m^" else "\ESC[1;33m^")
                       ++ "--- " ++ msg d ++ "\ESC[0m"
                  else ""
    in header ++ "\n" ++ locInfo ++ "\n" ++ snippet

prettyPrintDiags :: String -> String -> [Diag] -> String
prettyPrintDiags filename source diags = 
    intercalate "\n\n" $ map (formatDiag filename source) diags