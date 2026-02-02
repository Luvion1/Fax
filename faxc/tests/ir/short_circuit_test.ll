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
    %x_0_ptr = alloca i64
    %res_1_ptr = alloca i64
    %y_2_ptr = alloca i64
    %res2_3_ptr = alloca i64
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
    store i64 0, i64* %x_0_ptr
    %log_res4 = alloca i64
    store i64 1, i64* %log_res4
    %lcond4 = icmp ne i64 1, 0
    br i1 %lcond4, label %log_end4, label %log_next4

log_next4:
        store i64 1, i64* %x_0_ptr
        %cmp5 = icmp eq i64 1, 1
        %zext5 = zext i1 %cmp5 to i64
        store i64 %zext5, i64* %log_res4
        br label %log_end4

log_end4:
    %res6 = load i64, i64* %log_res4
    store i64 %res6, i64* %res_1_ptr
    %tmp7 = load i64, i64* %x_0_ptr
    call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %tmp7)
    store i64 0, i64* %y_2_ptr
    %log_res8 = alloca i64
    store i64 0, i64* %log_res8
    %lcond8 = icmp ne i64 0, 0
    br i1 %lcond8, label %log_next8, label %log_end8

log_next8:
        store i64 1, i64* %y_2_ptr
        %cmp9 = icmp eq i64 1, 1
        %zext9 = zext i1 %cmp9 to i64
        store i64 %zext9, i64* %log_res8
        br label %log_end8

log_end8:
    %res10 = load i64, i64* %log_res8
    store i64 %res10, i64* %res2_3_ptr
    %tmp11 = load i64, i64* %y_2_ptr
    call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %tmp11)
    call void @fax_gc_pop_frame()
    ret i64 0
}