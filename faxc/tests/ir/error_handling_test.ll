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

define i64 @validate(i64 %arg_n) {
entry:
    %full_frame = alloca %struct.GCFrame
    %frame = getelementptr %struct.GCFrame, %struct.GCFrame* %full_frame, i32 0, i32 0
    %roots_alloca = getelementptr %struct.GCFrame, %struct.GCFrame* %full_frame, i32 0, i32 1
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
    %atmp0 = add i64 %arg_n, 0
    %cmp1 = icmp slt i64 %atmp0, 0
    %zext1 = zext i1 %cmp1 to i64
    %cond_bit2 = icmp ne i64 %zext1, 0
    br i1 %cond_bit2, label %then2, label %end2

then2:
        %res_raw3 = call i8* @fax_gc_alloc(i64 24, i64* null, i64 0)
        %res_cast3 = bitcast i8* %res_raw3 to %struct.Result*
        %tag_ptr3 = getelementptr %struct.Result, %struct.Result* %res_cast3, i32 0, i32 0
        store i64 1, i64* %tag_ptr3
        %val_ptr3 = getelementptr %struct.Result, %struct.Result* %res_cast3, i32 0, i32 2
        store i64 404, i64* %val_ptr3
        %cast_i644 = ptrtoint i8* %res_raw3 to i64
        call void @fax_gc_pop_frame()
        ret i64 %cast_i644
        br label %end2

end2:
    %atmp5 = add i64 %arg_n, 0
    %bin6 = mul i64 %atmp5, 2
    %res_raw7 = call i8* @fax_gc_alloc(i64 24, i64* null, i64 0)
    %res_cast7 = bitcast i8* %res_raw7 to %struct.Result*
    %tag_ptr7 = getelementptr %struct.Result, %struct.Result* %res_cast7, i32 0, i32 0
    store i64 0, i64* %tag_ptr7
    %val_ptr7 = getelementptr %struct.Result, %struct.Result* %res_cast7, i32 0, i32 1
    store i64 %bin6, i64* %val_ptr7
    %cast_i648 = ptrtoint i8* %res_raw7 to i64
    call void @fax_gc_pop_frame()
    ret i64 %cast_i648
    call void @fax_gc_pop_frame()
    ret i64 0
}

define i64 @try_example(i64 %arg_n) {
entry:
    %full_frame = alloca %struct.GCFrame
    %frame = getelementptr %struct.GCFrame, %struct.GCFrame* %full_frame, i32 0, i32 0
    %roots_alloca = getelementptr %struct.GCFrame, %struct.GCFrame* %full_frame, i32 0, i32 1
    %val_9_ptr = alloca i64
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
    %atmp10 = add i64 %arg_n, 0
    %res11 = call i64 @validate(i64 %atmp10)
    store i64 %res11, i64* %val_9_ptr
    %tmp12 = load i64, i64* %val_9_ptr
    %ign13 = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %tmp12)
    %tmp14 = load i64, i64* %val_9_ptr
    %res_raw15 = call i8* @fax_gc_alloc(i64 24, i64* null, i64 0)
    %res_cast15 = bitcast i8* %res_raw15 to %struct.Result*
    %tag_ptr15 = getelementptr %struct.Result, %struct.Result* %res_cast15, i32 0, i32 0
    store i64 0, i64* %tag_ptr15
    %val_ptr15 = getelementptr %struct.Result, %struct.Result* %res_cast15, i32 0, i32 1
    store i64 %tmp14, i64* %val_ptr15
    %cast_i6416 = ptrtoint i8* %res_raw15 to i64
    call void @fax_gc_pop_frame()
    ret i64 %cast_i6416
    call void @fax_gc_pop_frame()
    ret i64 0
}

define i64 @main() {
entry:
    call void @fax_gc_init()
    %full_frame = alloca %struct.GCFrame
    %frame = getelementptr %struct.GCFrame, %struct.GCFrame* %full_frame, i32 0, i32 0
    %roots_alloca = getelementptr %struct.GCFrame, %struct.GCFrame* %full_frame, i32 0, i32 1
    %res1_17_ptr = alloca i64
    %res2_18_ptr = alloca i64
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
    %ign19 = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 111)
    %res20 = call i64 @try_example(i64 10)
    store i64 %res20, i64* %res1_17_ptr
    %ign21 = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 222)
    %un22 = sub i64 0, 5
    %res23 = call i64 @try_example(i64 %un22)
    store i64 %res23, i64* %res2_18_ptr
    %ign24 = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 333)
    call void @fax_gc_pop_frame()
    ret i64 0
}