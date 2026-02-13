const std = @import("std");

pub const PageType = enum { Small, Medium, Large };

pub const Page = struct {
    start: [*]u8,
    top: [*]u8,
    end: [*]u8,
    type: PageType,
    alloc_bytes: usize = 0,

    pub fn init(alloc: std.mem.Allocator, t: PageType, sz: usize) !*Page {
        const size: usize = switch (t) {
            .Small => 2 * 1024 * 1024,
            .Medium => 32 * 1024 * 1024,
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
