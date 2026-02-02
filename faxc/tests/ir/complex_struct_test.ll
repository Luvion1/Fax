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
%struct.Point = type { i64, i64 }
%struct.Rect = type { i64, i64 }

define i64 @main() {
entry:
    call void @fax_gc_init()
    %frame = alloca %struct.StackFrame
    %roots_alloca = alloca [8 x i8*]
    %p1_0_ptr = alloca i8*
    %p2_1_ptr = alloca i8*
    %r_2_ptr = alloca i8*
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
    %p1_0_ptr_raw = call i8* @fax_gc_alloc(i64 16, i64* null, i64 0)
    store i8* %p1_0_ptr_raw, i8** %p1_0_ptr
    %slot0 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 0
    store i8* %p1_0_ptr_raw, i8** %slot0
    %p1_0_cast = bitcast i8* %p1_0_ptr_raw to %struct.Point*
    %gep3 = getelementptr %struct.Point, %struct.Point* %p1_0_cast, i32 0, i32 0
    store i64 1, i64* %gep3
    %gep4 = getelementptr %struct.Point, %struct.Point* %p1_0_cast, i32 0, i32 1
    store i64 2, i64* %gep4
    %p2_1_ptr_raw = call i8* @fax_gc_alloc(i64 16, i64* null, i64 0)
    store i8* %p2_1_ptr_raw, i8** %p2_1_ptr
    %slot1 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 1
    store i8* %p2_1_ptr_raw, i8** %slot1
    %p2_1_cast = bitcast i8* %p2_1_ptr_raw to %struct.Point*
    %gep5 = getelementptr %struct.Point, %struct.Point* %p2_1_cast, i32 0, i32 0
    store i64 10, i64* %gep5
    %gep6 = getelementptr %struct.Point, %struct.Point* %p2_1_cast, i32 0, i32 1
    store i64 20, i64* %gep6
    %r_2_ptr_raw = call i8* @fax_gc_alloc(i64 16, i64* null, i64 0)
    store i8* %r_2_ptr_raw, i8** %r_2_ptr
    %slot2 = getelementptr [8 x i8*], [8 x i8*]* %roots_alloca, i32 0, i32 2
    store i8* %r_2_ptr_raw, i8** %slot2
    %r_2_cast = bitcast i8* %r_2_ptr_raw to %struct.Rect*
    %ptr_raw7 = load i8*, i8** %p1_0_ptr
    %cast_i648 = ptrtoint i8* %ptr_raw7 to i64
    %gep9 = getelementptr %struct.Rect, %struct.Rect* %r_2_cast, i32 0, i32 0
    store i64 %cast_i648, i64* %gep9
    %ptr_raw10 = load i8*, i8** %p2_1_ptr
    %cast_i6411 = ptrtoint i8* %ptr_raw10 to i64
    %gep12 = getelementptr %struct.Rect, %struct.Rect* %r_2_cast, i32 0, i32 1
    store i64 %cast_i6411, i64* %gep12
    %ptr_raw13 = load i8*, i8** %r_2_ptr
    %cast14 = bitcast i8* %ptr_raw13 to %struct.Rect*
    %gep14 = getelementptr %struct.Rect, %struct.Rect* %cast14, i32 0, i32 0
    %val_ptr14 = load i64, i64* %gep14
    %val14 = inttoptr i64 %val_ptr14 to i8*
    %cast15 = bitcast i8* %val14 to %struct.Point*
    %gep15 = getelementptr %struct.Point, %struct.Point* %cast15, i32 0, i32 0
    %val15 = load i64, i64* %gep15
    call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %val15)
    %ptr_raw16 = load i8*, i8** %r_2_ptr
    %cast17 = bitcast i8* %ptr_raw16 to %struct.Rect*
    %gep17 = getelementptr %struct.Rect, %struct.Rect* %cast17, i32 0, i32 1
    %val_ptr17 = load i64, i64* %gep17
    %val17 = inttoptr i64 %val_ptr17 to i8*
    %cast18 = bitcast i8* %val17 to %struct.Point*
    %gep18 = getelementptr %struct.Point, %struct.Point* %cast18, i32 0, i32 1
    %val18 = load i64, i64* %gep18
    call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %val18)
    call void @fax_gc_pop_frame()
    ret i64 0
}