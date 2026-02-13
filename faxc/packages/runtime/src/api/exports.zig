const std = @import("std");
const Fgc = @import("gc/fgc.zig").Fgc;

var global_fgc: ?Fgc = null;

pub fn initFgc() void {
    if (global_fgc == null) {
        global_fgc = Fgc.init(std.heap.page_allocator) catch unreachable;
    }
}

pub fn registerRoot(ptr: ?*?*anyopaque, slot: usize) void {
    if (global_fgc == null) initFgc();
    if (slot < 256) global_fgc.?.globals[slot] = ptr;
}

pub fn allocate(size: usize, ptr_map_ptr: ?[*]const usize, ptr_map_len: usize) ?*anyopaque {
    if (global_fgc == null) initFgc();
    const obj = global_fgc.?.allocObject(size, if (ptr_map_ptr) |p| p[0..ptr_map_len] else &.{}, .Struct) catch return null;
    return @ptrCast(obj.data());
}

pub fn allocateString(length: usize) ?*anyopaque {
    if (global_fgc == null) initFgc();
    const obj = global_fgc.?.allocObject(length + 1, &.{}, .String) catch return null;
    obj.data()[length] = 0;
    return @ptrCast(obj.data());
}

pub fn allocateArray(element_size: usize, count: usize, is_pointer: bool) ?*anyopaque {
    if (global_fgc == null) initFgc();
    var obj = global_fgc.?.allocObject(element_size * count, &.{}, .Array) catch return null;
    if (is_pointer) obj.flags |= @as(u8, @bitCast(@as(u1, 1)));
    return @ptrCast(obj.data());
}

pub fn collectGarbage() void {
    if (global_fgc) |*gc| gc.collect() catch {};
}

pub fn pushFrame(ptr: ?*anyopaque) void {
    if (global_fgc == null) initFgc();
    if (ptr) |p| {
        const frame = @as(*Frame, @ptrCast(@alignCast(p)));
        frame.next = global_fgc.?.top_frame;
        global_fgc.?.top_frame = frame;
    }
}

pub fn popFrame() void {
    if (global_fgc) |*gc| if (gc.top_frame) |frame| {
        gc.top_frame = frame.next;
    };
}

pub fn concatStrings(a_ptr: ?*anyopaque, b_ptr: ?*anyopaque) ?*anyopaque {
    if (global_fgc == null) initFgc();
    if (a_ptr == null or b_ptr == null) return null;

    const a = @as([*:0]const u8, @ptrCast(a_ptr));
    const b = @as([*:0]const u8, @ptrCast(b_ptr));

    const len_a = std.mem.len(a);
    const len_b = std.mem.len(b);
    const total = len_a + len_b;

    const obj = global_fgc.?.allocObject(total + 1, &.{}, .String) catch return null;
    const data = obj.data();

    @memcpy(data[0..len_a], a[0..len_a]);
    @memcpy(data[len_a..total], b[0..len_b]);
    data[total] = 0;

    return @ptrCast(data);
}

const Frame = @import("gc/fgc.zig").Frame;
