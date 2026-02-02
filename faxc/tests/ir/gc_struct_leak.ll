; Fax Codegen
target triple = "x86_64-pc-linux-gnu"

declare void @fax_gc_init()
declare i8* @fax_gc_alloc(i64, i64*, i64)
declare void @fax_gc_collect()
declare void @fax_gc_register_root(i8*, i64)
declare i64 @printf(i8*, ...)

define void @Std_io_collect_gc() {
entry:
    call void @fax_gc_collect()
    ret void
}

@fmt_int = private unnamed_addr constant [5 x i8] c"%ld\0A\00"
@fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"
%struct.Inner = type { i64 }
%struct.Outer = type { i64 }

define i64 @create_outer() {
entry:
call void @fax_gc_init()
    %i_0_ptr = alloca i8*
    store i8* null, i8** %i_0_ptr
    %o_1_ptr = alloca i8*
    store i8* null, i8** %o_1_ptr
    %i_0_ptr_raw = call i8* @fax_gc_alloc(i64 8, i64* null, i64 0)
    store i8* %i_0_ptr_raw, i8** %i_0_ptr
    call void @fax_gc_register_root(i8* %i_0_ptr_raw, i64 0)
    %i_0_cast = bitcast i8* %i_0_ptr_raw to %struct.Inner*
    %gep3 = getelementptr %struct.Inner, %struct.Inner* %i_0_cast, i32 0, i32 0
    store i64 42, i64* %gep3
    %map_ptr4 = getelementptr [1 x i64], [1 x i64]* @pmap_4, i32 0, i32 0
    %o_1_ptr_raw = call i8* @fax_gc_alloc(i64 8, i64* %map_ptr4, i64 1)
    store i8* %o_1_ptr_raw, i8** %o_1_ptr
    call void @fax_gc_register_root(i8* %o_1_ptr_raw, i64 1)
    %o_1_cast = bitcast i8* %o_1_ptr_raw to %struct.Outer*
    %ptr_raw5 = load i8*, i8** %i_0_ptr
    %field_cast6 = ptrtoint i8* %ptr_raw5 to i64
    %gep7 = getelementptr %struct.Outer, %struct.Outer* %o_1_cast, i32 0, i32 0
    store i64 %field_cast6, i64* %gep7
    %ptr_raw8 = load i8*, i8** %o_1_ptr
    %cast_i649 = ptrtoint i8* %ptr_raw8 to i64
    ret i64 %cast_i649
    ret i64 0
}

define i64 @main() {
entry:
    call void @fax_gc_init()
    %o_10_ptr = alloca i8*
    store i8* null, i8** %o_10_ptr
    %res11 = call i64 @create_outer()
    %call_ptr12 = inttoptr i64 %res11 to i8*
    %var_cast13 = ptrtoint i8* %call_ptr12 to i64
    %ptr_val14 = inttoptr i64 %var_cast13 to i8*
    store i8* %ptr_val14, i8** %o_10_ptr
    call void @fax_gc_register_root(i8* %ptr_val14, i64 2)
    call void @Std_io_collect_gc()
    %ptr_raw16 = load i8*, i8** %o_10_ptr
    %cast17 = bitcast i8* %ptr_raw16 to %struct.Outer*
    %gep17 = getelementptr %struct.Outer, %struct.Outer* %cast17, i32 0, i32 0
    %val_ptr17 = load i64, i64* %gep17
    %val17 = inttoptr i64 %val_ptr17 to i8*
    %cast18 = bitcast i8* %val17 to %struct.Inner*
    %gep18 = getelementptr %struct.Inner, %struct.Inner* %cast18, i32 0, i32 0
    %val18 = load i64, i64* %gep18
    %ign19 = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %val18)
    ret i64 0
}
@pmap_4 = private unnamed_addr constant [1 x i64] [i64 0]