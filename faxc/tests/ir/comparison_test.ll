; Fax Codegen
target triple = "x86_64-pc-linux-gnu"

%struct.StackFrame = type { %struct.StackFrame*, i8**, i64 }
%struct.Result = type { i64, i64, i64 }

declare void @fax_gc_init()
declare i8* @fax_gc_alloc(i64, i64*, i64)
declare void @fax_gc_collect()
declare void @fax_gc_push_frame(%struct.StackFrame*)
declare void @fax_gc_pop_frame()
declare i64 @printf(i8*, ...)

@fmt_int = private unnamed_addr constant [5 x i8] c"%ld\0A\00"
@fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"

define i64 @main() {
entry:
    call void @fax_gc_init()
    %frame = alloca %struct.StackFrame
    %roots_alloca = alloca [8 x i8*]
    %result_0_ptr = alloca i64
    %result2_1_ptr = alloca i64
    %frame_next = getelementptr %struct.StackFrame, %struct.StackFrame* %frame, i32 0, i32 0
    store %struct.StackFrame* null, %struct.StackFrame** %frame_next
    %frame_roots_gep = getelementptr %struct.StackFrame, %struct.StackFrame* %frame, i32 0, i32 1
    %roots_ptr = bitcast [8 x i8*]* %roots_alloca to i8**
    store i8** %roots_ptr, i8*** %frame_roots_gep
    %frame_count = getelementptr %struct.StackFrame, %struct.StackFrame* %frame, i32 0, i32 2
    store i64 8, i64* %frame_count
    %rinit0 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 0
    store i8* null, i8** %rinit0
    %rinit1 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 1
    store i8* null, i8** %rinit1
    %rinit2 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 2
    store i8* null, i8** %rinit2
    %rinit3 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 3
    store i8* null, i8** %rinit3
    %rinit4 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 4
    store i8* null, i8** %rinit4
    %rinit5 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 5
    store i8* null, i8** %rinit5
    %rinit6 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 6
    store i8* null, i8** %rinit6
    %rinit7 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 7
    store i8* null, i8** %rinit7
    call void @fax_gc_push_frame(%struct.StackFrame* %frame)
    %bin2 = add i64 5, 5
    %cmp3 = icmp eq i64 10, %bin2
    %zext3 = zext i1 %cmp3 to i64
    store i64 %zext3, i64* %result_0_ptr
    %tmp4 = load i64, i64* %result_0_ptr
    call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %tmp4)
    %cmp5 = icmp sgt i64 5, 3
    %zext5 = zext i1 %cmp5 to i64
    %cmp6 = icmp sgt i64 2, 1
    %zext6 = zext i1 %cmp6 to i64
    %cmp7 = icmp eq i64 %zext5, %zext6
    %zext7 = zext i1 %cmp7 to i64
    store i64 %zext7, i64* %result2_1_ptr
    %tmp8 = load i64, i64* %result2_1_ptr
    call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %tmp8)
    call void @fax_gc_pop_frame()
    ret i64 0
}