const std = @import("std");
const json = std.json;
const Parser = @import("parser.zig").Parser;
const ParseErr = @import("parser.zig").ParseErr;

pub fn parseExpression(self: *Parser) ParseErr!json.Value {
    return try parseBinary(self, 0);
}

fn getPrecedence(op: []const u8) i32 {
    const map = .{ .{ ".", 10 }, .{ "[", 10 }, .{ "*", 7 }, .{ "/", 7 }, .{ "%", 7 }, .{ "+", 6 }, .{ "-", 6 }, .{ "<", 5 }, .{ ">", 5 }, .{ "<=", 5 }, .{ ">=", 5 }, .{ "==", 4 }, .{ "!=", 4 }, .{ "&&", 3 }, .{ "||", 2 }, .{ "..", 1 } };
    inline for (map) |kv| if (std.mem.eql(u8, op, kv[0])) return kv[1];
    return 0;
}

fn mapOperator(op: []const u8) []const u8 {
    const map = .{ .{ "+", "add" }, .{ "-", "sub" }, .{ "*", "mul" }, .{ "/", "sdiv" }, .{ "%", "srem" }, .{ "..", "range" }, .{ "==", "eq" }, .{ "!=", "ne" }, .{ "<", "slt" }, .{ ">", "sgt" }, .{ "<=", "sle" }, .{ ">=", "sge" }, .{ "&&", "land" }, .{ "||", "lor" } };
    inline for (map) |kv| if (std.mem.eql(u8, op, kv[0])) return kv[1];
    return op;
}

pub fn parsePostfix(self: *Parser, left_in: json.Value) ParseErr!json.Value {
    var left = left_in;
    while (true) {
        const t = self.peek();
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
        } else break;
    }
    return left;
}

pub fn parseUnary(self: *Parser) ParseErr!json.Value {
    const t = self.peek();
    
    if (std.mem.eql(u8, t.value, "-")) {
        _ = self.advance();
        const operand = try parseUnary(self);
        var node = try self.createNode("UnaryExpression", t);
        try node.put("op", .{ .string = "neg" });
        try node.put("operand", operand);
        return .{ .object = node };
    }
    
    if (std.mem.eql(u8, t.value, "!")) {
        _ = self.advance();
        const operand = try parseUnary(self);
        var node = try self.createNode("UnaryExpression", t);
        try node.put("op", .{ .string = "not" });
        try node.put("operand", operand);
        return .{ .object = node };
    }

    if (std.mem.eql(u8, t.value, "ref")) {
        _ = self.advance();
        const operand = try parseUnary(self);
        var node = try self.createNode("ReferenceExpression", t);
        try node.put("operand", operand);
        return .{ .object = node };
    }

    if (std.mem.eql(u8, t.value, "*")) {
        _ = self.advance();
        const operand = try parseUnary(self);
        var node = try self.createNode("DereferenceExpression", t);
        try node.put("operand", operand);
        return .{ .object = node };
    }
    
    return try parsePostfix(self, try parsePrimary(self));
}

pub fn parseBinary(self: *Parser, min: i32) ParseErr!json.Value {
    var left = try parseUnary(self);
    
    while (true) {
        const t = self.peek();
        const precedence = getPrecedence(t.value);
        if (precedence <= min) break;

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
    return left;
}

pub fn parseIfExpression(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    _ = try self.expectValue("if");

    const condition = try parseExpression(self);
    _ = try self.expectValue("{");
    
    var thenBody = std.ArrayList(json.Value).init(self.allocator);
    while (!std.mem.eql(u8, self.peek().value, "}")) {
        try thenBody.append(try parseExpression(self));
        if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance();
    }
    _ = try self.expectValue("}");

    var elseBranch = std.ArrayList(json.Value).init(self.allocator);
    if (std.mem.eql(u8, self.peek().value, "else")) {
        _ = self.advance();
        if (std.mem.eql(u8, self.peek().value, "if")) {
            try elseBranch.append(try parseIfExpression(self));
        } else {
            _ = try self.expectValue("{");
            while (!std.mem.eql(u8, self.peek().value, "}")) {
                try elseBranch.append(try parseExpression(self));
                if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance();
            }
            _ = try self.expectValue("}");
        }
    }

    var node = try self.createNode("IfExpression", startToken);
    try node.put("condition", condition);
    try node.put("then_branch", .{ .array = thenBody });
    try node.put("else_branch", .{ .array = elseBranch });
    return .{ .object = node };
}

pub fn parseMatchExpression(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    _ = try self.expectValue("match");
    const target = try parseExpression(self);
    _ = try self.expectValue("{");

    var cases = std.ArrayList(json.Value).init(self.allocator);
    var defaultCase: ?json.Value = null;

    while (!std.mem.eql(u8, self.peek().value, "}")) {
        const t = self.peek();
        if (std.mem.eql(u8, t.value, "case")) {
            _ = self.advance();
            const pattern = try parseExpression(self);
            _ = try self.expectValue("=>");
            var body = std.ArrayList(json.Value).init(self.allocator);
            if (std.mem.eql(u8, self.peek().value, "{")) {
                _ = self.advance();
                while (!std.mem.eql(u8, self.peek().value, "}")) {
                    try body.append(try parseExpression(self));
                    if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance();
                }
                _ = self.advance();
            } else {
                try body.append(try parseExpression(self));
                if (std.mem.eql(u8, self.peek().value, ",")) _ = self.advance();
            }
            
            var caseNode = try self.createNode("MatchCase", t);
            try caseNode.put("pattern", pattern);
            try caseNode.put("body", .{ .array = body });
            try cases.append(.{ .object = caseNode });
        } else if (std.mem.eql(u8, t.value, "default")) {
            _ = self.advance();
            _ = try self.expectValue("=>");
            var body = std.ArrayList(json.Value).init(self.allocator);
            if (std.mem.eql(u8, self.peek().value, "{")) {
                _ = self.advance();
                while (!std.mem.eql(u8, self.peek().value, "}")) {
                    try body.append(try parseExpression(self));
                    if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance();
                }
                _ = self.advance();
            } else {
                try body.append(try parseExpression(self));
                if (std.mem.eql(u8, self.peek().value, ",")) _ = self.advance();
            }
            defaultCase = .{ .array = body };
        } else break;
    }
    _ = try self.expectValue("}");

    var node = try self.createNode("MatchExpression", startToken);
    try node.put("target", target);
    try node.put("cases", .{ .array = cases });
    if (defaultCase) |d| try node.put("default", d) else try node.put("default", .null);
    return .{ .object = node };
}

pub fn parsePrimary(self: *Parser) ParseErr!json.Value {
    const t = self.peek();

    if (std.mem.eql(u8, t.value, "(")) {
        _ = self.advance();
        var expressions = std.ArrayList(json.Value).init(self.allocator);
        while (!std.mem.eql(u8, self.peek().value, ")")) {
            try expressions.append(try parseExpression(self));
            if (std.mem.eql(u8, self.peek().value, ",")) _ = self.advance();
        }
        _ = try self.expectValue(")");
        
        if (expressions.items.len == 1) {
            return expressions.items[0];
        } else {
            var node = try self.createNode("TupleLiteral", t);
            try node.put("elements", .{ .array = expressions });
            return .{ .object = node };
        }
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

    if (std.mem.eql(u8, t.value, "if")) return try parseIfExpression(self);
    if (std.mem.eql(u8, t.value, "match")) return try parseMatchExpression(self);

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