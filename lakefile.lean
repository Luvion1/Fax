import Lake
open Lake DSL

-- Package configuration
package fax where
  srcDir := "faxc"
  -- Add C++ build configuration
  moreLinkArgs := #[
    "-L/root/Fax/build/lib",
    "-lfax_proto",
    "-lprotobuf",
    "-lpthread"
  ]

-- Lean library
lean_lib Compiler where
  roots := #[`Compiler]

-- Executable
lean_exe faxc where
  root := `Compiler.Driver
  moreLinkArgs := #[
    "-L/root/Fax/build/lib",
    "-lfax_proto",
    "-lprotobuf",
    "-lpthread",
    "-Wl,-rpath,/root/Fax/build/lib"
  ]

-- External library for protobuf FFI
extern_lib fax_proto where
  -- Build C++ protobuf wrapper
  let buildDir := "build"
  let libDir := buildDir / "lib"
  let objDir := buildDir / "obj"
  let protoDir := "faxc" / "Compiler" / "Proto"
  let generatedDir := protoDir / "Generated"
  
  -- Source files
  let ffiCpp := protoDir / "ffi.cpp"
  let ffiO := objDir / "ffi.o"
  
  -- Create directories
  IO.FS.createDirAll libDir
  IO.FS.createDirAll objDir
  IO.FS.createDirAll generatedDir
  
  -- Compile C++ to object file
  let compileCmd := #[
    "g++", "-c", "-fPIC", "-O2",
    "-I" ++ (protoDir.toString),
    "-I" ++ (generatedDir.toString),
    "-I/usr/include",
    ffiCpp.toString,
    "-o", ffiO.toString
  ]
  
  -- Create shared library
  let linkCmd := #[
    "g++", "-shared", "-o",
    (libDir / "libfax_proto.so").toString,
    ffiO.toString,
    "-lprotobuf"
  ]
  
  -- Execute commands
  let compileRes ← IO.Process.run { cmd := compileCmd.head!, args := compileCmd.tail! }
  let linkRes ← IO.Process.run { cmd := linkCmd.head!, args := linkCmd.tail! }
  
  return (libDir / "libfax_proto.so")

-- Target to generate protobuf C++ code
target protobuf_gen : Unit := do
  let protoDir := "proto"
  let genDir := "faxc" / "Compiler" / "Proto" / "Generated"
  
  IO.FS.createDirAll genDir
  
  -- Generate protobuf C++ code
  let protoFiles := #[
    protoDir / "common.proto",
    protoDir / "types.proto",
    protoDir / "literal.proto",
    protoDir / "pattern.proto",
    protoDir / "token.proto",
    protoDir / "expr.proto",
    protoDir / "decl.proto",
    protoDir / "compiler.proto"
  ]
  
  let protocCmd := #["protoc", "--cpp_out=" ++ genDir.toString, "--proto_path=" ++ protoDir.toString] ++ protoFiles.map toString
  
  let _ ← IO.Process.run { cmd := protocCmd.head!, args := protocCmd.tail! }
  
  return ()
