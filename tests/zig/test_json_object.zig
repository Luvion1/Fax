const std = @import("std");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const allocator = gpa.allocator();
    var obj = std.json.ObjectMap.init(allocator);
    defer obj.deinit();
    try obj.put("key", std.json.Value{ .integer = 42 });
}
