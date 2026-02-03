const std = @import("std");

pub const ObjectType = enum(u8) {
    Integer,
    Struct,
    Array,
    String,
};

// ZGC Coloring (Bit Flags)
const ZGC_COLOR_M0: u8 = 0x1;
const ZGC_COLOR_M1: u8 = 0x2;
const ZGC_COLOR_REMAPPED: u8 = 0x4;

const MAGIC_NUMBER: u64 = 0xDEADC0DECAFEBABE;
const FORWARDED_MAGIC: u64 = 0xF084A4D146000000;

pub const Object = struct {
    magic: u64 = MAGIC_NUMBER,
    forwarding_ptr: ?*Object = null,
    size: usize,
    ptr_map: []const usize = &.{},
    obj_type: ObjectType,
    color: u8,
    flags: u8 = 0,

    pub const FLAG_PTR_ARRAY: u8 = 0x1;

    pub fn data(self: *Object) [*]u8 {
        const header_size = std.mem.alignForward(usize, @sizeOf(Object), 16);
        return @ptrCast(@as([*]u8, @ptrCast(self)) + header_size);
    }

    pub fn fromData(d: ?*anyopaque) ?*Object {
        if (d == null) return null;
        const header_size = std.mem.alignForward(usize, @sizeOf(Object), 16);
        const obj_ptr = @as([*]u8, @ptrCast(d.?)) - header_size;
        const obj = @as(*Object, @ptrCast(@alignCast(obj_ptr)));
        if (obj.magic != MAGIC_NUMBER and obj.magic != FORWARDED_MAGIC) return null;
        return obj;
    }

    /// Load Barrier (Jantung ZGC)
    pub fn loadBarrier(self: *Object, current_fgc_color: u8) *Object {
        if (self.color != current_fgc_color) {
            return self.slowPath(current_fgc_color);
        }
        return self;
    }

    fn slowPath(self: *Object, current_fgc_color: u8) *Object {
        var current = self;
        if (current.magic == FORWARDED_MAGIC) {
            current = current.forwarding_ptr.?;
        }
        if (current_fgc_color != ZGC_COLOR_REMAPPED) {
            current.color = current_fgc_color;
        }
        return current;
    }

    pub fn fullSize(self: *Object) usize {
        const header_size = std.mem.alignForward(usize, @sizeOf(Object), 16);
        return std.mem.alignForward(usize, header_size + self.size, 16);
    }
};

const PageType = enum { Small, Medium, Large };

const ZPage = struct {
    start: [*]u8,
    top: [*]u8,
    end: [*]u8,
    p_type: PageType,
    allocated_bytes: usize = 0,

    pub fn init(allocator: std.mem.Allocator, p_type: PageType, custom_size: usize) !*ZPage {
        const size: usize = switch (p_type) {
            .Small => 2 * 1024 * 1024,   // 2MB
            .Medium => 32 * 1024 * 1024, // 32MB
            .Large => custom_size,
        };
        
        const page_struct = try allocator.create(ZPage);
        const mem = try allocator.alloc(u8, size);
        @memset(mem, 0);
        
        page_struct.* = .{
            .start = mem.ptr,
            .top = mem.ptr,
            .end = mem.ptr + size,
            .p_type = p_type,
            .allocated_bytes = 0,
        };
        return page_struct;
    }

    pub fn remains(self: *ZPage) usize {
        return @intFromPtr(self.end) - @intFromPtr(self.top);
    }

    pub fn deinit(self: *ZPage, allocator: std.mem.Allocator) void {
        const size = @intFromPtr(self.end) - @intFromPtr(self.start);
        allocator.free(self.start[0..size]);
        allocator.destroy(self);
    }
};

pub const StackFrame = struct {
    next: ?*StackFrame,
    roots: [*]*anyopaque,
    root_count: usize,
};

pub const Fgc = struct {
    allocator: std.mem.Allocator,
    
    // Page Management
    small_pages: std.ArrayListUnmanaged(*ZPage),
    medium_pages: std.ArrayListUnmanaged(*ZPage),
    large_pages: std.ArrayListUnmanaged(*ZPage),

    // State
    current_color: u8 = ZGC_COLOR_M0,
    fgc_phase: Phase = .Idle,
    mark_stack: std.ArrayListUnmanaged(*Object),
    
    // Roots
    stack_top: ?*StackFrame = null,
    global_roots: [256]?*?*anyopaque = [_]?*?*anyopaque{null} ** 256,

    pub const Phase = enum { Idle, Marking, Relocating };

    pub fn init(allocator: std.mem.Allocator) !Fgc {
        var self = Fgc{
            .allocator = allocator,
            .small_pages = .{},
            .medium_pages = .{},
            .large_pages = .{},
            .mark_stack = .{},
        };
        try self.small_pages.append(allocator, try ZPage.init(allocator, .Small, 0));
        return self;
    }

    pub fn allocate(self: *Fgc, size: usize, ptr_map: []const usize, obj_type: ObjectType) !*Object {
        const header_size = std.mem.alignForward(usize, @sizeOf(Object), 16);
        const full_size = std.mem.alignForward(usize, header_size + size, 16);
        
        const p_type: PageType = if (full_size <= 2048) .Small else if (full_size <= 256 * 1024) .Medium else .Large;
        
        var target_list = switch (p_type) {
            .Small => &self.small_pages,
            .Medium => &self.medium_pages,
            .Large => &self.large_pages,
        };

        var target_page: ?*ZPage = null;
        for (target_list.items) |p| {
            if (p.remains() >= full_size) {
                target_page = p;
                break;
            }
        }

        if (target_page == null) {
            try self.collect();
            for (target_list.items) |p| {
                if (p.remains() >= full_size) {
                    target_page = p;
                    break;
                }
            }
        }

        if (target_page == null) {
            target_page = try ZPage.init(self.allocator, p_type, full_size);
            try target_list.append(self.allocator, target_page.?);
        }

        const memory = target_page.?.top;
        target_page.?.top += full_size;
        target_page.?.allocated_bytes += full_size;
        
        const obj = @as(*Object, @ptrCast(@alignCast(memory)));
        obj.* = .{ 
            .size = size, 
            .ptr_map = ptr_map,
            .obj_type = obj_type,
            .color = self.current_color,
        };
        return obj;
    }

    fn evacuatePages(self: *Fgc, pages: *std.ArrayListUnmanaged(*ZPage), p_type: PageType) !std.ArrayListUnmanaged(*ZPage) {
        if (pages.items.len == 0) return .{};

        var old_pages = std.ArrayListUnmanaged(*ZPage){};
        try old_pages.appendSlice(self.allocator, pages.items);

        var new_page_list = std.ArrayListUnmanaged(*ZPage){};
        const first_dest = try ZPage.init(self.allocator, p_type, 0);
        try new_page_list.append(self.allocator, first_dest);
        var dest_page = first_dest;

        for (old_pages.items) |p| {
            var scan = p.start;
            while (@intFromPtr(scan) < @intFromPtr(p.top)) {
                const obj = @as(*Object, @ptrCast(@alignCast(scan)));
                const fs = obj.fullSize();
                
                if (obj.magic == MAGIC_NUMBER and obj.color == self.current_color) {
                    std.debug.print("[FGC] Relocating obj at {x} to new page\n", .{@intFromPtr(obj)});
                    if (dest_page.remains() < fs) {
                        dest_page = try ZPage.init(self.allocator, p_type, 0);
                        try new_page_list.append(self.allocator, dest_page);
                    }

                    @memcpy(dest_page.top[0..fs], scan[0..fs]);
                    const new_obj = @as(*Object, @ptrCast(@alignCast(dest_page.top)));
                    
                    obj.magic = FORWARDED_MAGIC;
                    obj.forwarding_ptr = new_obj;
                    new_obj.color = ZGC_COLOR_REMAPPED;
                    
                    dest_page.top += fs;
                    dest_page.allocated_bytes += fs;
                }
                scan += fs;
            }
        }
        
        pages.* = new_page_list;
        return old_pages;
    }

    fn remapPointersInPages(pages: []const *ZPage) void {
        for (pages) |p| {
            var scan = p.start;
            while (@intFromPtr(scan) < @intFromPtr(p.top)) {
                const obj = @as(*Object, @ptrCast(@alignCast(scan)));
                const fs = obj.fullSize();
                const d = obj.data();
                
                if (obj.ptr_map.len > 0) {
                    for (obj.ptr_map) |offset| {
                        const field = @as(*?*anyopaque, @ptrCast(@alignCast(d + offset)));
                        if (field.*) |v| {
                            if (Object.fromData(v)) |child| {
                                var actual_child = child;
                                if (actual_child.magic == FORWARDED_MAGIC) {
                                    actual_child = actual_child.forwarding_ptr.?;
                                }
                                field.* = actual_child.data();
                            }
                        }
                    }
                }

                if (obj.obj_type == .Array and (obj.flags & Object.FLAG_PTR_ARRAY) != 0) {
                    const ptrs = @as([*]?*anyopaque, @ptrCast(@alignCast(d)));
                    const count = obj.size / @sizeOf(?*anyopaque);
                    var j: usize = 0;
                    while (j < count) : (j += 1) {
                        if (ptrs[j]) |v| {
                            if (Object.fromData(v)) |child| {
                                var actual_child = child;
                                if (actual_child.magic == FORWARDED_MAGIC) {
                                    actual_child = actual_child.forwarding_ptr.?;
                                }
                                ptrs[j] = actual_child.data();
                            }
                        }
                    }
                }
                scan += fs;
            }
        }
    }

    pub fn collect(self: *Fgc) !void {
        std.debug.print("[FGC] Cycle Start. Phase: Marking\n", .{});
        self.fgc_phase = .Marking;
        self.current_color = if (self.current_color == ZGC_COLOR_M0) ZGC_COLOR_M1 else ZGC_COLOR_M0;

        // 1. Mark Roots
        for (self.global_roots) |maybe_root_ptr| {
            if (maybe_root_ptr) |root_ptr| {
                if (root_ptr.*) |root| {
                    if (Object.fromData(root)) |obj| {
                        if (obj.color != self.current_color) {
                            obj.color = self.current_color;
                            try self.mark_stack.append(self.allocator, obj);
                        }
                    }
                }
            }
        }

        var current_frame = self.stack_top;
        while (current_frame) |frame| {
            var i: usize = 0;
            while (i < frame.root_count) : (i += 1) {
                if (Object.fromData(frame.roots[i])) |obj| {
                    if (obj.color != self.current_color) {
                        obj.color = self.current_color;
                        try self.mark_stack.append(self.allocator, obj);
                    }
                }
            }
            current_frame = frame.next;
        }

        // 2. Transitive Closure
        while (self.mark_stack.items.len > 0) {
            const obj = self.mark_stack.items[self.mark_stack.items.len - 1];
            self.mark_stack.items.len -= 1;
            const d = obj.data();
            
            if (obj.ptr_map.len > 0) {
                for (obj.ptr_map) |offset| {
                    const field = @as(*?*anyopaque, @ptrCast(@alignCast(d + offset)));
                    if (field.*) |v| {
                        if (Object.fromData(v)) |child| {
                            if (child.color != self.current_color) {
                                child.color = self.current_color;
                                try self.mark_stack.append(self.allocator, child);
                            }
                        }
                    }
                }
            }

            if (obj.obj_type == .Array and (obj.flags & Object.FLAG_PTR_ARRAY) != 0) {
                const ptrs = @as([*]?*anyopaque, @ptrCast(@alignCast(d)));
                const count = obj.size / @sizeOf(?*anyopaque);
                var j: usize = 0;
                while (j < count) : (j += 1) {
                    if (ptrs[j]) |v| {
                        if (Object.fromData(v)) |child| {
                            if (child.color != self.current_color) {
                                child.color = self.current_color;
                                try self.mark_stack.append(self.allocator, child);
                            }
                        }
                    }
                }
            }
        }

        // 3. Relocation
        std.debug.print("[FGC] Phase: Relocating\n", .{});
        self.fgc_phase = .Relocating;
        
        var old_small = try self.evacuatePages(&self.small_pages, .Small);
        defer {
            for (old_small.items) |p| p.deinit(self.allocator);
            old_small.deinit(self.allocator);
        }
        
        var old_medium = try self.evacuatePages(&self.medium_pages, .Medium);
        defer {
            for (old_medium.items) |p| p.deinit(self.allocator);
            old_medium.deinit(self.allocator);
        }

        // 4. Remapping
        std.debug.print("[FGC] Phase: Remapping\n", .{});
        for (self.global_roots) |maybe_root_ptr| {
            if (maybe_root_ptr) |root_ptr| {
                if (root_ptr.*) |root| {
                    if (Object.fromData(root)) |obj| {
                        var actual = obj;
                        if (actual.magic == FORWARDED_MAGIC) actual = actual.forwarding_ptr.?;
                        root_ptr.* = actual.data();
                    }
                }
            }
        }

        var remap_frame = self.stack_top;
        while (remap_frame) |frame| {
            var j: usize = 0;
            while (j < frame.root_count) : (j += 1) {
                if (Object.fromData(frame.roots[j])) |obj| {
                    var actual = obj;
                    if (actual.magic == FORWARDED_MAGIC) actual = actual.forwarding_ptr.?;
                    const old_ptr = frame.roots[j];
                    frame.roots[j] = actual.data();
                    if (old_ptr != frame.roots[j]) {
                        std.debug.print("[FGC] Remapped root {d} from {x} to {x}\n", .{j, @intFromPtr(old_ptr), @intFromPtr(frame.roots[j])});
                    }
                }
            }
            remap_frame = frame.next;
        }

        remapPointersInPages(self.small_pages.items);
        remapPointersInPages(self.medium_pages.items);
        remapPointersInPages(self.large_pages.items);

        // 5. Cleanup
        var i: usize = 0;
        while (i < self.large_pages.items.len) {
            const p = self.large_pages.items[i];
            const obj = @as(*Object, @ptrCast(@alignCast(p.start)));
            if (obj.color != self.current_color) {
                _ = self.large_pages.swapRemove(i);
                p.deinit(self.allocator);
            } else {
                i += 1;
            }
        }

        self.fgc_phase = .Idle;
        std.debug.print("[FGC] Cycle Finished.\n", .{});
    }
};

var global_fgc: ?Fgc = null;

export fn fax_fgc_init() void {
    if (global_fgc != null) return;
    global_fgc = Fgc.init(std.heap.page_allocator) catch unreachable;
}

export fn fax_fgc_register_root(root_ptr: ?*?*anyopaque, slot: usize) void {
    if (global_fgc == null) fax_fgc_init();
    if (slot < 256) {
        global_fgc.?.global_roots[slot] = root_ptr;
    }
}

export fn fax_fgc_alloc(size: usize, ptr_map_ptr: ?[*]const usize, ptr_map_len: usize) ?*anyopaque {
    if (global_fgc == null) fax_fgc_init();
    const ptr_map = if (ptr_map_ptr) |ptr| ptr[0..ptr_map_len] else &[_]usize{};
    const obj = global_fgc.?.allocate(size, ptr_map, .Struct) catch return null;
    return @as(*anyopaque, @ptrCast(obj.data()));
}

export fn fax_fgc_alloc_string(len: usize) ?*anyopaque {
    if (global_fgc == null) fax_fgc_init();
    const obj = global_fgc.?.allocate(len + 1, &.{}, .String) catch return null;
    const d = obj.data();
    d[len] = 0;
    return @as(*anyopaque, @ptrCast(d));
}

export fn fax_fgc_alloc_array(element_size: usize, count: usize, is_ptr_array: bool) ?*anyopaque {
    if (global_fgc == null) fax_fgc_init();
    const total_size = element_size * count;
    var obj = global_fgc.?.allocate(total_size, &.{}, .Array) catch return null;
    if (is_ptr_array) obj.flags |= Object.FLAG_PTR_ARRAY;
    return @as(*anyopaque, @ptrCast(obj.data()));
}

export fn fax_fgc_collect() void {
    if (global_fgc) |*fgc| fgc.collect() catch {};
}

export fn fax_fgc_push_frame(frame_ptr: ?*anyopaque) void {
    if (global_fgc == null) fax_fgc_init();
    if (frame_ptr) |fp| {
        const frame = @as(*StackFrame, @ptrCast(@alignCast(fp)));
        frame.next = global_fgc.?.stack_top;
        global_fgc.?.stack_top = frame;
    }
}

export fn fax_fgc_pop_frame() void {
    if (global_fgc) |*fgc| {
        if (fgc.stack_top) |top| fgc.stack_top = top.next;
    }
}

test "StackFrame relocation bug fix" {
    const allocator = std.heap.page_allocator;
    var fgc = try Fgc.init(allocator);

    const obj = try fgc.allocate(16, &.{}, .Struct);
    const original_ptr = obj.data();

    var roots = [_]*anyopaque{@ptrCast(original_ptr)};
    var frame = StackFrame{
        .next = null,
        .roots = @ptrCast(&roots),
        .root_count = 1,
    };
    fgc.stack_top = &frame;

    try fgc.collect();

    const new_ptr = roots[0];
    const original_any = @as(*anyopaque, @ptrCast(original_ptr));
    
    // Verifikasi relokasi benar-benar terjadi dan pointer diupdate
    try std.testing.expect(new_ptr != original_any);
    
    // Verifikasi data tetap sama
    const new_obj = Object.fromData(new_ptr).?;
    try std.testing.expect(new_obj.color == fgc.current_color or new_obj.color == ZGC_COLOR_REMAPPED);
}