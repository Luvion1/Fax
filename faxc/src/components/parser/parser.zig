const std = @import("std");
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

    fn expect(self: *Parser, t_type: []const u8) !Token {
        const t = self.eat();
        if (!std.mem.eql(u8, t.type, t_type)) return error.UnexpectedToken;
        return t;
    }

    pub fn parseProgram(self: *Parser) !json.Value {
        var body = std.ArrayList(json.Value).init(self.allocator);
        while (!std.mem.eql(u8, self.peek().type, "EOF")) {
            try body.append(try self.parseStatement());
        }
        var obj = json.ObjectMap.init(self.allocator);
        try obj.put("type", json.Value{ .string = "Program" });
        try obj.put("body", json.Value{ .array = body });
        return json.Value{ .object = obj };
    }

    fn parseStatement(self: *Parser) !json.Value {
        const t = self.peek();
        if (std.mem.eql(u8, t.value, "fn")) {
            return try self.parseFunction();
        } else if (std.mem.eql(u8, t.value, "struct")) {
            return try self.parseStruct();
        } else if (std.mem.eql(u8, t.value, "let") or std.mem.eql(u8, t.value, "var")) {
            return try self.parseVarDecl();
        } else if (std.mem.eql(u8, t.value, "return")) {
            _ = self.eat();
            const expr = try self.parseExpression();
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "ReturnStatement" });
            try obj.put("argument", expr);
            return json.Value{ .object = obj };
        } else {
            const expr = try self.parseExpression();
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "ExpressionStatement" });
            try obj.put("expression", expr);
            return json.Value{ .object = obj };
        }
    }

    fn parseFunction(self: *Parser) !json.Value {
        _ = try self.expect("Keyword");
        const name = (try self.expect("Identifier")).value;
        _ = try self.expect("Symbol"); // (
        var params = std.ArrayList(json.Value).init(self.allocator);
        while (!std.mem.eql(u8, self.peek().value, ")")) {
            const p_name = (try self.expect("Identifier")).value;
            _ = try self.expect("Symbol"); // :
            const p_type = (try self.expect("Identifier")).value;
            var p_obj = json.ObjectMap.init(self.allocator);
            try p_obj.put("name", json.Value{ .string = p_name });
            try p_obj.put("type", json.Value{ .string = p_type });
            try params.append(json.Value{ .object = p_obj });
            if (std.mem.eql(u8, self.peek().value, ",")) _ = self.eat();
        }
        _ = try self.expect("Symbol"); // )
        var ret_type: []const u8 = "void";
        if (std.mem.eql(u8, self.peek().value, ":")) {
            _ = self.eat();
            ret_type = (try self.expect("Identifier")).value;
        }
        _ = try self.expect("Symbol"); // {
        var body = std.ArrayList(json.Value).init(self.allocator);
        while (!std.mem.eql(u8, self.peek().value, "}")) {
            try body.append(try self.parseStatement());
        }
        _ = try self.expect("Symbol"); // }
        var obj = json.ObjectMap.init(self.allocator);
        try obj.put("type", json.Value{ .string = "FunctionDeclaration" });
        try obj.put("name", json.Value{ .string = name });
        try obj.put("params", json.Value{ .array = params });
        try obj.put("returnType", json.Value{ .string = ret_type });
        try obj.put("body", json.Value{ .array = body });
        return json.Value{ .object = obj };
    }

    fn parseStruct(self: *Parser) !json.Value {
        _ = try self.expect("Keyword");
        const name = (try self.expect("Identifier")).value;
        _ = try self.expect("Symbol"); // {
        var fields = std.ArrayList(json.Value).init(self.allocator);
        while (!std.mem.eql(u8, self.peek().value, "}")) {
            const f_name = (try self.expect("Identifier")).value;
            _ = try self.expect("Symbol"); // :
            const f_type = (try self.expect("Identifier")).value;
            var f_obj = json.ObjectMap.init(self.allocator);
            try f_obj.put("name", json.Value{ .string = f_name });
            try f_obj.put("type", json.Value{ .string = f_type });
            try fields.append(json.Value{ .object = f_obj });
            if (std.mem.eql(u8, self.peek().value, ",")) _ = self.eat();
        }
        _ = try self.expect("Symbol"); // }
        var obj = json.ObjectMap.init(self.allocator);
        try obj.put("type", json.Value{ .string = "StructDeclaration" });
        try obj.put("name", json.Value{ .string = name });
        try obj.put("fields", json.Value{ .array = fields });
        return json.Value{ .object = obj };
    }

    fn parseVarDecl(self: *Parser) !json.Value {
        const kind = self.eat().value;
        const name = (try self.expect("Identifier")).value;
        _ = try self.expect("Symbol"); // =
        const init_expr = try self.parseExpression();
        var obj = json.ObjectMap.init(self.allocator);
        try obj.put("type", json.Value{ .string = "VariableDeclaration" });
        try obj.put("kind", json.Value{ .string = kind });
        try obj.put("name", json.Value{ .string = name });
        try obj.put("init", init_expr);
        return json.Value{ .object = obj };
    }

    fn parseExpression(self: *Parser) !json.Value {
        return try self.parseBinaryExpr(0);
    }

    fn getPrecedence(op: []const u8) i32 {
        if (std.mem.eql(u8, op, "==") or std.mem.eql(u8, op, "!=") or std.mem.eql(u8, op, "<") or std.mem.eql(u8, op, ">") or std.mem.eql(u8, op, "<=") or std.mem.eql(u8, op, ">=")) return 1;
        if (std.mem.eql(u8, op, "+") or std.mem.eql(u8, op, "-")) return 2;
        if (std.mem.eql(u8, op, "*") or std.mem.eql(u8, op, "/")) return 3;
        return 0;
    }

    fn parseBinaryExpr(self: *Parser, min_prec: i32) !json.Value {
        var left = try self.parsePrimaryExpr();
        while (true) {
            const t = self.peek();
            if (!std.mem.eql(u8, t.type, "Operator") and !std.mem.eql(u8, t.type, "Symbol")) break;
            const prec = getPrecedence(t.value);
            if (prec <= min_prec) break;
            _ = self.eat();
            const right = try self.parseBinaryExpr(prec);
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "BinaryExpression" });
            try obj.put("operator", json.Value{ .string = t.value });
            try obj.put("left", left);
            try obj.put("right", right);
            left = json.Value{ .object = obj };
        }
        return left;
    }

    fn parsePrimaryExpr(self: *Parser) !json.Value {
        const t = self.eat();
        if (std.mem.eql(u8, t.type, "Number")) {
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "Literal" });
            try obj.put("value", json.Value{ .number_string = t.value });
            try obj.put("rawType", json.Value{ .string = "int" });
            return json.Value{ .object = obj };
        } else if (std.mem.eql(u8, t.type, "String")) {
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "Literal" });
            try obj.put("value", json.Value{ .string = t.value });
            try obj.put("rawType", json.Value{ .string = "string" });
            return json.Value{ .object = obj };
        } else if (std.mem.eql(u8, t.type, "Identifier")) {
            if (std.mem.eql(u8, self.peek().value, "(")) {
                _ = self.eat();
                var args = std.ArrayList(json.Value).init(self.allocator);
                while (!std.mem.eql(u8, self.peek().value, ")")) {
                    try args.append(try self.parseExpression());
                    if (std.mem.eql(u8, self.peek().value, ",")) _ = self.eat();
                }
                _ = self.eat();
                var obj = json.ObjectMap.init(self.allocator);
                try obj.put("type", json.Value{ .string = "CallExpression" });
                try obj.put("callee", json.Value{ .string = t.value });
                try obj.put("arguments", json.Value{ .array = args });
                return json.Value{ .object = obj };
            } else if (std.mem.eql(u8, self.peek().value, "{")) {
                _ = self.eat();
                var fields = json.ObjectMap.init(self.allocator);
                while (!std.mem.eql(u8, self.peek().value, "}")) {
                    const f_name = (try self.expect("Identifier")).value;
                    _ = try self.expect("Symbol"); // :
                    try fields.put(f_name, try self.parseExpression());
                    if (std.mem.eql(u8, self.peek().value, ",")) _ = self.eat();
                }
                _ = self.eat();
                var obj = json.ObjectMap.init(self.allocator);
                try obj.put("type", json.Value{ .string = "StructInitialization" });
                try obj.put("structName", json.Value{ .string = t.value });
                try obj.put("fields", json.Value{ .object = fields });
                return json.Value{ .object = obj };
            } else if (std.mem.eql(u8, self.peek().value, ".")) {
                const object_name = t.value;
                _ = self.eat();
                const member = (try self.expect("Identifier")).value;
                var obj = json.ObjectMap.init(self.allocator);
                try obj.put("type", json.Value{ .string = "MemberExpression" });
                try obj.put("object", json.Value{ .string = object_name });
                try obj.put("property", json.Value{ .string = member });
                return json.Value{ .object = obj };
            }
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "Identifier" });
            try obj.put("name", json.Value{ .string = t.value });
            return json.Value{ .object = obj };
        } else if (std.mem.eql(u8, t.value, "[")) {
            var elements = std.ArrayList(json.Value).init(self.allocator);
            while (!std.mem.eql(u8, self.peek().value, "]")) {
                try elements.append(try self.parseExpression());
                if (std.mem.eql(u8, self.peek().value, ",")) _ = self.eat();
            }
            _ = self.eat();
            var obj = json.ObjectMap.init(self.allocator);
            try obj.put("type", json.Value{ .string = "ArrayExpression" });
            try obj.put("elements", json.Value{ .array = elements });
            return json.Value{ .object = obj };
        }
        return error.UnexpectedToken;
    }
};

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();
    const args = try std.process.argsAlloc(allocator);
    if (args.len < 2) return;
    const file_content = try std.fs.cwd().readFileAlloc(allocator, args[1], 1024 * 1024);
    const tokens_json = try json.parseFromSlice(json.Value, allocator, file_content, .{});
    var tokens = std.ArrayList(Token).init(allocator);
    const tokens_arr = if (tokens_json.value == .object) tokens_json.value.object.get("tokens").?.array else tokens_json.value.array;
    for (tokens_arr.items) |t_val| {
        try tokens.append(Token{ .type = t_val.object.get("type").?.string, .value = t_val.object.get("value").?.string, .line = 0, .column = 0 });
    }
    var parser = Parser.init(allocator, tokens.items);
    const ast = try parser.parseProgram();
    try json.stringify(ast, .{}, std.fs.File.stdout().writer());
}
