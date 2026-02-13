const std = @import("std");

// Color constants for garbage collection phases
const COLOR_M0: u8 = 0x1;
const COLOR_M1: u8 = 0x2;
const COLOR_REMAPPED: u8 = 0x4;

// Magic numbers for object identification
const MAGIC: u64 = 0xDEADC0DECAFEBABE;
const FORWARD_MAGIC: u64 = 0xF084A4D146000000;

// Page size constants
const SMALL_PAGE_SIZE: usize = 2 * 1024 * 1024; // 2MB
const MEDIUM_PAGE_SIZE: usize = 32 * 1024 * 1024; // 32MB

// Size thresholds for page allocation
const SMALL_PAGE_THRESHOLD: usize = 2048; // Objects <= 2KB go to small pages
const MEDIUM_PAGE_THRESHOLD: usize = 256 * 1024; // Objects <= 256KB go to medium pages

pub const ObjType = enum(u8) { Integer, Struct, Array, String };

pub const Obj = struct {
    magic: u64 = MAGIC,
    forward: ?*Obj = null,
    size: usize,
    ptr_map: []const usize = &.{},
    type: ObjType,
    color: u8,
    flags: u8 = 0,

    pub const FLAG_PTR: u8 = 0x1;

    pub fn data(self: *Obj) [*]u8 {
        return @ptrCast(@as([*]u8, @ptrCast(self)) + std.mem.alignForward(usize, @sizeOf(Obj), 16));
    }

    pub fn from(d: ?*anyopaque) ?*Obj {
        if (d == null) return null;
        const o = @as(*Obj, @ptrCast(@alignCast(@as([*]u8, @ptrCast(d.?)) - std.mem.alignForward(usize, @sizeOf(Obj), 16))));
        if (o.magic != MAGIC and o.magic != FORWARD_MAGIC) return null;
        return o;
    }

    pub fn barrier(self: *Obj, current_color: u8) *Obj {
        if (self.color != current_color) return self.slow(current_color);
        return self;
    }

    fn slow(self: *Obj, current_color: u8) *Obj {
        var curr = self;
        if (curr.magic == FORWARD_MAGIC) curr = curr.forward.?;
        if (current_color != COLOR_REMAPPED) curr.color = current_color;
        return curr;
    }

    pub fn fullSize(self: *Obj) usize {
        return std.mem.alignForward(usize, std.mem.alignForward(usize, @sizeOf(Obj), 16) + self.size, 16);
    }
};

const PageType = enum { Small, Medium, Large };

const Page = struct {
    start: [*]u8,
    top: [*]u8,
    end: [*]u8,
    type: PageType,
    alloc_bytes: usize = 0,

    pub fn init(alloc: std.mem.Allocator, t: PageType, sz: usize) !*Page {
        const size: usize = switch (t) {
            .Small => SMALL_PAGE_SIZE,
            .Medium => MEDIUM_PAGE_SIZE,
            .Large => sz,
        };
        const p = try alloc.create(Page);
        const mem = try alloc.alloc(u8, size);
        @memset(mem, 0);
        p.* = .{ .start = mem.ptr, .top = mem.ptr, .end = mem.ptr + size, .type = t };
        return p;
    }

    pub fn remains(self: *Page) usize {
        return @intFromPtr(self.end) - @intFromPtr(self.top);
    }

    pub fn deinit(self: *Page, alloc: std.mem.Allocator) void {
        alloc.free(self.start[0..(@intFromPtr(self.end) - @intFromPtr(self.start))]);
        alloc.destroy(self);
    }
};

pub const Frame = struct { next: ?*Frame, roots: [*]*anyopaque, count: usize };

pub const Fgc = struct {
    alloc: std.mem.Allocator,
    small: std.ArrayListUnmanaged(*Page) = .{},
    medium: std.ArrayListUnmanaged(*Page) = .{},
    large: std.ArrayListUnmanaged(*Page) = .{},
    color: u8 = COLOR_M0,
    phase: enum { Idle, Marking, Relocating } = .Idle,
    stack: std.ArrayListUnmanaged(*Obj) = .{},
    top: ?*Frame = null,
    globals: [256]?*?*anyopaque = [_]?*?*anyopaque{null} ** 256,

    pub fn init(alloc: std.mem.Allocator) !Fgc {
        var self = Fgc{ .alloc = alloc };
        try self.small.append(alloc, try Page.init(alloc, .Small, 0));
        return self;
    }

    pub fn allocObj(self: *Fgc, sz: usize, pmap: []const usize, t: ObjType) !*Obj {
        const full = std.mem.alignForward(usize, std.mem.alignForward(usize, @sizeOf(Obj), 16) + sz, 16);
        const pt: PageType = if (full <= SMALL_PAGE_THRESHOLD) .Small else if (full <= MEDIUM_PAGE_THRESHOLD) .Medium else .Large;
        var list = switch (pt) {
            .Small => &self.small,
            .Medium => &self.medium,
            .Large => &self.large,
        };

        var page: ?*Page = null;
        for (list.items) |p| if (p.remains() >= full) {
            page = p;
            break;
        };
        if (page == null) {
            try self.collect();
            for (list.items) |p| if (p.remains() >= full) {
                page = p;
                break;
            };
        }
        if (page == null) {
            page = try Page.init(self.alloc, pt, full);
            try list.append(self.alloc, page.?);
        }

        const mem = page.?.top;
        page.?.top += full;
        page.?.alloc_bytes += full;
        const o = @as(*Obj, @ptrCast(@alignCast(mem)));
        o.* = .{ .size = sz, .ptr_map = pmap, .type = t, .color = self.color };
        return o;
    }

    fn evacuate(self: *Fgc, pages: *std.ArrayListUnmanaged(*Page), t: PageType) !std.ArrayListUnmanaged(*Page) {
        if (pages.items.len == 0) return .{};
        var old = std.ArrayListUnmanaged(*Page){};
        try old.appendSlice(self.alloc, pages.items);
        var new_list = std.ArrayListUnmanaged(*Page){};
        var dest = try Page.init(self.alloc, t, 0);
        try new_list.append(self.alloc, dest);

        for (old.items) |p| {
            var scan = p.start;
            while (@intFromPtr(scan) < @intFromPtr(p.top)) {
                const o = @as(*Obj, @ptrCast(@alignCast(scan)));
                const fs = o.fullSize();
                if (o.magic == MAGIC and o.color == self.color) {
                    if (dest.remains() < fs) {
                        dest = try Page.init(self.alloc, t, 0);
                        try new_list.append(self.alloc, dest);
                    }
                    @memcpy(dest.top[0..fs], scan[0..fs]);
                    const no = @as(*Obj, @ptrCast(@alignCast(dest.top)));
                    o.magic = FORWARD_MAGIC;
                    o.forward = no;
                    no.color = COLOR_REMAPPED;
                    dest.top += fs;
                    dest.alloc_bytes += fs;
                }
                scan += fs;
            }
        }
        pages.* = new_list;
        return old;
    }

    pub fn collect(self: *Fgc) !void {
        self.phase = .Marking;
        self.color = if (self.color == COLOR_M0) COLOR_M1 else COLOR_M0;

        for (self.globals) |g| if (g) |ptr| if (ptr.*) |r| if (Obj.from(r)) |o| if (o.color != self.color) {
            o.color = self.color;
            try self.stack.append(self.alloc, o);
        };

        var f = self.top;
        while (f) |curr| {
            for (curr.roots[0..curr.count]) |r| if (Obj.from(r)) |o| if (o.color != self.color) {
                o.color = self.color;
                try self.stack.append(self.alloc, o);
            };
            f = curr.next;
        }

        while (self.stack.items.len > 0) {
            const o = self.stack.items[self.stack.items.len - 1];
            self.stack.items.len -= 1;
            const d = o.data();
            for (o.ptr_map) |off| if (off < o.size) {
                if (@as(*?*anyopaque, @ptrCast(@alignCast(d + off))).*) |v| if (Obj.from(v)) |c| if (c.color != self.color) {
                    c.color = self.color;
                    try self.stack.append(self.alloc, c);
                };
            };
            if (o.type == .Array and (o.flags & Obj.FLAG_PTR) != 0) {
                const ptrs = @as([*]?*anyopaque, @ptrCast(@alignCast(d)));
                for (ptrs[0..(o.size / @sizeOf(?*anyopaque))]) |v| if (v) |val| if (Obj.from(val)) |c| if (c.color != self.color) {
                    c.color = self.color;
                    try self.stack.append(self.alloc, c);
                };
            }
        }

        self.phase = .Relocating;
        var old_s = try self.evacuate(&self.small, .Small);
        defer {
            for (old_s.items) |p| p.deinit(self.alloc);
            old_s.deinit(self.alloc);
        }
        var old_m = try self.evacuate(&self.medium, .Medium);
        defer {
            for (old_m.items) |p| p.deinit(self.alloc);
            old_m.deinit(self.alloc);
        }

        for (self.globals) |g| if (g) |ptr| if (ptr.*) |r| if (Obj.from(r)) |o| {
            var actual = o;
            if (actual.magic == FORWARD_MAGIC) actual = actual.forward.?;
            ptr.* = actual.data();
        };

        var rf = self.top;
        while (rf) |curr| {
            for (curr.roots[0..curr.count]) |*r| if (Obj.from(r.*)) |o| {
                var actual = o;
                if (actual.magic == FORWARD_MAGIC) actual = actual.forward.?;
                r.* = actual.data();
            };
            rf = curr.next;
        }

        self.remapAll();
        var i: usize = 0;
        while (i < self.large.items.len) {
            const p = self.large.items[i];
            if (@as(*Obj, @ptrCast(@alignCast(p.start))).color != self.color) {
                _ = self.large.swapRemove(i);
                p.deinit(self.alloc);
            } else i += 1;
        }
        self.phase = .Idle;
    }

    fn remapAll(self: *Fgc) void {
        remapList(self.small.items);
        remapList(self.medium.items);
        remapList(self.large.items);
    }

    fn remapList(pages: []const *Page) void {
        for (pages) |p| {
            var scan = p.start;
            while (@intFromPtr(scan) < @intFromPtr(p.top)) {
                const o = @as(*Obj, @ptrCast(@alignCast(scan)));
                const d = o.data();
                for (o.ptr_map) |off| if (off < o.size) {
                    const field_ptr = @as(*?*anyopaque, @ptrCast(@alignCast(d + off)));
                    if (field_ptr.*) |v| if (Obj.from(v)) |c| {
                        var act = c;
                        if (act.magic == FORWARD_MAGIC) act = act.forward.?;
                        field_ptr.* = act.data();
                    };
                };
                if (o.type == .Array and (o.flags & Obj.FLAG_PTR) != 0) {
                    const ptrs = @as([*]?*anyopaque, @ptrCast(@alignCast(d)));
                    for (ptrs[0..(o.size / @sizeOf(?*anyopaque))]) |*v| if (v.*) |val| if (Obj.from(val)) |c| {
                        var act = c;
                        if (act.magic == FORWARD_MAGIC) act = act.forward.?;
                        v.* = act.data();
                    };
                }
                scan += o.fullSize();
            }
        }
    }
};

var global: ?Fgc = null;

export fn fax_fgc_init() void {
    if (global == null) global = Fgc.init(std.heap.page_allocator) catch unreachable;
}

export fn fax_fgc_register_root(ptr: ?*?*anyopaque, slot: usize) void {
    if (global == null) fax_fgc_init();
    if (slot < 256) global.?.globals[slot] = ptr;
}

export fn fax_fgc_alloc(sz: usize, pmap_ptr: ?[*]const usize, pmap_len: usize) ?*anyopaque {
    if (global == null) fax_fgc_init();
    const o = global.?.allocObj(sz, if (pmap_ptr) |p| p[0..pmap_len] else &.{}, .Struct) catch return null;
    return @ptrCast(o.data());
}

export fn fax_fgc_alloc_string(len: usize) ?*anyopaque {
    if (global == null) fax_fgc_init();
    const o = global.?.allocObj(len + 1, &.{}, .String) catch return null;
    o.data()[len] = 0;
    return @ptrCast(o.data());
}

export fn fax_fgc_alloc_array(esz: usize, count: usize, is_ptr: bool) ?*anyopaque {
    if (global == null) fax_fgc_init();
    var o = global.?.allocObj(esz * count, &.{}, .Array) catch return null;
    if (is_ptr) o.flags |= Obj.FLAG_PTR;
    return @ptrCast(o.data());
}

export fn fax_fgc_collect() void {
    if (global) |*g| g.collect() catch {};
}

export fn fax_fgc_push_frame(ptr: ?*anyopaque) void {
    if (global == null) fax_fgc_init();
    if (ptr) |p| {
        const f = @as(*Frame, @ptrCast(@alignCast(p)));
        f.next = global.?.top;
        global.?.top = f;
    }
}

export fn fax_fgc_pop_frame() void {
    if (global) |*g| if (g.top) |t| {
        g.top = t.next;
    };
}

export fn fax_str_concat(a_ptr: ?*anyopaque, b_ptr: ?*anyopaque) ?*anyopaque {
    if (global == null) fax_fgc_init();
    if (a_ptr == null or b_ptr == null) return null;
    const a = @as([*:0]const u8, @ptrCast(a_ptr));
    const b = @as([*:0]const u8, @ptrCast(b_ptr));
    const la = std.mem.len(a);
    const lb = std.mem.len(b);
    const tot = la + lb;
    const o = global.?.allocObj(tot + 1, &.{}, .String) catch return null;
    const d = o.data();
    @memcpy(d[0..la], a[0..la]);
    @memcpy(d[la..tot], b[0..lb]);
    d[tot] = 0;
    return @ptrCast(d);
}
