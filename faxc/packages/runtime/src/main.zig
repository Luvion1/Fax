const std = @import("std");
const fgc = @import("gc/fgc.zig");
const object = @import("gc/object.zig");

const Fgc = fgc.Fgc;
const Obj = object.Obj;
const Frame = fgc.Frame;
const GcError = fgc.GcError;

// Thread-safe runtime state
var runtime_gc: ?*Fgc = null;
var runtime_mutex: std.Thread.Mutex = .{};

// Error message buffer for debugging
var error_buffer: [1024]u8 = undefined;
var error_len: usize = 0;

/// Safe initialization with error handling
fn ensureInitialized() bool {
    if (runtime_gc != null) return true;

    runtime_mutex.lock();
    defer runtime_mutex.unlock();

    if (runtime_gc != null) return true;

    const g = std.heap.c_allocator.create(Fgc) catch {
        setError("Failed to allocate GC");
        return false;
    };

    g.* = fgc.Fgc.init(std.heap.c_allocator) catch |err| {
        std.heap.c_allocator.destroy(g);
        setError(switch (err) {
            GcError.OutOfMemory => "Out of memory during GC initialization",
            else => "Unknown error during GC initialization",
        });
        return false;
    };

    runtime_gc = g;
    return true;
}

/// Set error message
fn setError(msg: []const u8) void {
    error_len = @min(msg.len, error_buffer.len);
    @memcpy(error_buffer[0..error_len], msg[0..error_len]);
}

/// Get last error message (for debugging)
export fn fax_fgc_get_error() ?[*:0]const u8 {
    if (error_len == 0) return null;
    error_buffer[error_len] = 0;
    return @ptrCast(&error_buffer);
}

/// Safe initialization
export fn fax_fgc_init() void {
    _ = ensureInitialized();
}

/// Safe allocation with validation
export fn fax_fgc_alloc(sz: usize, pmap_ptr: ?[*]const usize, pmap_len: usize) ?*anyopaque {
    if (!ensureInitialized()) return null;

    if (sz == 0 or sz > 1024 * 1024 * 1024) {
        setError("Invalid allocation size");
        return null;
    }

    const o = runtime_gc.?.allocObj(sz, if (pmap_ptr) |p| p[0..pmap_len] else &.{}, .Struct) catch |err| {
        setError(switch (err) {
            GcError.OutOfMemory => "Out of memory",
            GcError.AllocationTooLarge => "Allocation too large",
            else => "Allocation failed",
        });
        return null;
    };

    return @ptrCast(o.data());
}

/// Safe string allocation
export fn fax_fgc_alloc_string(len: usize) ?*anyopaque {
    if (!ensureInitialized()) return null;

    if (len > 1024 * 1024 * 1024) {
        setError("String too long");
        return null;
    }

    const o = runtime_gc.?.allocObj(len + 1, &.{}, .String) catch |err| {
        setError(switch (err) {
            GcError.OutOfMemory => "Out of memory",
            else => "String allocation failed",
        });
        return null;
    };

    o.count = len;
    o.data()[len] = 0;
    return @ptrCast(o.data());
}

/// Safe array allocation
export fn fax_fgc_alloc_array(esz: usize, count: usize, is_ptr: bool) ?*anyopaque {
    if (!ensureInitialized()) return null;

    if (count == 0 or esz == 0) {
        setError("Invalid array parameters");
        return null;
    }

    // Check for overflow
    const total_size = esz * count;
    if (total_size / count != esz or total_size > 1024 * 1024 * 1024) {
        setError("Array size overflow");
        return null;
    }

    var o = runtime_gc.?.allocObj(total_size, &.{}, .Array) catch |err| {
        setError(switch (err) {
            GcError.OutOfMemory => "Out of memory",
            else => "Array allocation failed",
        });
        return null;
    };

    o.count = count;
    if (is_ptr) o.flags |= Obj.FLAG_PTR;
    return @ptrCast(o.data());
}

/// Get array/string length safely
export fn fax_fgc_len(ptr: ?*anyopaque) usize {
    if (ptr == null) return 0;
    if (runtime_gc == null) return 0;

    const o = runtime_gc.?.getObj(ptr) orelse return 0;
    return o.count;
}

/// Get string length (C-style null-terminated)
export fn fax_str_len(ptr: ?*anyopaque) usize {
    if (ptr == null) return 0;

    const s = @as([*:0]const u8, @ptrCast(ptr));
    return std.mem.len(s);
}

/// Trigger garbage collection
export fn fax_fgc_collect() void {
    if (runtime_gc) |g| {
        _ = write(1, "GC: collect\n", 12);
        g.collect() catch |err| {
            setError(switch (err) {
                GcError.OutOfMemory => "Out of memory during collection",
                else => "Collection failed",
            });
        };
    }
}

/// Trigger minor collection (nursery only)
export fn fax_fgc_minor_collect() void {
    if (runtime_gc) |g| {
        g.minorCollect() catch |err| {
            setError(switch (err) {
                GcError.OutOfMemory => "Out of memory during minor collection",
                else => "Minor collection failed",
            });
        };
    }
}

/// Write barrier for generational GC
export fn fax_fgc_write_barrier(obj: ?*anyopaque, value: ?*anyopaque) void {
    if (runtime_gc) |g| {
        g.writeBarrier(obj, value);
    }
}

/// Read barrier - follow forwarding pointers
export fn fax_fgc_barrier(ptr: ?*anyopaque) ?*anyopaque {
    if (ptr == null) return null;
    if (runtime_gc == null) return ptr;

    const o = runtime_gc.?.getObj(ptr) orelse return ptr;
    return o.barrier(runtime_gc.?.color).data();
}

/// Read barrier with healing - updates source pointer
export fn fax_fgc_barrier_heal(addr: ?*?*anyopaque) ?*anyopaque {
    if (addr == null or addr.?.* == null) {
        return if (addr) |a| a.* else null;
    }
    if (runtime_gc == null) return addr.?.*;

    const ptr = addr.?.*;
    const o = runtime_gc.?.getObj(ptr) orelse return ptr;
    const new_obj = o.barrier(runtime_gc.?.color);
    const new_ptr: ?*anyopaque = @ptrCast(new_obj.data());

    if (new_ptr != ptr) {
        addr.?.* = new_ptr;
    }

    return new_ptr;
}

/// Bounds check with safe error handling
export fn fax_fgc_bounds_check(ptr: ?*anyopaque, index: usize) bool {
    if (ptr == null) {
        setError("Null pointer in bounds check");
        return false;
    }
    if (runtime_gc == null) {
        setError("GC not initialized");
        return false;
    }

    const o = runtime_gc.?.getObj(ptr) orelse {
        setError("Invalid object in bounds check");
        return false;
    };

    if (index >= o.count) {
        setError("Array index out of bounds");
        return false;
    }

    return true;
}

/// Push frame for root scanning
export fn fax_fgc_push_frame(ptr: ?*anyopaque) void {
    if (!ensureInitialized()) return;

    if (ptr) |p| {
        const f = @as(*Frame, @ptrCast(@alignCast(p)));
        f.next = runtime_gc.?.top;
        runtime_gc.?.top = f;
    }
}

/// Pop frame
export fn fax_fgc_pop_frame() void {
    if (runtime_gc) |g| {
        if (g.top) |t| {
            g.top = t.next;
        }
    }
}

/// String concatenation with safety checks
export fn fax_str_concat(a_ptr: ?*anyopaque, b_ptr: ?*anyopaque) ?*anyopaque {
    if (!ensureInitialized()) return null;
    if (a_ptr == null or b_ptr == null) return null;

    const a = @as([*:0]const u8, @ptrCast(a_ptr));
    const b = @as([*:0]const u8, @ptrCast(b_ptr));

    const la = std.mem.len(a);
    const lb = std.mem.len(b);

    // Check for overflow
    if (la > 1024 * 1024 * 1024 or lb > 1024 * 1024 * 1024) {
        setError("String too long for concatenation");
        return null;
    }

    const total = la + lb;
    if (total < la or total > 1024 * 1024 * 1024) {
        setError("String concatenation overflow");
        return null;
    }

    const o = runtime_gc.?.allocObj(total + 1, &.{}, .String) catch |err| {
        setError(switch (err) {
            GcError.OutOfMemory => "Out of memory during string concat",
            else => "String concatenation failed",
        });
        return null;
    };

    const d = o.data();
    @memcpy(d[0..la], a[0..la]);
    @memcpy(d[la..total], b[0..lb]);
    d[total] = 0;
    o.count = total;

    return @ptrCast(d);
}

/// Register global root
export fn fax_fgc_register_root(ptr: ?*?*anyopaque, slot: usize) void {
    if (!ensureInitialized()) return;
    if (slot >= 256) {
        setError("Root slot out of range");
        return;
    }
    runtime_gc.?.globals[slot] = ptr;
}

/// Get GC statistics
export fn fax_fgc_get_stats(stats: ?*GcStats) void {
    if (stats == null or runtime_gc == null) return;

    const s = runtime_gc.?.getStats();
    stats.?.* = .{
        .total_allocations = s.total_allocations,
        .total_collections = s.total_collections,
        .small_pages = s.small_pages,
        .medium_pages = s.medium_pages,
        .large_pages = s.large_pages,
    };
}

/// Statistics structure for FFI
pub const GcStats = extern struct {
    total_allocations: usize,
    total_collections: usize,
    small_pages: usize,
    medium_pages: usize,
    large_pages: usize,
};

/// Clean shutdown
export fn fax_fgc_shutdown() void {
    runtime_mutex.lock();
    defer runtime_mutex.unlock();

    if (runtime_gc) |g| {
        g.deinit();
        std.heap.c_allocator.destroy(g);
        runtime_gc = null;
    }
}

extern fn write(fd: i32, buf: [*]const u8, len: usize) isize;
