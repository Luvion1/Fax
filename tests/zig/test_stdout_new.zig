const std = @import("std");

pub fn main() !void {
    const stdout = std.fs.stdout().writer();
    try stdout.writeAll("hello from std.fs.stdout");
}
