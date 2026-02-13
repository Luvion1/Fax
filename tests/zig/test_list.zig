const std = @import("std");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const allocator = gpa.allocator();
    var list = std.ArrayList(u32).init(allocator);
    defer list.deinit();
    try list.append(42);
    std.debug.print("Value: {d}\n", .{list.items[0]});
}