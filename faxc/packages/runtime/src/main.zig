const std = @import("std");
const fgc_mod = @import("gc/fgc.zig");
const object = @import("gc/object.zig");

const Fgc = fgc_mod.Fgc;
const Obj = object.Obj;
const Frame = fgc_mod.Frame;

export var runtime_gc: ?*Fgc = null;

extern fn write(fd: i32, buf: [*]const u8, len: usize) isize;

export fn fax_fgc_init() void {
    if (runtime_gc == null) {
        const g = std.heap.c_allocator.create(Fgc) catch unreachable;
        g.* = Fgc.init(std.heap.c_allocator) catch unreachable;
        runtime_gc = g;
    }
}

export fn fax_fgc_register_root(ptr: ?*?*anyopaque, slot: usize) void {
    if (runtime_gc == null) fax_fgc_init();
    if (slot < 256) runtime_gc.?.globals[slot] = ptr;
}

export fn fax_fgc_alloc(sz: usize, pmap_ptr: ?[*]const usize, pmap_len: usize) ?*anyopaque {
    if (runtime_gc == null) fax_fgc_init();
    const o = runtime_gc.?.allocObj(sz, if (pmap_ptr) |p| p[0..pmap_len] else &.{}, .Struct) catch return null;
    return @ptrCast(o.data());
}

export fn fax_fgc_alloc_string(len: usize) ?*anyopaque {
    if (runtime_gc == null) fax_fgc_init();
    const o = runtime_gc.?.allocObj(len + 1, &.{}, .String) catch return null;
    o.data()[len] = 0;
    return @ptrCast(o.data());
}

export fn fax_fgc_alloc_array(esz: usize, count: usize, is_ptr: bool) ?*anyopaque {
    if (runtime_gc == null) fax_fgc_init();
    var o = runtime_gc.?.allocObj(esz * count, &.{}, .Array) catch return null;
    if (is_ptr) o.flags |= Obj.FLAG_PTR;
    return @ptrCast(o.data());
}

export fn fax_fgc_collect() void {
    if (runtime_gc) |g| {
        _ = write(1, "GC: collect\n", 12);
        g.collect() catch {};
    } else {
        _ = write(1, "GC: null\n", 9);
    }
}

export fn fax_fgc_push_frame(ptr: ?*anyopaque) void {
    if (runtime_gc == null) fax_fgc_init();
    if (ptr) |p| {
        const f = @as(*Frame, @ptrCast(@alignCast(p)));
        f.next = runtime_gc.?.top;
        runtime_gc.?.top = f;
    }
}

export fn fax_fgc_pop_frame() void {
    if (runtime_gc) |g| if (g.top) |t| {
        g.top = t.next;
    };
}

export fn fax_str_concat(a_ptr: ?*anyopaque, b_ptr: ?*anyopaque) ?*anyopaque {
    if (runtime_gc == null) fax_fgc_init();
    if (a_ptr == null or b_ptr == null) return null;
    const a = @as([*:0]const u8, @ptrCast(a_ptr));
    const b = @as([*:0]const u8, @ptrCast(b_ptr));
    const la = std.mem.len(a);
    const lb = std.mem.len(b);
    const tot = la + lb;
    const o = runtime_gc.?.allocObj(tot + 1, &.{}, .String) catch return null;
    const d = o.data();
    @memcpy(d[0..la], a[0..la]);
    @memcpy(d[la..tot], b[0..lb]);
    d[tot] = 0;
    return @ptrCast(d);
}
