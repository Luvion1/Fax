const std = @import("std");
const json = std.json;
const Parser = @import("parser.zig").Parser;
const ParseErr = @import("parser.zig").ParseErr;

pub fn parseType(self: *Parser) ParseErr!json.Value {
    const t = self.peek();
    
    // Modern reference syntax: 'ref T' instead of '&mut T'
    if (std.mem.eql(u8, t.value, "ref")) {
        _ = self.advance();
        const inner = try parseType(self);
        var full = std.ArrayList(u8).init(self.allocator);
        try full.appendSlice("ref ");
        try full.appendSlice(inner.string);
        return .{ .string = try full.toOwnedSlice() };
    }

    const tok = self.advance();
    if (tok.type != .Identifier and tok.type != .Keyword) return error.SyntaxError;
    
    var full = std.ArrayList(u8).init(self.allocator);
    try full.appendSlice(tok.value);
    
    // Array syntax: T[]
    while (std.mem.eql(u8, self.peek().value, "[")) {
        _ = self.advance();
        try full.appendSlice("[");
        if (!std.mem.eql(u8, self.peek().value, "]")) {
            const inner = try parseType(self);
            try full.appendSlice(inner.string);
            if (std.mem.eql(u8, self.peek().value, ";")) {
                _ = self.advance();
                try full.appendSlice(";");
                const size = self.advance().value;
                try full.appendSlice(size);
            }
        }
        _ = try self.expectValue("]");
        try full.appendSlice("]");
    }
    return .{ .string = try full.toOwnedSlice() };
}
