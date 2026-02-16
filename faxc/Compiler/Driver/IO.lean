namespace Compiler.Driver.IO

def readFile (path : String) : IO String := do
  let handle ← IO.FS.Handle.mk path IO.FS.Mode.read false
  let content ← handle.getToEnd
  return content

def writeFile (path : String) (content : String) : IO Unit :=
  IO.FS.writeFile path content

end Compiler.Driver.IO
