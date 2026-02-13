const std = @import("std");
const object = @import("object.zig");
const page = @import("page.zig");

const Obj = object.Obj;
const ObjType = object.ObjType;
const Page = page.Page;
const PageType = page.PageType;

pub const COLOR_M0: u8 = 0x1;
pub const COLOR_M1: u8 = 0x2;

pub const Frame = extern struct { next: ?*Frame, roots: [*]*anyopaque, count: usize };

// Thread-safe mutex for GC operations
var gc_mutex: std.Thread.Mutex = .{};

// Safety configuration
const SAFETY_CHECKS = true;
const MAX_ALLOCATION_SIZE = 1024 * 1024 * 1024; // 1GB max single allocation
const MAX_PAGES = 1024; // Maximum number of pages to prevent unbounded growth

// Error types for better error handling
pub const GcError = error{
    OutOfMemory,
    InvalidPointer,
    AllocationTooLarge,
    TooManyPages,
    NullPointer,
    InvalidObject,
    ConcurrentModification,
};

pub const Fgc = struct {
    alloc: std.mem.Allocator,
    small: std.ArrayListUnmanaged(*Page) = .{},
    medium: std.ArrayListUnmanaged(*Page) = .{},
    large: std.ArrayListUnmanaged(*Page) = .{},
    tenured: std.ArrayListUnmanaged(*Page) = .{},
    remembered_set: std.ArrayListUnmanaged(*Obj) = .{},
    old_pages: std.ArrayListUnmanaged(*Page) = .{},
    nursery: std.ArrayListUnmanaged(*Page) = .{}, // Add nursery for generational GC
    color: u8 = COLOR_M0,
    phase: enum { Idle, Marking, Relocating } = .Idle,

    // Memory Boundaries for O(1) isManaged check
    min_addr: usize = std.math.maxInt(usize),
    max_addr: usize = 0,

    // Marking state
    stack: std.ArrayListUnmanaged(*Obj) = .{},
    stack_mutex: std.Thread.Mutex = .{},
    pending_tasks: std.atomic.Value(usize) = std.atomic.Value(usize).init(0),
    pool: ?*std.Thread.Pool = null,

    top: ?*Frame = null,
    globals: [256]?*?*anyopaque = [_]?*?*anyopaque{null} ** 256,

    // Statistics for debugging
    total_allocations: usize = 0,
    total_collections: usize = 0,

    // Constants
    const AGE_THRESHOLD: u8 = 3; // Lower threshold for faster promotion
    const NURSERY_SIZE: usize = 4 * 1024 * 1024; // 4MB nursery

    pub fn init(alloc: std.mem.Allocator) GcError!Fgc {
        var self = Fgc{ .alloc = alloc };

        // Initialize nursery with a dedicated page
        const nursery_page = Page.init(alloc, .Small, 0) catch return GcError.OutOfMemory;
        try self.addPage(&self.nursery, nursery_page);

        // Initialize thread pool for parallel marking
        const cpu_count = std.Thread.getCpuCount() catch 4;
        const pool = alloc.create(std.Thread.Pool) catch null;
        if (pool) |pl| {
            pl.init(.{ .allocator = alloc, .n_jobs = @intCast(cpu_count) }) catch {
                alloc.destroy(pl);
                self.pool = null;
            };
            self.pool = pl;
        }

        return self;
    }

    fn addPage(self: *Fgc, list: *std.ArrayListUnmanaged(*Page), p: *Page) GcError!void {
        if (list.items.len >= MAX_PAGES) {
            return GcError.TooManyPages;
        }

        list.append(self.alloc, p) catch return GcError.OutOfMemory;
        const start = @intFromPtr(p.start);
        const end = @intFromPtr(p.end);

        // Update memory boundaries atomically
        if (start < self.min_addr) {
            @atomicStore(usize, &self.min_addr, start, .seq_cst);
        }
        if (end > self.max_addr) {
            @atomicStore(usize, &self.max_addr, end, .seq_cst);
        }
    }

    /// Safely allocate an object with comprehensive checks
    pub fn allocObj(self: *Fgc, sz: usize, pmap: []const usize, t: ObjType) GcError!*Obj {
        // Safety check: validate size
        if (sz > MAX_ALLOCATION_SIZE) {
            return GcError.AllocationTooLarge;
        }

        if (sz == 0) {
            return GcError.InvalidObject;
        }

        const full = std.mem.alignForward(usize, 64 + sz, 16);

        // Try nursery allocation for small objects (generational GC)
        if (full <= 2048 and t != .String) {
            if (try self.allocInNursery(full, sz, pmap, t)) |obj| {
                return obj;
            }
        }

        // Fall back to regular generational allocation
        return try self.allocInGeneration(full, sz, pmap, t);
    }

    /// Allocate in nursery (young generation)
    fn allocInNursery(self: *Fgc, full: usize, sz: usize, pmap: []const usize, t: ObjType) GcError!?*Obj {
        // Find space in existing nursery pages
        for (self.nursery.items) |p| {
            if (p.remains() >= full) {
                return try self.createObject(p, full, sz, pmap, t, true);
            }
        }

        // Nursery full - trigger minor collection
        self.minorCollect() catch {};

        // Try again after collection
        for (self.nursery.items) |p| {
            if (p.remains() >= full) {
                return try self.createObject(p, full, sz, pmap, t, true);
            }
        }

        // If still no space, allocate new nursery page
        const new_page = Page.init(self.alloc, .Small, 0) catch return GcError.OutOfMemory;
        try self.addPage(&self.nursery, new_page);
        return try self.createObject(new_page, full, sz, pmap, t, true);
    }

    /// Allocate in regular generation
    fn allocInGeneration(self: *Fgc, full: usize, sz: usize, pmap: []const usize, t: ObjType) GcError!*Obj {
        const pt: PageType = if (full <= 2048) .Small else if (full <= 256 * 1024) .Medium else .Large;
        const list = switch (pt) {
            .Small => &self.small,
            .Medium => &self.medium,
            .Large => &self.large,
        };

        // Try to find existing page with space
        for (list.items) |p| {
            if (p.remains() >= full) {
                return try self.createObject(p, full, sz, pmap, t, false);
            }
        }

        // No space - trigger collection
        self.collect() catch {};

        // Try again after collection
        for (list.items) |p| {
            if (p.remains() >= full) {
                return try self.createObject(p, full, sz, pmap, t, false);
            }
        }

        // Allocate new page
        const new_page = Page.init(self.alloc, pt, full) catch return GcError.OutOfMemory;
        try self.addPage(list, new_page);
        return try self.createObject(new_page, full, sz, pmap, t, false);
    }

    /// Create object at page location with safety checks
    fn createObject(self: *Fgc, page_ptr: *Page, full: usize, sz: usize, pmap: []const usize, t: ObjType, in_nursery: bool) GcError!*Obj {
        if (SAFETY_CHECKS) {
            if (@intFromPtr(page_ptr.top) + full > @intFromPtr(page_ptr.end)) {
                return GcError.OutOfMemory;
            }
        }

        const mem = page_ptr.top;
        page_ptr.top += full;
        page_ptr.alloc_bytes += full;

        const o = @as(*Obj, @ptrCast(@alignCast(mem)));

        // Initialize with proper values
        o.* = .{
            .magic = Obj.MAGIC,
            .forward = null,
            .size = sz,
            .count = sz / 8,
            .pmap_ptr = if (pmap.len > 0) pmap.ptr else null,
            .pmap_len = pmap.len,
            .type = t,
            .color = @atomicLoad(u8, &self.color, .seq_cst),
            .age = if (in_nursery) 0 else 1, // Objects in nursery start at age 0
            .flags = 0,
            ._pad = undefined,
        };

        self.total_allocations += 1;
        return o;
    }

    /// Write barrier for generational GC
    pub fn writeBarrier(self: *Fgc, object_ptr: ?*anyopaque, value_ptr: ?*anyopaque) void {
        if (object_ptr == null or value_ptr == null) return;

        const o = self.getObj(object_ptr) orelse return;
        if ((@atomicLoad(u8, &o.flags, .seq_cst) & Obj.FLAG_DIRTY) != 0) return;

        const v = self.getObj(value_ptr) orelse return;

        // Only track if old object references young object
        if (o.age >= AGE_THRESHOLD and v.age < AGE_THRESHOLD) {
            const old_flags = @atomicRmw(u8, &o.flags, .Or, Obj.FLAG_DIRTY, .seq_cst);
            if ((old_flags & Obj.FLAG_DIRTY) == 0) {
                self.stack_mutex.lock();
                defer self.stack_mutex.unlock();
                self.remembered_set.append(self.alloc, o) catch {
                    _ = @atomicRmw(u8, &o.flags, .And, ~Obj.FLAG_DIRTY, .seq_cst);
                };
            }
        }
    }

    /// Check if pointer is managed by this GC
    pub fn isManaged(self: *const Fgc, ptr: ?*anyopaque) bool {
        if (ptr == null) return false;

        const addr = @intFromPtr(ptr.?);
        const min = @atomicLoad(usize, &self.min_addr, .seq_cst);
        const max = @atomicLoad(usize, &self.max_addr, .seq_cst);

        if (addr < min or addr >= max) return false;

        // Check all page lists
        inline for (.{ &self.small, &self.medium, &self.large, &self.tenured, &self.nursery }) |list| {
            for (list.items) |p| {
                if (addr >= @intFromPtr(p.start) and addr < @intFromPtr(p.top)) {
                    return true;
                }
            }
        }

        // Check old pages during relocation
        if (@atomicLoad(u8, @as(*const u8, @ptrCast(&self.phase)), .seq_cst) == @intFromEnum(@as(enum { Idle, Marking, Relocating }, .Relocating))) {
            for (self.old_pages.items) |p| {
                if (addr >= @intFromPtr(p.start) and addr < @intFromPtr(p.top)) {
                    return true;
                }
            }
        }

        return false;
    }

    /// Safely get object from pointer with validation
    pub fn getObj(self: *const Fgc, ptr: ?*anyopaque) ?*Obj {
        if (!self.isManaged(ptr)) return null;

        // Calculate object header location (64 bytes before data)
        const header_addr = @intFromPtr(ptr.?) - 64;
        if (header_addr < self.min_addr) return null;

        const o = @as(*Obj, @ptrCast(@alignCast(@as(*anyopaque, @ptrFromInt(header_addr)))));
        const magic = @atomicLoad(u64, &o.magic, .seq_cst);

        if (magic != Obj.MAGIC and magic != Obj.FORWARD_MAGIC) {
            return null;
        }

        return o;
    }

    /// Minor collection - only collects nursery
    pub fn minorCollect(self: *Fgc) GcError!void {
        if (self.nursery.items.len == 0) return;

        gc_mutex.lock();
        defer gc_mutex.unlock();

        const next_color = if (self.color == COLOR_M0) COLOR_M1 else COLOR_M0;

        // Mark from roots
        try self.markRoots(next_color);
        try self.processMarkStack(next_color);

        // Evacuate live objects from nursery
        var old_nursery = std.ArrayListUnmanaged(*Page){};
        try old_nursery.appendSlice(self.alloc, self.nursery.items);
        self.nursery.clearRetainingCapacity();

        // Create new nursery page
        const new_nursery = Page.init(self.alloc, .Small, 0) catch return GcError.OutOfMemory;
        try self.addPage(&self.nursery, new_nursery);

        // Promote survivors
        for (old_nursery.items) |p| {
            var scan = p.start;
            while (@intFromPtr(scan) < @intFromPtr(p.top)) {
                const o = @as(*Obj, @ptrCast(@alignCast(scan)));
                const fs = o.fullSize();

                if (@atomicLoad(u64, &o.magic, .seq_cst) == Obj.MAGIC and
                    @atomicLoad(u8, &o.color, .seq_cst) == next_color)
                {

                    // Promote to old generation
                    var dest_page: ?*Page = null;
                    for (self.small.items) |sp| {
                        if (sp.remains() >= fs) {
                            dest_page = sp;
                            break;
                        }
                    }

                    if (dest_page == null) {
                        const new_page = Page.init(self.alloc, .Small, 0) catch return GcError.OutOfMemory;
                        try self.addPage(&self.small, new_page);
                        dest_page = new_page;
                    }

                    // Copy object
                    @memcpy(dest_page.?.top[0..fs], scan[0..fs]);
                    const new_obj = @as(*Obj, @ptrCast(@alignCast(dest_page.?.top)));
                    if (new_obj.age < 255) new_obj.age += 1;

                    // Set forwarding pointer
                    @atomicStore(u64, &o.magic, Obj.FORWARD_MAGIC, .seq_cst);
                    @atomicStore(?*Obj, &o.forward, new_obj, .seq_cst);

                    dest_page.?.top += fs;
                    dest_page.?.alloc_bytes += fs;
                }

                scan += fs;
            }
        }

        // Update color and fixup
        self.color = next_color;
        self.fixupGlobalAndStack();

        // Cleanup old nursery pages
        for (old_nursery.items) |p| {
            p.deinit(self.alloc);
        }
        old_nursery.deinit(self.alloc);

        self.total_collections += 1;
    }

    /// Full collection with parallel marking
    pub fn collect(self: *Fgc) GcError!void {
        gc_mutex.lock();
        defer gc_mutex.unlock();

        @atomicStore(u8, @as(*u8, @ptrCast(&self.phase)), @intFromEnum(@as(enum { Idle, Marking, Relocating }, .Marking)), .seq_cst);
        const next_color = if (self.color == COLOR_M0) COLOR_M1 else COLOR_M0;

        self.pending_tasks.store(0, .seq_cst);
        self.stack.clearRetainingCapacity();

        // Mark from globals
        for (self.globals) |g| {
            if (g) |ptr| {
                if (ptr.*) |r| {
                    if (self.getObj(r)) |o| {
                        if (@atomicLoad(u8, &o.color, .seq_cst) != next_color) {
                            if (@cmpxchgStrong(u8, &o.color, o.color, next_color, .seq_cst, .seq_cst) == null) {
                                try self.stack.append(self.alloc, o);
                            }
                        }
                    }
                }
            }
        }

        // Mark from stack frames
        var f = self.top;
        while (f) |curr| {
            for (curr.roots[0..curr.count]) |r_ptr| {
                const r = @as(*?*anyopaque, @ptrCast(@alignCast(r_ptr)));
                if (r.*) |data| {
                    if (self.getObj(data)) |o| {
                        if (@atomicLoad(u8, &o.color, .seq_cst) != next_color) {
                            if (@cmpxchgStrong(u8, &o.color, o.color, next_color, .seq_cst, .seq_cst) == null) {
                                try self.stack.append(self.alloc, o);
                            }
                        }
                    }
                }
            }
            f = curr.next;
        }

        // Parallel or sequential marking
        if (self.pool) |p| {
            try self.parallelMark(p, next_color);
        } else {
            try self.sequentialMark(next_color);
        }

        self.color = next_color;

        // Relocation phase
        @atomicStore(u8, @as(*u8, @ptrCast(&self.phase)), @intFromEnum(@as(enum { Idle, Marking, Relocating }, .Relocating)), .seq_cst);

        var old_s = try self.evacuate(&self.small, .Small);
        var old_m = try self.evacuate(&self.medium, .Medium);

        try self.old_pages.appendSlice(self.alloc, old_s.items);
        try self.old_pages.appendSlice(self.alloc, old_m.items);

        self.fixupGlobalAndStack();
        self.remapAll();

        // Cleanup old pages
        for (old_s.items) |p| p.deinit(self.alloc);
        for (old_m.items) |p| p.deinit(self.alloc);
        old_s.deinit(self.alloc);
        old_m.deinit(self.alloc);
        self.old_pages.clearRetainingCapacity();

        // Clean up large objects
        var i: usize = 0;
        while (i < self.large.items.len) {
            const p = self.large.items[i];
            const o = @as(*Obj, @ptrCast(@alignCast(p.start)));
            if (@atomicLoad(u8, &o.color, .seq_cst) != self.color) {
                _ = self.large.swapRemove(i);
                p.deinit(self.alloc);
            } else {
                i += 1;
            }
        }

        @atomicStore(u8, @as(*u8, @ptrCast(&self.phase)), @intFromEnum(@as(enum { Idle, Marking, Relocating }, .Idle)), .seq_cst);
        self.total_collections += 1;
    }

    fn markRoots(self: *Fgc, next_color: u8) GcError!void {
        // Mark from globals
        for (self.globals) |g| {
            if (g) |ptr| {
                if (ptr.*) |r| {
                    if (self.getObj(r)) |o| {
                        if (@atomicLoad(u8, &o.color, .seq_cst) != next_color) {
                            _ = @cmpxchgStrong(u8, &o.color, o.color, next_color, .seq_cst, .seq_cst);
                            try self.stack.append(self.alloc, o);
                        }
                    }
                }
            }
        }

        // Mark from stack
        var f = self.top;
        while (f) |curr| {
            for (curr.roots[0..curr.count]) |r| {
                if (self.getObj(r)) |o| {
                    if (@atomicLoad(u8, &o.color, .seq_cst) != next_color) {
                        _ = @cmpxchgStrong(u8, &o.color, o.color, next_color, .seq_cst, .seq_cst);
                        try self.stack.append(self.alloc, o);
                    }
                }
            }
            f = curr.next;
        }
    }

    fn processMarkStack(self: *Fgc, next_color: u8) GcError!void {
        while (self.stack.items.len > 0) {
            const o = self.stack.items[self.stack.items.len - 1];
            self.stack.items.len -= 1;

            const d = o.data();

            // Process pointer map
            if (o.pmap_ptr) |pmap| {
                for (pmap[0..o.pmap_len]) |off| {
                    if (off + 8 <= o.size) {
                        const field_ptr = @as(*?*anyopaque, @ptrCast(@alignCast(d + off)));
                        if (self.getObj(field_ptr.*)) |c| {
                            if (@atomicLoad(u8, &c.color, .seq_cst) != next_color) {
                                _ = @cmpxchgStrong(u8, &c.color, c.color, next_color, .seq_cst, .seq_cst);
                                try self.stack.append(self.alloc, c);
                            }
                        }
                    }
                }
            }

            // Process array elements if they contain pointers
            if (o.type == .Array and (o.flags & Obj.FLAG_PTR) != 0) {
                const ptrs = @as([*]?*anyopaque, @ptrCast(@alignCast(d)));
                for (ptrs[0..(o.size / 8)]) |v| {
                    if (self.getObj(v)) |c| {
                        if (@atomicLoad(u8, &c.color, .seq_cst) != next_color) {
                            _ = @cmpxchgStrong(u8, &c.color, c.color, next_color, .seq_cst, .seq_cst);
                            try self.stack.append(self.alloc, c);
                        }
                    }
                }
            }
        }
    }

    fn parallelMark(self: *Fgc, p: *std.Thread.Pool, next_color: u8) GcError!void {
        self.stack_mutex.lock();
        for (self.stack.items) |o| {
            _ = self.pending_tasks.fetchAdd(1, .seq_cst);
            p.spawn(markTask, .{ self, o, next_color }) catch {
                _ = self.pending_tasks.fetchSub(1, .seq_cst);
            };
        }
        self.stack.clearRetainingCapacity();

        // Process remembered set
        var rs_idx: usize = 0;
        while (rs_idx < self.remembered_set.items.len) {
            const o = self.remembered_set.items[rs_idx];
            _ = @atomicRmw(u8, &o.flags, .And, ~Obj.FLAG_DIRTY, .seq_cst);
            if (@atomicLoad(u8, &o.color, .seq_cst) == next_color) {
                _ = self.pending_tasks.fetchAdd(1, .seq_cst);
                p.spawn(markTask, .{ self, o, next_color }) catch {
                    _ = self.pending_tasks.fetchSub(1, .seq_cst);
                };
                rs_idx += 1;
            } else {
                _ = self.remembered_set.swapRemove(rs_idx);
            }
        }
        self.stack_mutex.unlock();

        // Wait for all tasks
        while (self.pending_tasks.load(.seq_cst) > 0) {
            std.Thread.yield() catch {};
        }
    }

    fn sequentialMark(self: *Fgc, next_color: u8) GcError!void {
        while (self.stack.items.len > 0) {
            const o = self.stack.items[self.stack.items.len - 1];
            self.stack.items.len -= 1;
            try self.scanObjectSequential(o, next_color);
        }

        // Process remembered set
        var rs_idx: usize = 0;
        while (rs_idx < self.remembered_set.items.len) {
            const o = self.remembered_set.items[rs_idx];
            _ = @atomicRmw(u8, &o.flags, .And, ~Obj.FLAG_DIRTY, .seq_cst);
            if (@atomicLoad(u8, &o.color, .seq_cst) == next_color) {
                try self.scanObjectSequential(o, next_color);
                rs_idx += 1;
            } else {
                _ = self.remembered_set.swapRemove(rs_idx);
            }
        }
    }

    fn markTask(self: *Fgc, o: *Obj, next_color: u8) void {
        self.scanObjectParallel(o, next_color) catch {};
        _ = self.pending_tasks.fetchSub(1, .seq_cst);
    }

    fn scanObjectParallel(self: *Fgc, o: *Obj, next_color: u8) GcError!void {
        const d = o.data();
        if (o.pmap_ptr) |pmap| {
            for (pmap[0..o.pmap_len]) |off| {
                if (off + 8 <= o.size) {
                    const field_ptr = @as(*?*anyopaque, @ptrCast(@alignCast(d + off)));
                    if (self.getObj(field_ptr.*)) |c| {
                        if (@atomicLoad(u8, &c.color, .seq_cst) != next_color) {
                            if (@cmpxchgStrong(u8, &c.color, c.color, next_color, .seq_cst, .seq_cst) == null) {
                                _ = self.pending_tasks.fetchAdd(1, .seq_cst);
                                if (self.pool) |pool| {
                                    pool.spawn(markTask, .{ self, c, next_color }) catch {
                                        _ = self.pending_tasks.fetchSub(1, .seq_cst);
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }
        if (o.type == .Array and (o.flags & Obj.FLAG_PTR) != 0) {
            const ptrs = @as([*]?*anyopaque, @ptrCast(@alignCast(d)));
            for (ptrs[0..(o.size / 8)]) |v| {
                if (self.getObj(v)) |c| {
                    if (@atomicLoad(u8, &c.color, .seq_cst) != next_color) {
                        if (@cmpxchgStrong(u8, &c.color, c.color, next_color, .seq_cst, .seq_cst) == null) {
                            _ = self.pending_tasks.fetchAdd(1, .seq_cst);
                            if (self.pool) |pool| {
                                pool.spawn(markTask, .{ self, c, next_color }) catch {
                                    _ = self.pending_tasks.fetchSub(1, .seq_cst);
                                };
                            }
                        }
                    }
                }
            }
        }
    }

    fn scanObjectSequential(self: *Fgc, o: *Obj, next_color: u8) GcError!void {
        const d = o.data();
        if (o.pmap_ptr) |pmap| {
            for (pmap[0..o.pmap_len]) |off| {
                if (off + 8 <= o.size) {
                    const field_ptr = @as(*?*anyopaque, @ptrCast(@alignCast(d + off)));
                    if (self.getObj(field_ptr.*)) |c| {
                        if (@atomicLoad(u8, &c.color, .seq_cst) != next_color) {
                            _ = @cmpxchgStrong(u8, &c.color, c.color, next_color, .seq_cst, .seq_cst);
                            try self.stack.append(self.alloc, c);
                        }
                    }
                }
            }
        }
        if (o.type == .Array and (o.flags & Obj.FLAG_PTR) != 0) {
            const ptrs = @as([*]?*anyopaque, @ptrCast(@alignCast(d)));
            for (ptrs[0..(o.size / 8)]) |v| {
                if (self.getObj(v)) |c| {
                    if (@atomicLoad(u8, &c.color, .seq_cst) != next_color) {
                        _ = @cmpxchgStrong(u8, &c.color, c.color, next_color, .seq_cst, .seq_cst);
                        try self.stack.append(self.alloc, c);
                    }
                }
            }
        }
    }

    fn fixupGlobalAndStack(self: *Fgc) void {
        // Fixup globals
        for (self.globals) |g| {
            if (g) |ptr| {
                if (ptr.*) |r| {
                    if (self.getObj(r)) |o| {
                        var act = o;
                        if (@atomicLoad(u64, &act.magic, .seq_cst) == Obj.FORWARD_MAGIC) {
                            act = @atomicLoad(?*Obj, &act.forward, .seq_cst).?;
                        }
                        ptr.* = act.data();
                    }
                }
            }
        }

        // Fixup stack
        var rf = self.top;
        while (rf) |curr| {
            for (curr.roots[0..curr.count]) |r_ptr| {
                const r = @as(*?*anyopaque, @ptrCast(@alignCast(r_ptr)));
                if (r.*) |data| {
                    if (self.getObj(data)) |o| {
                        var act = o;
                        if (@atomicLoad(u64, &act.magic, .seq_cst) == Obj.FORWARD_MAGIC) {
                            act = @atomicLoad(?*Obj, &act.forward, .seq_cst).?;
                        }
                        r.* = act.data();
                    }
                }
            }
            rf = curr.next;
        }
    }

    fn evacuate(self: *Fgc, pages: *std.ArrayListUnmanaged(*Page), t: PageType) GcError!std.ArrayListUnmanaged(*Page) {
        if (pages.items.len == 0) return .{};

        var old = std.ArrayListUnmanaged(*Page){};
        try old.appendSlice(self.alloc, pages.items);
        var new_list = std.ArrayListUnmanaged(*Page){};
        var dest = try Page.init(self.alloc, t, 0);
        try self.addPage(&new_list, dest);

        for (old.items) |p| {
            var scan = p.start;
            while (@intFromPtr(scan) < @intFromPtr(p.top)) {
                const o = @as(*Obj, @ptrCast(@alignCast(scan)));
                const fs = o.fullSize();

                if (@atomicLoad(u64, &o.magic, .seq_cst) == Obj.MAGIC and
                    @atomicLoad(u8, &o.color, .seq_cst) == self.color)
                {
                    var target_list = &new_list;
                    var target_dest = &dest;

                    // Promote old objects to tenured generation
                    if (o.age >= AGE_THRESHOLD and t != .Large) {
                        target_list = &self.tenured;
                        if (target_list.items.len == 0) {
                            const new_page = try Page.init(self.alloc, t, 0);
                            try self.addPage(target_list, new_page);
                        }
                        target_dest = &target_list.items[target_list.items.len - 1];
                    }

                    if (target_dest.*.remains() < fs) {
                        target_dest.* = try Page.init(self.alloc, t, 0);
                        try self.addPage(target_list, target_dest.*);
                    }

                    @memcpy(target_dest.*.top[0..fs], scan[0..fs]);
                    const no = @as(*Obj, @ptrCast(@alignCast(target_dest.*.top)));
                    @atomicStore(u64, &o.magic, Obj.FORWARD_MAGIC, .seq_cst);
                    @atomicStore(?*Obj, &o.forward, no, .seq_cst);
                    if (no.age < 255) no.age += 1;

                    target_dest.*.top += fs;
                    target_dest.*.alloc_bytes += fs;
                }
                scan += fs;
            }
        }

        pages.* = new_list;
        return old;
    }

    fn remapAll(self: *Fgc) void {
        inline for (.{ &self.small, &self.medium, &self.large, &self.tenured }) |list| {
            self.remapList(list.items);
        }
    }

    fn remapList(self: *Fgc, pages: []const *Page) void {
        for (pages) |p| {
            var scan = p.start;
            while (@intFromPtr(scan) < @intFromPtr(p.top)) {
                const o = @as(*Obj, @ptrCast(@alignCast(scan)));
                const d = o.data();

                if (o.pmap_ptr) |pmap| {
                    for (pmap[0..o.pmap_len]) |off| {
                        if (off + 8 <= o.size) {
                            const field_ptr = @as(*?*anyopaque, @ptrCast(@alignCast(d + off)));
                            if (self.getObj(field_ptr.*)) |c| {
                                var act = c;
                                if (@atomicLoad(u64, &act.magic, .seq_cst) == Obj.FORWARD_MAGIC) {
                                    act = @atomicLoad(?*Obj, &act.forward, .seq_cst).?;
                                }
                                field_ptr.* = act.data();
                            }
                        }
                    }
                }

                if (o.type == .Array and (o.flags & Obj.FLAG_PTR) != 0) {
                    const ptrs = @as([*]?*anyopaque, @ptrCast(@alignCast(d)));
                    for (ptrs[0..(o.size / 8)]) |*v| {
                        if (self.getObj(v.*)) |c| {
                            var act = c;
                            if (@atomicLoad(u64, &act.magic, .seq_cst) == Obj.FORWARD_MAGIC) {
                                act = @atomicLoad(?*Obj, &act.forward, .seq_cst).?;
                            }
                            v.* = act.data();
                        }
                    }
                }

                scan += o.fullSize();
            }
        }
    }

    /// Get GC statistics for debugging
    pub fn getStats(self: *const Fgc) struct {
        total_allocations: usize,
        total_collections: usize,
        small_pages: usize,
        medium_pages: usize,
        large_pages: usize,
        tenured_pages: usize,
        nursery_pages: usize,
    } {
        return .{
            .total_allocations = self.total_allocations,
            .total_collections = self.total_collections,
            .small_pages = self.small.items.len,
            .medium_pages = self.medium.items.len,
            .large_pages = self.large.items.len,
            .tenured_pages = self.tenured.items.len,
            .nursery_pages = self.nursery.items.len,
        };
    }

    /// Clean shutdown - deallocate all pages
    pub fn deinit(self: *Fgc) void {
        gc_mutex.lock();
        defer gc_mutex.unlock();

        // Deinit all page lists
        inline for (.{ &self.small, &self.medium, &self.large, &self.tenured, &self.nursery, &self.old_pages }) |list| {
            for (list.items) |p| {
                p.deinit(self.alloc);
            }
            list.deinit(self.alloc);
        }

        // Deinit remembered set
        self.remembered_set.deinit(self.alloc);
        self.stack.deinit(self.alloc);

        // Shutdown thread pool
        if (self.pool) |p| {
            p.deinit();
            self.alloc.destroy(p);
            self.pool = null;
        }
    }
};

/// Thread-safe global GC instance
var global_gc: ?Fgc = null;
var global_gc_mutex: std.Thread.Mutex = .{};

/// Initialize global GC safely
pub fn initGlobalGc(alloc: std.mem.Allocator) GcError!void {
    global_gc_mutex.lock();
    defer global_gc_mutex.unlock();

    if (global_gc == null) {
        global_gc = try Fgc.init(alloc);
    }
}

/// Get global GC instance
pub fn getGlobalGc() ?*Fgc {
    global_gc_mutex.lock();
    defer global_gc_mutex.unlock();
    return if (global_gc) |*gc| gc else null;
}

/// Safely shutdown global GC
pub fn deinitGlobalGc() void {
    global_gc_mutex.lock();
    defer global_gc_mutex.unlock();

    if (global_gc) |*gc| {
        gc.deinit();
        global_gc = null;
    }
}
