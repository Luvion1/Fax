module Sema.Api where

import Sema.Types
import Sema.Checker
import Sema.Diagnostics
import Sema.Pretty (prettyPrintDiags)
import Text.ParserCombinators.Parsec
import Control.Monad (void)
import System.Environment
import System.Exit
import System.IO (hPutStrLn, stderr, readFile)
import Control.Exception (try, SomeException)
import qualified Control.Exception
import qualified Data.List as L

ws :: Parser ()
ws = skipMany (space <|> newline <|> tab)

pVal :: Parser JVal
pVal = ws >> (pStr <|> pNum <|> pObj <|> pArr <|> pBool <|> pNull)

pStr :: Parser JVal
pStr = JStr <$> (char '"' >> manyTill ((char '\\' >> anyChar) <|> noneOf "\"") (char '"'))

pNum :: Parser JVal
pNum = JNum <$> many1 (digit <|> oneOf ".-eE")

pBool :: Parser JVal
pBool = (string "true" >> return (JBool True)) <|> (string "false" >> return (JBool False))

pNull :: Parser JVal
pNull = string "null" >> return JNull

pObj :: Parser JVal
pObj = JObj <$> (char '{' >> ws >> sepBy pPair (ws >> char ',' >> ws) <* (ws >> char '}'))

pPair :: Parser (String, JVal)
pPair = do { ws; JStr k <- pStr; ws; void $ char ':'; ws; v <- pVal; return (k, v) }

pArr :: Parser JVal
pArr = JArr <$> (char '[' >> ws >> sepBy pVal (ws >> char ',' >> ws) <* (ws >> char ']'))

pAST :: String -> Either ParseError JVal
pAST = parse (ws >> (pObj <|> pArr) <* ws) "AST"

encodeJVal :: JVal -> String
encodeJVal (JStr s) = "\"" ++ concatMap escape s ++ "\""
  where escape '"' = "\\\""
        escape '\\' = "\\\\"
        escape '\n' = "\\n"
        escape c = [c]
encodeJVal (JNum n) = n
encodeJVal (JBool True) = "true"
encodeJVal (JBool False) = "false"
encodeJVal JNull = "null"
encodeJVal (JArr xs) = "[" ++ L.intercalate "," (map encodeJVal xs) ++ "]"
encodeJVal (JObj fs) = "{" ++ L.intercalate "," (map (\(k, v) -> "\"" ++ k ++ "\":" ++ encodeJVal v) fs) ++ "}"

run :: FilePath -> IO ()
run path = do
  content <- readFile path
  case pAST content of
    Left err -> hPutStrLn stderr ("Error: Could not parse input JSON: " ++ show err) >> exitFailure
    Right ast -> do
      let stmts = case ast of
                    JObj fs -> maybe [ast] (\(JArr a) -> a) (lookup "body" fs)
                    JArr a -> a
                    _ -> [ast]
      
      let (diags, _) = checkProgram emptyCtx stmts
      let isSuccess = not $ any (\d -> sev d == Error) diags
      
      let sourcePath = case ast of
                         JObj fs -> case lookup "source_file" fs of
                                      Just (JStr s) -> s
                                      _ -> "input.fax"
                         _ -> "input.fax"
      
      sourceResult <- Control.Exception.try (readFile sourcePath) :: IO (Either SomeException String)
      let source = case sourceResult of
                     Left _ -> ""
                     Right c -> c
      
      if not isSuccess
        then do
          hPutStrLn stderr $ prettyPrintDiags sourcePath source diags
          exitFailure
        else do
          let warnings = filter (\d -> sev d == Warning) diags
          if not (null warnings) 
            then hPutStrLn stderr $ prettyPrintDiags sourcePath source warnings
            else return ()
          putStr $ encodeJVal ast
