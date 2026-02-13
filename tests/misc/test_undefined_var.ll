; Fax Codegen

declare void @fax_fgc_init()
declare i8* @fax_fgc_alloc(i64, i64*, i64)
declare void @fax_fgc_collect()
declare void @fax_fgc_register_root(i8**, i64)
declare i64 @printf(i8*, ...)

define void @Std_io_collect_fgc() {
entry:
    call void @fax_fgc_collect()
    ret void
}

@fmt_int = private unnamed_addr constant [5 x i8] c"%ld\0A\00"
@fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"

define i64 @main() {
entry:
    call void @fax_fgc_init()
    %x_0_ptr = alloca i64
    store i64 0, i64* %x_0_ptr
    store i64 42, i64* %x_0_ptr
    %tmp1 = load i64, i64* %y_ptr
    %ign2 = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 %tmp1)
    ret i64 0
}
