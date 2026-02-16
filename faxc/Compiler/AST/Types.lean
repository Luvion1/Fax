import Compiler.AST.Types.Literal
import Compiler.AST.Types.Operators
import Compiler.AST.Types.Basic
import Compiler.AST.Types.Utils

namespace Compiler.AST

alias Lit := Types.Literal.Literal
alias UnOp := Types.UnaryOp
alias BinOp := Types.BinaryOp
alias Ty := Types.Ty

-- Re-export utility functions
export Types.Ty (isNumeric isInteger isFloat isReference sizeOf isCompatible commonSupertype toString matches canBeDefaultInit)

end Compiler.AST
