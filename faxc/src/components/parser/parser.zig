const std = @import("std");
const builtin = @import("builtin");
const json = std.json;

const Token = struct {
    type: []const u8,
    value: []const u8,
    line: usize,
    column: usize,
};

const Parser = struct {
    allocator: std.mem.Allocator,
    tokens: []const Token,
    pos: usize,

    pub fn init(allocator: std.mem.Allocator, tokens: []const Token) Parser {
        return Parser{
            .allocator = allocator,
            .tokens = tokens,
            .pos = 0,
        };
    }

    fn peek(self: *Parser) Token {
        if (self.pos >= self.tokens.len) return Token{ .type = "EOF", .value = "", .line = 0, .column = 0 };
        return self.tokens[self.pos];
    }

    fn eat(self: *Parser) Token {
        const t = self.peek();
        self.pos += 1;
        return t;
    }

    fn expect(self: *Parser, t_type: []const u8) anyerror!Token {
        const t = self.eat();
        if (!std.mem.eql(u8, t.type, t_type)) return error.UnexpectedToken;
        return t;
    }

    fn expectValue(self: *Parser, val: []const u8) anyerror!Token {
        const t = self.eat();
        if (!std.mem.eql(u8, t.value, val)) return error.UnexpectedToken;
        return t;
    }

    pub fn parseProgram(self: *Parser) anyerror!json.Value {
        var body = json.Array.init(self.allocator);
        while (!std.mem.eql(u8, self.peek().type, "EOF")) {
            try body.append(try self.parseStatement());
        }
        var obj = json.ObjectMap.init(self.allocator);
        try obj.put("type", json.Value{ .string = "Program" });
        try obj.put("body", json.Value{ .array = body });
        return json.Value{ .object = obj };
    }

    fn parseStatement(self: *Parser) anyerror!json.Value {
        const t = self.peek();
        var res: json.Value = undefined;
        if (std.mem.eql(u8, t.value, "fn")) {
            res = try self.parseFunction();
        } else if (std.mem.eql(u8, t.value, "struct")) {
            res = try self.parseStruct();
        } else if (std.mem.eql(u8, t.value, "let") or std.mem.eql(u8, t.value, "var")) {
            res = try self.parseVarDecl();
        } else if (std.mem.eql(u8, t.value, "return")) {
            _ = self.eat();
            const expr = try self.parseExpression();
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "ReturnStatement" });
            try obj.put("argument", expr);
            res = json.Value{ .object = obj };
        } else {
            res = try self.parseExpression();
        }
        
        if (std.mem.eql(u8, self.peek().value, ";")) {
            _ = self.eat();
        }
        return res;
    }

    fn parseFunction(self: *Parser) anyerror!json.Value {
        _ = try self.expectValue("fn");
        const name = (try self.expect("Identifier")).value;
        _ = try self.expectValue("(");
        var args = json.Array.init(self.allocator);
        while (!std.mem.eql(u8, self.peek().value, ")")) {
            const p_name = (try self.expect("Identifier")).value;
            _ = try self.expectValue(":");
            const p_type = (try self.expect("Identifier")).value;
            var p_obj = json.ObjectMap.init(self.allocator);
            try p_obj.put("name", json.Value{ .string = p_name });
            try p_obj.put("type", json.Value{ .string = p_type });
            try args.append(json.Value{ .object = p_obj });
            if (std.mem.eql(u8, self.peek().value, ",")) _ = self.eat();
        }
        _ = try self.expectValue(")");
        if (std.mem.eql(u8, self.peek().value, ":")) {
            _ = self.eat();
            _ = (try self.expect("Identifier")).value;
        }
        _ = try self.expectValue("{");
        var body = json.Array.init(self.allocator);
        while (!std.mem.eql(u8, self.peek().value, "}")) {
            try body.append(try self.parseStatement());
        }
        _ = try self.expectValue("}");
        var obj = json.ObjectMap.init(self.allocator);
        try obj.put("type", json.Value{ .string = "FunctionDeclaration" });
        try obj.put("name", json.Value{ .string = name });
        try obj.put("args", json.Value{ .array = args });
        try obj.put("body", json.Value{ .array = body });
        return json.Value{ .object = obj };
    }

    fn parseStruct(self: *Parser) anyerror!json.Value {
        _ = try self.expectValue("struct");
        const name = (try self.expect("Identifier")).value;
        _ = try self.expectValue("{");
        var fields = json.Array.init(self.allocator);
        while (!std.mem.eql(u8, self.peek().value, "}")) {
            const f_name = (try self.expect("Identifier")).value;
            _ = try self.expectValue(":");
            const f_type = (try self.expect("Identifier")).value;
            var f_obj = json.ObjectMap.init(self.allocator);
            try f_obj.put("name", json.Value{ .string = f_name });
            try f_obj.put("type", json.Value{ .string = f_type });
            try fields.append(json.Value{ .object = f_obj });
            if (std.mem.eql(u8, self.peek().value, ",")) _ = self.eat();
        }
        _ = try self.expectValue("}");
        var obj = json.ObjectMap.init(self.allocator);
        try obj.put("type", json.Value{ .string = "StructDeclaration" });
        try obj.put("name", json.Value{ .string = name });
        try obj.put("fields", json.Value{ .array = fields });
        return json.Value{ .object = obj };
    }

    fn parseVarDecl(self: *Parser) anyerror!json.Value {
        _ = self.eat(); // let/var
        const name = (try self.expect("Identifier")).value;
        _ = try self.expectValue("=");
        const initializer = try self.parseExpression();
        var obj = json.ObjectMap.init(self.allocator);
        try obj.put("type", json.Value{ .string = "VariableDeclaration" });
        try obj.put("name", json.Value{ .string = name });
        try obj.put("expr", initializer);
        return json.Value{ .object = obj };
    }

    fn parseExpression(self: *Parser) anyerror!json.Value {
        return try self.parseBinaryExpr(0);
    }

    fn getPrecedence(op: []const u8) i32 {
        if (std.mem.eql(u8, op, "==") or std.mem.eql(u8, op, "!=") or std.mem.eql(u8, op, "<") or std.mem.eql(u8, op, ">") or std.mem.eql(u8, op, "<=") or std.mem.eql(u8, op, ">=")) return 1;
        if (std.mem.eql(u8, op, "+") or std.mem.eql(u8, op, "-")) return 2;
        if (std.mem.eql(u8, op, "*") or std.mem.eql(u8, op, "/")) return 3;
        return 0;
    }

    fn parseBinaryExpr(self: *Parser, min_prec: i32) anyerror!json.Value {
        var left = try self.parsePrimaryExpr();
        while (true) {
            const t = self.peek();
            const prec = getPrecedence(t.value);
            if (prec <= min_prec) break;
            _ = self.eat();
            const right = try self.parseBinaryExpr(prec);
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "BinaryExpression" });
            try obj.put("op", json.Value{ .string = t.value }); // Codegen expects "op"
            try obj.put("left", left);
            try obj.put("right", right);
            left = json.Value{ .object = obj };
        }
        return left;
    }

    fn parsePrimaryExpr(self: *Parser) anyerror!json.Value {
        const t = self.eat();
        if (std.mem.eql(u8, t.type, "Number")) {
            return json.Value{ .string = t.value }; // Codegen expects number as string
        } else if (std.mem.eql(u8, t.type, "String")) {
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "StringLiteral" });
            try obj.put("value", json.Value{ .string = t.value });
            return json.Value{ .object = obj };
        } else if (std.mem.eql(u8, t.type, "Identifier")) {
            if (std.mem.eql(u8, self.peek().value, "(")) {
                _ = self.eat();
                var call_args = json.Array.init(self.allocator);
                while (!std.mem.eql(u8, self.peek().value, ")")) {
                    try call_args.append(try self.parseExpression());
                    if (std.mem.eql(u8, self.peek().value, ",")) _ = self.eat();
                }
                _ = self.eat();
                var obj = json.ObjectMap.init(self.allocator);
                try obj.put("type", json.Value{ .string = "CallExpression" });
                if (std.mem.eql(u8, t.value, "print")) {
                    try obj.put("name", json.Value{ .string = "io::print" });
                    if (call_args.items.len > 0) try obj.put("expr", call_args.items[0]);
                } else {
                    try obj.put("name", json.Value{ .string = t.value });
                    try obj.put("args", json.Value{ .array = call_args });
                }
                return json.Value{ .object = obj };
            }
            return json.Value{ .string = t.value }; // Codegen expects identifier as string
        } else if (std.mem.eql(u8, t.value, "[")) {
            var elements = json.Array.init(self.allocator);
            while (!std.mem.eql(u8, self.peek().value, "]")) {
                try elements.append(try self.parseExpression());
                if (std.mem.eql(u8, self.peek().value, ",")) _ = self.eat();
            }
            _ = self.eat();
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "ArrayLiteral" });
            try obj.put("elements", json.Value{ .array = elements });
            return json.Value{ .object = obj };
        }
        return error.UnexpectedToken;
    }
};

fn printJson(file: std.fs.File, val: json.Value) anyerror!void {
    switch (val) {
        .null => try file.writeAll("null"),
        .bool => |b| try file.writeAll(if (b) "true" else "false"),
        .integer => |i| {
            var buf: [32]u8 = undefined;
            const s = std.fmt.bufPrint(&buf, "{d}", .{i}) catch unreachable;
            try file.writeAll(s);
        },
        .float => |f| {
            var buf: [64]u8 = undefined;
            const s = std.fmt.bufPrint(&buf, "{d}", .{f}) catch unreachable;
            try file.writeAll(s);
        },
        .string => |s| {
            try file.writeAll("\"");
            for (s) |c| {
                switch (c) {
                    '\"' => try file.writeAll("\\\""),
                    '\\' => try file.writeAll("\\\\"),
                    '\n' => try file.writeAll("\\n"),
                    '\t' => try file.writeAll("\\t"),
                    else => try file.writeAll(&[_]u8{c}),
                }
            }
            try file.writeAll("\"");
        },
        .number_string => |ns| try file.writeAll(ns),
        .array => |arr| {
            try file.writeAll("[");
            for (arr.items, 0..) |v, i| {
                if (i > 0) try file.writeAll(",");
                try printJson(file, v);
            }
            try file.writeAll("]");
        },
        .object => |obj| {
            try file.writeAll("{");
            var it = obj.iterator();
            var first = true;
            while (it.next()) |entry| {
                if (!first) try file.writeAll(",");
                first = false;
                try file.writeAll("\"");
                try file.writeAll(entry.key_ptr.*);
                try file.writeAll("\":");
                try printJson(file, entry.value_ptr.*);
            }
            try file.writeAll("}");
        }
    }
}

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();
    
    const args = try std.process.argsAlloc(allocator);
    if (args.len < 2) return;
    
    const file_content = try std.fs.cwd().readFileAlloc(allocator, args[1], 1024 * 1024);
    const tokens_json = try json.parseFromSlice(json.Value, allocator, file_content, .{});
    
    var tokens_list = std.ArrayListUnmanaged(Token){};
    const tokens_arr = if (tokens_json.value == .object)
        tokens_json.value.object.get("tokens").?.array
    else
        tokens_json.value.array;

    for (tokens_arr.items) |t_val| {
        try tokens_list.append(allocator, Token{
            .type = t_val.object.get("type").?.string,
            .value = t_val.object.get("value").?.string,
            .line = 0, .column = 0,
        });
    }
    
    var parser = Parser.init(allocator, tokens_list.items);
    const ast = try parser.parseProgram();
    
    const stdout = std.fs.File{ .handle = if (builtin.target.os.tag == .windows) 0 else 1 };
    try printJson(stdout, ast);
}