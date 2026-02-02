; Fax Compiler Output
target triple = "aarch64-unknown-linux-gnu"

%struct.User = type { i32, i32 }

define i32 @main() {
entry:
    %x = alloca i32
    store i32 0, i32* %x
    %isActive = alloca i1
    store i1 0, i1* %isActive
    ; Call to Std.IO.println.
    ret i32 0
}