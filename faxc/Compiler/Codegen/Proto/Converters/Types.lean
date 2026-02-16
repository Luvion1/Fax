/-
Codegen Types Protobuf Converters
-/

import Compiler.Codegen.Types
import Compiler.Proto.Messages

namespace Compiler.Codegen.Proto.Converters.Types

open Compiler.Codegen.Types
open Compiler.Proto.Messages

-- Export IR converters
export Compiler.Codegen.Proto.Converters.IR (CodegenType.toProto Ty.toCodegenType
  LLVMValue.toProto LLVMInstruction.toProto
  LLVMFunction.toProto LLVMModule.toProto
  Module.toLLVMModule)

end Compiler.Codegen.Proto.Converters.Types
