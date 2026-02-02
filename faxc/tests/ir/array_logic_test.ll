; Fax Codegen
target triple = "x86_64-pc-linux-gnu"

%struct.StackFrame = type { %struct.StackFrame*, i8**, i64 }
%struct.Result = type { i32, i32, i32 }

declare void @fax_gc_init()
declare i8* @fax_gc_alloc(i64, i64*, i64)
declare void @fax_gc_collect()
declare void @fax_gc_push_frame(%struct.StackFrame*)
declare void @fax_gc_pop_frame()
declare i32 @printf(i8*, ...)

@fmt_int = private unnamed_addr constant [4 x i8] c"%d\0A\00"

define i32 @main() {
entry:
    call void @fax_gc_init()
    %frame = alloca %struct.StackFrame
    %roots_alloca = alloca [8 x i8*]
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
    %arr_ptr_raw = call i8* @fax_gc_alloc(i64 16, i64* null, i64 0)
    %arr_ptr = alloca i8*
    store i8* %arr_ptr_raw, i8** %arr_ptr
    %slot0 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 0
    store i8* %arr_ptr_raw, i8** %slot0
    %arr_acast = bitcast i8* %arr_ptr_raw to i32*
    %gep0 = getelementptr i32, i32* %arr_acast, i32 0
    store i32 10, i32* %gep0
    %gep1 = getelementptr i32, i32* %arr_acast, i32 1
    store i32 20, i32* %gep1
    %gep2 = getelementptr i32, i32* %arr_acast, i32 2
    store i32 30, i32* %gep2
    %gep3 = getelementptr i32, i32* %arr_acast, i32 3
    store i32 40, i32* %gep3
    %idx_ptr = alloca i32
    store i32 2, i32* %idx_ptr
    %tmp4 = load i32, i32* %idx_ptr
    %arr_ptr_raw5 = load i8*, i8** %arr_ptr
    %arr_ptr5 = bitcast i8* %arr_ptr_raw5 to i32*
    %gep5 = getelementptr i32, i32* %arr_ptr5, i32 %tmp4
    %val5 = load i32, i32* %gep5
    %val_ptr = alloca i32
    store i32 %val5, i32* %val_ptr
    %tmp6 = load i32, i32* %val_ptr
    call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @fmt_int, i32 0, i32 0), i32 %tmp6)
    %a_ptr = alloca i32
    store i32 1, i32* %a_ptr
    %b_ptr = alloca i32
    store i32 0, i32* %b_ptr
    %tmp7 = load i32, i32* %a_ptr
    %tmp8 = load i32, i32* %b_ptr
    %log9 = and i32 %tmp7, %tmp8
    %cond_bit10 = icmp ne i32 %log9, 0
    br i1 %cond_bit10, label %then10, label %else10

then10:
        call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @fmt_int, i32 0, i32 0), i32 1)
        br label %end10

else10:
        call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @fmt_int, i32 0, i32 0), i32 0)
        br label %end10

end10:
    %tmp11 = load i32, i32* %a_ptr
    %tmp12 = load i32, i32* %b_ptr
    %log13 = or i32 %tmp11, %tmp12
    %cond_bit14 = icmp ne i32 %log13, 0
    br i1 %cond_bit14, label %then14, label %end14

then14:
        call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @fmt_int, i32 0, i32 0), i32 99)
        br label %end14

end14:
    call void @fax_gc_pop_frame()
    ret i32 0
}