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

pub const Fgc = struct {
    alloc: std.mem.Allocator,
    small: std.ArrayListUnmanaged(*Page) = .{},
    medium: std.ArrayListUnmanaged(*Page) = .{},
    large: std.ArrayListUnmanaged(*Page) = .{},
    old_pages: std.ArrayListUnmanaged(*Page) = .{},
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
        const full = std.mem.alignForward(usize, 48 + sz, 16);
        const pt: PageType = if (full <= 2048) .Small else if (full <= 256 * 1024) .Medium else .Large;
        var list = switch (pt) {
            .Small => &self.small,
            .Medium => &self.medium,
            .Large => &self.large,
        };

        var p_ptr: ?*Page = null;
        for (list.items) |p| if (p.remains() >= full) {
            p_ptr = p;
            break;
        };
        if (p_ptr == null) {
            try self.collect();
            for (list.items) |p| if (p.remains() >= full) {
                p_ptr = p;
                break;
            };
        }
        if (p_ptr == null) {
            p_ptr = try Page.init(self.alloc, pt, full);
            try list.append(self.alloc, p_ptr.?);
        }

        const mem = p_ptr.?.top;
        p_ptr.?.top += full;
        p_ptr.?.alloc_bytes += full;
        const o = @as(*Obj, @ptrCast(@alignCast(mem)));
        o.* = .{ .magic = Obj.MAGIC, .forward = null, .size = sz, .pmap_ptr = if (pmap.len > 0) pmap.ptr else null, .pmap_len = pmap.len, .type = t, .color = self.color, .flags = 0, ._pad = undefined };
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
                if (o.magic == Obj.MAGIC and o.color == self.color) {
                    if (dest.remains() < fs) {
                        dest = try Page.init(self.alloc, t, 0);
                        try new_list.append(self.alloc, dest);
                    }
                    @memcpy(dest.top[0..fs], scan[0..fs]);
                    const no = @as(*Obj, @ptrCast(@alignCast(dest.top)));

                    o.magic = Obj.FORWARD_MAGIC;
                    o.forward = no;
                    no.color = self.color;

                    dest.top += fs;
                    dest.alloc_bytes += fs;
                }
                scan += fs;
            }
        }
        pages.* = new_list;
        return old;
    }

    pub fn isManaged(self: *const Fgc, ptr: ?*anyopaque) bool {
        if (ptr == null) return false;
        const addr = @intFromPtr(ptr.?);

        inline for (.{ &self.small, &self.medium, &self.large }) |list| {
            for (list.items) |p| {
                if (addr >= @intFromPtr(p.start) and addr < @intFromPtr(p.top)) return true;
            }
        }

        if (self.phase == .Relocating) {
            for (self.old_pages.items) |p| {
                if (addr >= @intFromPtr(p.start) and addr < @intFromPtr(p.top)) return true;
            }
        }

        return false;
    }

    fn getObj(self: *const Fgc, ptr: ?*anyopaque) ?*Obj {
        if (!self.isManaged(ptr)) return null;
        const o = @as(*Obj, @ptrCast(@alignCast(@as([*]u8, @ptrCast(ptr.?)) - 48)));
        if (o.magic != Obj.MAGIC and o.magic != Obj.FORWARD_MAGIC) return null;
        return o;
    }

    pub fn collect(self: *Fgc) !void {
        self.phase = .Marking;
        self.color = if (self.color == COLOR_M0) COLOR_M1 else COLOR_M0;

        for (self.globals) |g| {
            if (g) |ptr| {
                if (ptr.*) |r| {
                    if (self.getObj(r)) |o| {
                        if (o.color != self.color) {
                            o.color = self.color;
                            try self.stack.append(self.alloc, o);
                        }
                    }
                }
            }
        }

        var f = self.top;
        while (f) |curr| {
            for (curr.roots[0..curr.count]) |r_ptr| {
                const r = @as(*?*anyopaque, @ptrCast(@alignCast(r_ptr)));
                if (r.*) |data| {
                    if (self.getObj(data)) |o| {
                        if (o.color != self.color) {
                            o.color = self.color;
                            try self.stack.append(self.alloc, o);
                        }
                    }
                }
            }
            f = curr.next;
        }

        while (self.stack.items.len > 0) {
            const o = self.stack.items[self.stack.items.len - 1];
            self.stack.items.len -= 1;
            const d = o.data();

            if (o.pmap_ptr) |pmap| {
                for (pmap[0..o.pmap_len]) |off| {
                    if (off + @sizeOf(?*anyopaque) <= o.size) {
                        const field_ptr = @as(*?*anyopaque, @ptrCast(@alignCast(d + off)));
                        if (self.getObj(field_ptr.*)) |c| {
                            if (c.color != self.color) {
                                c.color = self.color;
                                try self.stack.append(self.alloc, c);
                            }
                        }
                    }
                }
            }

            if (o.type == .Array and (o.flags & Obj.FLAG_PTR) != 0) {
                const ptrs = @as([*]?*anyopaque, @ptrCast(@alignCast(d)));
                const count = o.size / @sizeOf(?*anyopaque);
                for (ptrs[0..count]) |v| {
                    if (v) |val| {
                        if (self.getObj(val)) |c| {
                            if (c.color != self.color) {
                                c.color = self.color;
                                try self.stack.append(self.alloc, c);
                            }
                        }
                    }
                }
            }
        }

        self.phase = .Relocating;
        var old_s = try self.evacuate(&self.small, .Small);
        var old_m = try self.evacuate(&self.medium, .Medium);
        try self.old_pages.appendSlice(self.alloc, old_s.items);
        try self.old_pages.appendSlice(self.alloc, old_m.items);

        for (self.globals) |g| {
            if (g) |ptr| {
                if (ptr.*) |r| {
                    if (self.getObj(r)) |o| {
                        var actual = o;
                        if (actual.magic == Obj.FORWARD_MAGIC) actual = actual.forward.?;
                        ptr.* = actual.data();
                    }
                }
            }
        }

        var rf = self.top;
        while (rf) |curr| {
            for (curr.roots[0..curr.count]) |r_ptr| {
                const r = @as(*?*anyopaque, @ptrCast(@alignCast(r_ptr)));
                if (r.*) |data| {
                    if (self.getObj(data)) |o| {
                        var actual = o;
                        if (actual.magic == Obj.FORWARD_MAGIC) actual = actual.forward.?;
                        r.* = actual.data();
                    }
                }
            }
            rf = curr.next;
        }

        self.remapAll();

        for (old_s.items) |p| p.deinit(self.alloc);
        for (old_m.items) |p| p.deinit(self.alloc);
        old_s.deinit(self.alloc);
        old_m.deinit(self.alloc);
        self.old_pages.clearRetainingCapacity();

        var i: usize = 0;
        while (i < self.large.items.len) {
            const p = self.large.items[i];
            const o = @as(*Obj, @ptrCast(@alignCast(p.start)));
            if (o.color != self.color) {
                _ = self.large.swapRemove(i);
                p.deinit(self.alloc);
            } else {
                i += 1;
            }
        }
        self.phase = .Idle;
    }

    fn remapAll(self: *Fgc) void {
        self.remapList(self.small.items);
        self.remapList(self.medium.items);
        self.remapList(self.large.items);
    }

    fn remapList(self: *Fgc, pages: []const *Page) void {
        for (pages) |p| {
            var scan = p.start;
            while (@intFromPtr(scan) < @intFromPtr(p.top)) {
                const o = @as(*Obj, @ptrCast(@alignCast(scan)));
                const d = o.data();
                if (o.pmap_ptr) |pmap| {
                    for (pmap[0..o.pmap_len]) |off| {
                        if (off + @sizeOf(?*anyopaque) <= o.size) {
                            const field_ptr = @as(*?*anyopaque, @ptrCast(@alignCast(d + off)));
                            if (self.getObj(field_ptr.*)) |c| {
                                var act = c;
                                if (act.magic == Obj.FORWARD_MAGIC) act = act.forward.?;
                                field_ptr.* = act.data();
                            }
                        }
                    }
                }
                if (o.type == .Array and (o.flags & Obj.FLAG_PTR) != 0) {
                    const ptrs = @as([*]?*anyopaque, @ptrCast(@alignCast(d)));
                    const count = o.size / @sizeOf(?*anyopaque);
                    for (ptrs[0..count]) |*v| {
                        if (v.*) |val| {
                            if (self.getObj(val)) |c| {
                                var act = c;
                                if (act.magic == Obj.FORWARD_MAGIC) act = act.forward.?;
                                v.* = act.data();
                            }
                        }
                    }
                }
                scan += o.fullSize();
            }
        }
    }
};
