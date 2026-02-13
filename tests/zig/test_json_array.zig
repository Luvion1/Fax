const std = @import("std");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const allocator = gpa.allocator();
    var array = std.json.Array.init(allocator);
    defer array.deinit();
    try array.append(std.json.Value{ .integer = 42 });
}
