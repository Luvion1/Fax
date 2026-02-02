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

define i64 @add(i64 %arg_a, i64 %arg_b) {
entry:
call void @fax_gc_init()
    %atmp0 = add i64 %arg_a, 0
    %atmp1 = add i64 %arg_b, 0
    %bin2 = add i64 %atmp0, %atmp1
    ret i64 %bin2
    ret i64 0
}

define i64 @main() {
entry:
    call void @fax_gc_init()
    %res_3_ptr = alloca i64
    store i64 0, i64* %res_3_ptr
    %res4 = call i64 @add(i64 10, i64 20)
    store i64 %res4, i64* %res_3_ptr
    %tmp5 = load i64, i64* %res_3_ptr
    %ign6 = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %tmp5)
    ret i64 0
}