const std = @import("std");

pub const ObjectType = enum {
    Integer,
    Struct,
};

pub const Object = struct {
    obj_type: ObjectType,
    marked: bool = false,
    next: ?*Object = null,
    size: usize,
    ptr_map: []const usize = &.{},

    pub fn data(self: *Object) [*]u8 {
        return @ptrCast(@as([*]u8, @ptrCast(self)) + @sizeOf(Object));
    }
};

pub const GC = struct {
    allocator: std.mem.Allocator,
    first_object: ?*Object = null,
    // Store object pointers directly
    roots: [128]?*anyopaque = [_]?*anyopaque{null} ** 128,
    num_objects: usize = 0,
    max_objects: usize = 4,

    pub fn init(allocator: std.mem.Allocator) GC {
        return GC{ .allocator = allocator };
    }

    pub fn allocate(self: *GC, obj_type: ObjectType, size: usize, ptr_map: []const usize) !*Object {
        if (self.num_objects >= self.max_objects) {
            try self.collect();
        }
        
        const full_size = @sizeOf(Object) + size;
        const memory = try self.allocator.alignedAlloc(u8, std.mem.Alignment.@"8", full_size);
        @memset(memory, 0);
        
        const obj = @as(*Object, @ptrCast(@alignCast(memory.ptr)));
        
        var owned_ptr_map: []const usize = &.{};
        if (ptr_map.len > 0) {
            const new_map = try self.allocator.alloc(usize, ptr_map.len);
            @memcpy(new_map, ptr_map);
            owned_ptr_map = new_map;
        }

        obj.* = .{ 
            .obj_type = obj_type, 
            .size = size, 
            .next = self.first_object,
            .ptr_map = owned_ptr_map 
        };
        
        self.first_object = obj;
        self.num_objects += 1;
        
        return obj;
    }

    pub fn mark(self: *GC) void {
        for (self.roots) |maybe_ptr| {
            if (maybe_ptr) |real_obj_ptr| {
                self.markObject(real_obj_ptr);
            }
        }
    }

    fn markObject(self: *GC, data_ptr: ?*anyopaque) void {
        if (data_ptr == null) return;
        const dp = data_ptr.?;
        if (@intFromPtr(dp) < 0x1000) return;

        // Verify object existence via linear scan
        var curr = self.first_object;
        while (curr) |obj| {
            if (@intFromPtr(obj.data()) == @intFromPtr(dp)) {
                if (obj.marked) return;
                obj.marked = true;
                
                if (obj.ptr_map.len > 0) {
                    const data_bytes = @as([*]u8, @ptrCast(dp));
                    for (obj.ptr_map) |offset| {
                        if (offset + 8 <= obj.size) {
                            const field_ptr_ptr = @as(*?*anyopaque, @ptrCast(@alignCast(data_bytes + offset)));
                            self.markObject(field_ptr_ptr.*);
                        }
                    }
                }
                break;
            }
            curr = obj.next;
        }
    }

    pub fn sweep(self: *GC) void {
        var prev: ?*Object = null;
        var current = self.first_object;

        while (current) |obj| {
            if (!obj.marked) {
                const next = obj.next;
                if (prev) |p| p.next = next else self.first_object = next;
                
                if (obj.ptr_map.len > 0) {
                    self.allocator.free(obj.ptr_map);
                }

                const full_size = @sizeOf(Object) + obj.size;
                self.allocator.free(@as([*]u8, @ptrCast(obj))[0..full_size]);
                
                self.num_objects -= 1;
                current = next;
            } else {
                obj.marked = false; // Reset for next cycle
                prev = current;
                current = obj.next;
            }
        }
    }

    pub fn collect(self: *GC) !void {
        self.mark();
        self.sweep();
        self.max_objects = self.num_objects + 4;
    }
};

var global_gc: ?GC = null;

export fn fax_gc_init() void {
    global_gc = GC.init(std.heap.page_allocator);
}

export fn fax_gc_register_root(obj_ptr: ?*anyopaque, slot: usize) void {
    if (global_gc) |*gc| {
        if (slot < 128) gc.roots[slot] = obj_ptr;
    }
}

export fn fax_gc_alloc(size: usize, ptr_map_ptr: ?[*]const usize, ptr_map_len: usize) ?*anyopaque {
    if (global_gc == null) fax_gc_init();
    const ptr_map = if (ptr_map_ptr) |ptr| ptr[0..ptr_map_len] else &[_]usize{};
    const obj = global_gc.?.allocate(.Struct, size, ptr_map) catch return null;
    return @as(*anyopaque, @ptrCast(obj.data()));
}

export fn fax_gc_collect() void {
    if (global_gc) |*gc| gc.collect() catch {};
}

// Stubs for backward compatibility
export fn fax_gc_push_frame(_: ?*anyopaque) void {}
export fn fax_gc_pop_frame() void {}