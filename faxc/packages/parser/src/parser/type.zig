const std = @import("std");
const json = std.json;
const Parser = @import("parser.zig").Parser;
const ParseErr = @import("parser.zig").ParseErr;

pub fn parseType(self: *Parser) ParseErr!json.Value {
    const tok = self.advance();
    if (tok.type != .Identifier and tok.type != .Keyword) return error.SyntaxError;
    var full = std.ArrayList(u8).init(self.allocator);
    try full.appendSlice(tok.value);
    while (std.mem.eql(u8, self.peek().value, "[")) {
        _ = self.advance();
        try full.appendSlice("[");
        _ = try self.expectValue("]");
        try full.appendSlice("]");
    }
    return .{ .string = try full.toOwnedSlice() };
}
