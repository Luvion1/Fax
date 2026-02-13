module Sema.Diagnostics where

import System.IO (hPutStrLn, stderr)

data Severity = Error | Warning deriving (Show, Eq)

data Diag = Diag 
    { sev :: Severity
    , code :: String
    , msg :: String
    , line :: Int
    , col :: Int
    } deriving (Show, Eq)

report :: [Diag] -> IO ()
report = mapM_ (\d -> hPutStrLn stderr $ "[" ++ show (sev d) ++ " " ++ code d ++ "] " ++ msg d)
