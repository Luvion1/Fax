module Main where

import System.Environment (getArgs)
import System.Exit (exitFailure)
import System.IO (hPutStrLn, stderr)
import Sema.Api (run)

main :: IO ()
main = do
    args <- getArgs
    case args of
        [path] -> run path
        _ -> do
            hPutStrLn stderr "Usage: sema <input.json>"
            exitFailure