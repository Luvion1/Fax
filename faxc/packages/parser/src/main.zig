const std = @import("std");
const json_api = @import("api/json_api.zig");

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const alloc = arena.allocator();

    const args = try std.process.argsAlloc(alloc);
    if (args.len < 2) {
        std.debug.print("Usage: {s} <input_file>\n", .{args[0]});
        std.process.exit(1);
    }

    const result = try json_api.parseJson(alloc, args[1]);
    const stdout = std.io.getStdOut().writer();
    try stdout.print("{s}\n", .{result});
}
