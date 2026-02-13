const std = @import("std");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const allocator = gpa.allocator();
    var list = std.ArrayListUnmanaged(u32){};
    try list.append(allocator, 42);
    std.debug.print("Value: {d}\n", .{list.items[0]});
    list.deinit(allocator);
}