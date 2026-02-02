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
%struct.Node = type { i64 }

define i64 @leak_checker(i64 %arg_n) {
entry:
    %full_frame = alloca %struct.GCFrame
    %frame = getelementptr %struct.GCFrame, %struct.GCFrame* %full_frame, i32 0, i32 0
    %roots_alloca = getelementptr %struct.GCFrame, %struct.GCFrame* %full_frame, i32 0, i32 1
    %node_0_ptr = alloca i8*
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
    %node_0_ptr_raw = call i8* @fax_gc_alloc(i64 8, i64* null, i64 0)
    store i8* %node_0_ptr_raw, i8** %node_0_ptr
    %slot0 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 0
    store i8* %node_0_ptr, i8** %slot0
    %node_0_cast = bitcast i8* %node_0_ptr_raw to %struct.Node*
    %atmp1 = add i64 %arg_n, 0
    %gep2 = getelementptr %struct.Node, %struct.Node* %node_0_cast, i32 0, i32 0
    store i64 %atmp1, i64* %gep2
    %atmp3 = add i64 %arg_n, 0
    %cmp4 = icmp sle i64 %atmp3, 0
    %zext4 = zext i1 %cmp4 to i64
    %cond_bit5 = icmp ne i64 %zext4, 0
    br i1 %cond_bit5, label %then5, label %end5

then5:
        call void @Std_io_collect_gc()
        call void @fax_gc_pop_frame()
        ret i64 0
        br label %end5

end5:
    %ptr_raw7 = load i8*, i8** %node_0_ptr
    %cast8 = bitcast i8* %ptr_raw7 to %struct.Node*
    %gep8 = getelementptr %struct.Node, %struct.Node* %cast8, i32 0, i32 0
    %val8 = load i64, i64* %gep8
    %atmp9 = add i64 %arg_n, 0
    %bin10 = sub i64 %atmp9, 1
    %res11 = call i64 @leak_checker(i64 %bin10)
    %bin12 = add i64 %val8, %res11
    call void @fax_gc_pop_frame()
    ret i64 %bin12
    call void @fax_gc_pop_frame()
    ret i64 0
}

define i64 @main() {
entry:
    call void @fax_gc_init()
    %full_frame = alloca %struct.GCFrame
    %frame = getelementptr %struct.GCFrame, %struct.GCFrame* %full_frame, i32 0, i32 0
    %roots_alloca = getelementptr %struct.GCFrame, %struct.GCFrame* %full_frame, i32 0, i32 1
    %result_13_ptr = alloca i64
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
    %res14 = call i64 @leak_checker(i64 10)
    store i64 %res14, i64* %result_13_ptr
    %tmp15 = load i64, i64* %result_13_ptr
    %ign16 = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %tmp15)
    call void @fax_gc_pop_frame()
    ret i64 0
}