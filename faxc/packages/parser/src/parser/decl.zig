const std = @import("std");
const json = std.json;
const Parser = @import("parser.zig").Parser;
const ParseErr = @import("parser.zig").ParseErr;
const Expr = @import("expr.zig");
const Stmt = @import("stmt.zig");
const Types = @import("type.zig");

pub fn parseModule(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    _ = try self.expectValue("module");
    const name = (try self.expect(.Identifier)).string;
    if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance();

    var node = try self.createNode("ModuleDeclaration", startToken);
    try node.put("name", .{ .string = name });
    return .{ .object = node };
}

pub fn parseUse(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    _ = try self.expectValue("use");
    var path = std.ArrayList(u8).init(self.allocator);
    while (true) {
        try path.appendSlice((try self.expect(.Identifier)).string);
        if (std.mem.eql(u8, self.peek().value, "::")) {
            _ = self.advance();
            try path.appendSlice("::");
        } else break;
    }
    if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance();

    var node = try self.createNode("ImportDeclaration", startToken);
    try node.put("path", .{ .string = try path.toOwnedSlice() });
    return .{ .object = node };
}

pub fn parseFunction(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    var isExtern = false;
    if (std.mem.eql(u8, self.peek().value, "extern")) {
        _ = self.advance();
        isExtern = true;
    }
    _ = try self.expectValue("fn");
    const name = (try self.expect(.Identifier)).string;
    _ = try self.expectValue("(");

    var args = std.ArrayList(json.Value).init(self.allocator);
    while (!std.mem.eql(u8, self.peek().value, ")")) {
        const argToken = self.peek();
        const argName = (try self.expect(.Identifier)).string;
        _ = try self.expectValue(":");
        const argType = try Types.parseType(self);

        var node = try self.createNode("FunctionArgument", argToken);
        try node.put("name", .{ .string = argName });
        try node.put("p_type", argType);
        try args.append(.{ .object = node });

        if (std.mem.eql(u8, self.peek().value, ",")) _ = self.advance();
    }
    _ = try self.expectValue(")");

    var returnType: []const u8 = "void";
    if (std.mem.eql(u8, self.peek().value, "->") or std.mem.eql(u8, self.peek().value, ":")) {
        _ = self.advance();
        returnType = (try Types.parseType(self)).string;
    }

    var body = std.ArrayList(json.Value).init(self.allocator);
    if (!isExtern) {
        _ = try self.expectValue("{");
        while (!std.mem.eql(u8, self.peek().value, "}")) {
            try body.append(try Stmt.parseStatement(self));
        }
        _ = try self.expectValue("}");
    } else if (std.mem.eql(u8, self.peek().value, ";")) {
        _ = self.advance();
    }

    var node = try self.createNode("FunctionDeclaration", startToken);
    try node.put("name", .{ .string = name });
    try node.put("returnType", .{ .string = returnType });
    try node.put("args", .{ .array = args });
    try node.put("body", .{ .array = body });
    try node.put("is_extern", .{ .bool = isExtern });
    return .{ .object = node };
}

pub fn parseStruct(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    _ = try self.expectValue("struct");
    const name = (try self.expect(.Identifier)).string;
    _ = try self.expectValue("{");

    var fields = std.ArrayList(json.Value).init(self.allocator);
    while (!std.mem.eql(u8, self.peek().value, "}")) {
        const fieldToken = self.peek();
        const fieldName = (try self.expect(.Identifier)).string;
        _ = try self.expectValue(":");
        const fieldType = try Types.parseType(self);

        var node = try self.createNode("StructField", fieldToken);
        try node.put("name", .{ .string = fieldName });
        try node.put("type", fieldType);
        try fields.append(.{ .object = node });

        if (std.mem.eql(u8, self.peek().value, ",")) _ = self.advance();
    }
    _ = try self.expectValue("}");

    var node = try self.createNode("StructDeclaration", startToken);
    try node.put("name", .{ .string = name });
    try node.put("fields", .{ .array = fields });
    return .{ .object = node };
}

pub fn parseVariable(self: *Parser, isConstant: bool) ParseErr!json.Value {
    const startToken = self.peek();
    if (isConstant) _ = try self.expectValue("const") else _ = try self.expectValue("let");

    var mutable = false;
    if (!isConstant and std.mem.eql(u8, self.peek().value, "mut")) {
        _ = self.advance();
        mutable = true;
    }
    const name = (try self.expect(.Identifier)).string;
    var typeNode: json.Value = .{ .string = "" };
    if (std.mem.eql(u8, self.peek().value, ":")) {
        _ = self.advance();
        typeNode = try Types.parseType(self);
    }
    _ = try self.expectValue("=");
    const expression = try Expr.parseExpression(self);
    if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance();

    var node = try self.createNode("VariableDeclaration", startToken);
    try node.put("name", .{ .string = name });
    try node.put("is_mutable", .{ .bool = mutable });
    try node.put("var_type", typeNode);
    try node.put("expr", expression);
    return .{ .object = node };
}
