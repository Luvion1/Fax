; Fax Codegen
target triple = "x86_64-pc-linux-gnu"

%struct.StackFrame = type { %struct.StackFrame*, i8**, i64 }
%struct.GCFrame = type { %struct.StackFrame, [8 x i8*] }
%struct.Result = type { i64, i64, i64 }

declare void @fax_gc_init()
declare i8* @fax_gc_alloc(i64, i64*, i64)
declare void @fax_gc_collect()
declare void @fax_gc_push_frame(%struct.StackFrame*)
declare void @fax_gc_pop_frame()
declare i64 @printf(i8*, ...)

define void @Std_io_collect_gc() {
entry:
    call void @fax_gc_collect()
    ret void
}

@fmt_int = private unnamed_addr constant [5 x i8] c"%ld\0A\00"
@fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"