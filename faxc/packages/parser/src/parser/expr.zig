const std = @import("std");
const json = std.json;
const Parser = @import("parser.zig").Parser;
const ParseErr = @import("parser.zig").ParseErr;

pub fn parseExpression(self: *Parser) ParseErr!json.Value {
    return try parseBinary(self, 0);
}

fn getPrecedence(op: []const u8) i32 {
    const map = .{ .{ "[", 5 }, .{ ".", 6 }, .{ "*", 4 }, .{ "/", 4 }, .{ "+", 3 }, .{ "-", 3 }, .{ "<", 2 }, .{ ">", 2 }, .{ "<=", 2 }, .{ ">=", 2 }, .{ "==", 1 }, .{ "!=", 1 } };
    inline for (map) |kv| if (std.mem.eql(u8, op, kv[0])) return kv[1];
    return 0;
}

fn mapOperator(op: []const u8) []const u8 {
    const map = .{ .{ "+", "add" }, .{ "-", "sub" }, .{ "*", "mul" }, .{ "/", "sdiv" }, .{ "==", "eq" }, .{ "!=", "ne" }, .{ "<", "slt" }, .{ ">", "sgt" }, .{ "<=", "sle" }, .{ ">=", "sge" } };
    inline for (map) |kv| if (std.mem.eql(u8, op, kv[0])) return kv[1];
    return op;
}

pub fn parseBinary(self: *Parser, min: i32) ParseErr!json.Value {
    var left = try parsePrimary(self);
    while (true) {
        const t = self.peek();
        const precedence = getPrecedence(t.value);
        if (precedence <= min) break;

        if (std.mem.eql(u8, t.value, "[")) {
            _ = self.advance();
            const index = try parseExpression(self);
            _ = try self.expectValue("]");

            var node = try self.createNode("IndexAccess", t);
            try node.put("base", left);
            try node.put("index", index);
            left = .{ .object = node };
        } else if (std.mem.eql(u8, t.value, ".")) {
            _ = self.advance();
            const field = self.advance().value;

            var node = try self.createNode("MemberAccess", t);
            try node.put("base", left);
            try node.put("field", .{ .string = field });
            left = .{ .object = node };
        } else {
            _ = self.advance();
            const right = try parseBinary(self, precedence);

            const isComparison = std.mem.eql(u8, t.value, "==") or std.mem.eql(u8, t.value, "!=") or
                std.mem.eql(u8, t.value, "<") or std.mem.eql(u8, t.value, ">") or
                std.mem.eql(u8, t.value, "<=") or std.mem.eql(u8, t.value, ">=");

            var node = try self.createNode(if (isComparison) "ComparisonExpression" else "BinaryExpression", t);
            try node.put("op", .{ .string = mapOperator(t.value) });
            try node.put("left", left);
            try node.put("right", right);
            left = .{ .object = node };
        }
    }
    return left;
}

pub fn parsePrimary(self: *Parser) ParseErr!json.Value {
    const t = self.peek();

    if (std.mem.eql(u8, t.value, "(")) {
        _ = self.advance();
        const expression = try parseExpression(self);
        _ = try self.expectValue(")");
        return expression;
    }

    if (t.type == .Number) {
        _ = self.advance();
        var node = try self.createNode("NumberLiteral", t);
        try node.put("value", .{ .string = t.value });
        return .{ .object = node };
    }

    if (t.type == .Boolean) {
        _ = self.advance();
        var node = try self.createNode("Boolean", t);
        try node.put("value", .{ .string = t.value });
        return .{ .object = node };
    }

    if (t.type == .Null) {
        _ = self.advance();
        const node = try self.createNode("NullLiteral", t);
        return .{ .object = node };
    }

    if (t.type == .String) {
        _ = self.advance();
        var node = try self.createNode("StringLiteral", t);
        try node.put("value", .{ .string = t.value });
        return .{ .object = node };
    }

    if (t.type == .Identifier) {
        _ = self.advance();
        if (std.mem.eql(u8, self.peek().value, "(")) {
            _ = self.advance();
            var arguments = std.ArrayList(json.Value).init(self.allocator);
            while (!std.mem.eql(u8, self.peek().value, ")")) {
                try arguments.append(try parseExpression(self));
                if (std.mem.eql(u8, self.peek().value, ",")) _ = self.advance();
            }
            _ = self.advance();

            var node = try self.createNode("CallExpression", t);
            try node.put("name", .{ .string = t.value });
            try node.put("args", .{ .array = arguments });
            return .{ .object = node };
        }

        if (std.mem.eql(u8, self.peek().value, "{") and self.peekAt(1).type == .Identifier and self.peekAt(2).type == .Colon) {
            _ = self.advance();
            var fields = std.ArrayList(json.Value).init(self.allocator);
            while (!std.mem.eql(u8, self.peek().value, "}")) {
                const nameToken = self.peek();
                const fieldName = (try self.expect(.Identifier)).string;
                _ = try self.expectValue(":");
                const expression = try parseExpression(self);

                var fieldNode = try self.createNode("StructField", nameToken);
                try fieldNode.put("name", .{ .string = fieldName });
                try fieldNode.put("expr", expression);
                try fields.append(.{ .object = fieldNode });

                if (std.mem.eql(u8, self.peek().value, ",")) _ = self.advance();
            }
            _ = self.advance();

            var node = try self.createNode("StructLiteral", t);
            try node.put("name", .{ .string = t.value });
            try node.put("fields", .{ .array = fields });
            return .{ .object = node };
        }

        var identifier = try self.createNode("Identifier", t);
        try identifier.put("value", .{ .string = t.value });
        return .{ .object = identifier };
    }

    if (std.mem.eql(u8, t.value, "[")) {
        _ = self.advance();
        var elements = std.ArrayList(json.Value).init(self.allocator);
        while (!std.mem.eql(u8, self.peek().value, "]")) {
            try elements.append(try parseExpression(self));
            if (std.mem.eql(u8, self.peek().value, ",")) _ = self.advance();
        }
        _ = self.advance();

        var node = try self.createNode("ArrayLiteral", t);
        try node.put("elements", .{ .array = elements });
        return .{ .object = node };
    }

    return error.SyntaxError;
}
