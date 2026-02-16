/-
Codegen Protobuf Converters
Converts between Lean Codegen types and Protobuf message types
-/

import Compiler.Codegen.Proto.Converters.IR
import Compiler.Codegen.Proto.Converters.Types

namespace Compiler.Codegen.Proto.Converters

-- Re-export all codegen converters
export IR (CodegenType.toProto Ty.toCodegenType
           LLVMValue.toProto LLVMInstruction.toProto
           LLVMFunction.toProto LLVMModule.toProto
           Module.toLLVMModule)
export Types

end Compiler.Codegen.Proto.Converters
