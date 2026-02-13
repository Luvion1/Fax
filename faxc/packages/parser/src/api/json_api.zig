const std = @import("std");
const json = std.json;
const Parser = @import("../parser/parser.zig").Parser;
const Token = @import("../parser/token.zig").Token;
const TokenType = @import("../parser/token.zig").TokenType;

pub fn parseJson(allocator: std.mem.Allocator, input_path: []const u8) ![]u8 {
    const content = try std.fs.cwd().readFileAlloc(allocator, input_path, 1024 * 1024);
    const j = try json.parseFromSlice(json.Value, allocator, content, .{});

    var tokens = std.ArrayList(Token).init(allocator);
    defer tokens.deinit();

    const source_file = if (j.value == .object) j.value.object.get("source_file") else null;
    const j_tokens = if (j.value == .array) j.value.array else j.value.object.get("tokens").?.array;
    for (j_tokens.items) |v| {
        const line = if (v.object.get("line")) |l| @as(usize, @intCast(l.integer)) else 0;
        const col = if (v.object.get("col")) |c| @as(usize, @intCast(c.integer)) else 0;
        try tokens.append(Token.init(TokenType.from(v.object.get("type").?.string), v.object.get("val").?.string, line, col));
    }

    var parser = Parser.init(allocator, tokens.items);
    var ast = try parser.parse();

    if (source_file) |sf| {
        try ast.object.put("source_file", sf);
    }

    var output_buffer = std.ArrayList(u8).init(allocator);
    try json.stringify(ast, .{}, output_buffer.writer());

    return output_buffer.toOwnedSlice();
}
