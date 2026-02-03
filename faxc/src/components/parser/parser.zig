const std = @import("std");
const json = std.json;

const Token = struct {
    type: []const u8,
    value: []const u8,
    line: usize,
    column: usize,
};

const ObjectMap = std.json.ObjectMap; 
const JsonArray = std.json.Array;

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

    fn peekAt(self: *Parser, offset: usize) Token {
        if (self.pos + offset >= self.tokens.len) return Token{ .type = "EOF", .value = "", .line = 0, .column = 0 };
        return self.tokens[self.pos + offset];
    }

    fn advance(self: *Parser) Token {
        const t = self.peek();
        if (self.pos < self.tokens.len) self.pos += 1;
        return t;
    }

    fn check(self: *Parser, val: []const u8) bool {
        return std.mem.eql(u8, self.peek().value, val);
    }

    fn checkType(self: *Parser, t_type: []const u8) bool {
        return std.mem.eql(u8, self.peek().type, t_type);
    }

    fn consume(self: *Parser, val: []const u8) !Token {
        if (self.check(val)) return self.advance();
        return error.UnexpectedToken;
    }

    pub fn parseProgram(self: *Parser) !std.json.Value {
        var body = JsonArray.init(self.allocator);
        while (self.pos < self.tokens.len) {
            if (self.checkType("EOF")) break;
            const decl = try self.parseDeclaration();
            if (decl == .array) {
                for (decl.array.items) |item| try body.append(item);
            } else {
                try body.append(decl);
            }
        }
        var obj = ObjectMap.init(self.allocator);
        try obj.put("type", std.json.Value{ .string = "Program" });
        try obj.put("body", std.json.Value{ .array = body });
        return std.json.Value{ .object = obj };
    }

    fn parseDeclaration(self: *Parser) anyerror!std.json.Value {
        if (self.check("fn")) return self.parseFunction(false);
        if (self.check("extern")) {
            _ = self.advance();
            return self.parseFunction(true);
        }
        if (self.check("struct")) return self.parseStruct();
        if (self.check("class")) return self.parseClass();
        return self.parseStatement();
    }

    fn parseClass(self: *Parser) !std.json.Value {
        _ = try self.consume("class");
        const name = self.advance().value;
        _ = try self.consume("{");
        
        var fields = JsonArray.init(self.allocator);
        var methods = JsonArray.init(self.allocator);
        
        while (!self.check("}") and !self.checkType("EOF")) {
            if (self.check("let") or self.check("const")) {
                try fields.append(try self.parseVariableDecl(self.check("const")));
            } else if (self.check("fn")) {
                try methods.append(try self.parseFunction(false));
            } else {
                std.debug.print("Unexpected token in class: {s} ({s})\n", .{self.peek().value, self.peek().type});
                return error.UnexpectedToken;
            }
        }
        _ = try self.consume("}");
        
        var obj = ObjectMap.init(self.allocator);
        try obj.put("type", std.json.Value{ .string = "ClassDeclaration" });
        try obj.put("name", std.json.Value{ .string = name });
        try obj.put("fields", std.json.Value{ .array = fields });
        try obj.put("methods", std.json.Value{ .array = methods });
        return std.json.Value{ .object = obj };
    }

    fn parseFunction(self: *Parser, is_extern: bool) !std.json.Value {
        _ = try self.consume("fn");
        const name = self.advance().value;
        var args = JsonArray.init(self.allocator);
        if (self.check("(")) {
            _ = self.advance();
            while (!self.check(")") and !self.checkType("EOF")) {
                const arg_name = self.advance().value;
                var arg_type: []const u8 = "any";
                if (self.check(":")) {
                    _ = self.advance();
                    arg_type = self.advance().value;
                }
                var arg_obj = ObjectMap.init(self.allocator);
                try arg_obj.put("name", std.json.Value{.string = arg_name});
                try arg_obj.put("type", std.json.Value{.string = arg_type});
                try args.append(std.json.Value{.object = arg_obj});
                if (self.check(",")) _ = self.advance();
            }
            _ = try self.consume(")");
        }
        var obj = ObjectMap.init(self.allocator);
        try obj.put("type", std.json.Value{ .string = "FunctionDeclaration" });
        try obj.put("name", std.json.Value{ .string = name });
        try obj.put("args", std.json.Value{ .array = args });
        try obj.put("isExtern", std.json.Value{ .bool = is_extern });

        if (self.check(":") or self.checkType("ReturnType")) {
             _ = self.advance(); 
             const ret_type = self.advance().value;
             try obj.put("returnType", std.json.Value{.string = ret_type});
        }

        if (self.check("{")) {
            try obj.put("body", try self.parseBlock());
        } else {
            if (self.check(";")) _ = self.advance();
            try obj.put("body", std.json.Value{ .null = {} });
        }
        return std.json.Value{ .object = obj };
    }

    fn parseStruct(self: *Parser) !std.json.Value {
        _ = try self.consume("struct");
        const name = self.advance().value;
        _ = try self.consume("{");
        var fields = JsonArray.init(self.allocator);
        while (!self.check("}") and !self.checkType("EOF")) {
            const f_name = self.advance().value;
            _ = try self.consume(":");
            const f_type = self.advance().value; 
            var f_obj = ObjectMap.init(self.allocator);
            try f_obj.put("name", std.json.Value{.string = f_name});
            try f_obj.put("type", std.json.Value{.string = f_type});
            try fields.append(std.json.Value{.object = f_obj});
            if (self.check(",")) _ = self.advance();
        }
        _ = try self.consume("}");
        var obj = ObjectMap.init(self.allocator);
        try obj.put("type", std.json.Value{ .string = "StructDeclaration" });
        try obj.put("name", std.json.Value{ .string = name });
        try obj.put("fields", std.json.Value{ .array = fields });
        return std.json.Value{ .object = obj };
    }

    fn parseBlock(self: *Parser) anyerror!std.json.Value {
        _ = try self.consume("{");
        var stmts = JsonArray.init(self.allocator);
        while (!self.check("}") and !self.checkType("EOF")) {
            const s = try self.parseStatement();
            if (s == .array) {
                for (s.array.items) |item| try stmts.append(item);
            } else {
                try stmts.append(s);
            }
        }
        _ = try self.consume("}");
        return std.json.Value{ .array = stmts };
    }

    fn parseStatement(self: *Parser) anyerror!std.json.Value {
        if (self.check("let") or self.check("const")) return self.parseVariableDecl(self.check("const"));
        if (self.check("if")) return self.parseIf();
        if (self.check("match")) return self.parseMatch();
        if (self.check("while")) return self.parseWhile();
        if (self.check("for")) return self.parseFor();
        if (self.check("print")) return self.parsePrint();
        if (self.check("return")) {
            _ = self.advance();
            var val = std.json.Value{.null = {}};
            if (!self.check(";")) val = try self.parseExpression();
            if (self.check(";")) _ = self.advance();
            var obj = ObjectMap.init(self.allocator);
            try obj.put("type", std.json.Value{.string = "ReturnStatement"});
            try obj.put("expr", val);
            return std.json.Value{.object = obj};
        }
        const expr = try self.parseExpression();
        if (self.check(";")) _ = self.advance();
        return expr;
    }

    fn parseExpression(self: *Parser) anyerror!std.json.Value {
        return self.parseAssignmentExpr();
    }

    fn parseAssignmentExpr(self: *Parser) anyerror!std.json.Value {
        const left = try self.parseLogicalOr();
        if (self.check("=") and !self.check("==")) {
            _ = self.advance();
            const right = try self.parseExpression();
            var obj = ObjectMap.init(self.allocator);
            try obj.put("type", std.json.Value{ .string = "Assignment" });
            try obj.put("target", left);
            try obj.put("expr", right);
            return std.json.Value{ .object = obj };
        }
        return left;
    }

    fn parseLogicalOr(self: *Parser) anyerror!std.json.Value {
        var left = try self.parseLogicalAnd();
        while (self.check("||")) {
            _ = self.advance();
            const right = try self.parseLogicalAnd();
            var obj = ObjectMap.init(self.allocator);
            try obj.put("type", std.json.Value{.string = "LogicalExpression"});
            try obj.put("left", left);
            try obj.put("op", std.json.Value{.string = "or"});
            try obj.put("right", right);
            left = std.json.Value{.object = obj};
        }
        return left;
    }

    fn parseLogicalAnd(self: *Parser) anyerror!std.json.Value {
        var left = try self.parseEquality();
        while (self.check("&&")) {
            _ = self.advance();
            const right = try self.parseEquality();
            var obj = ObjectMap.init(self.allocator);
            try obj.put("type", std.json.Value{.string = "LogicalExpression"});
            try obj.put("left", left);
            try obj.put("op", std.json.Value{.string = "and"});
            try obj.put("right", right);
            left = std.json.Value{.object = obj};
        }
        return left;
    }

    fn parseEquality(self: *Parser) anyerror!std.json.Value {
        var left = try self.parseRelational();
        while (self.check("==") or self.check("!=")) {
            const op_val = self.advance().value;
            const right = try self.parseRelational();
            var op_llvm: []const u8 = "eq";
            if (std.mem.eql(u8, op_val, "!=")) op_llvm = "ne";
            var obj = ObjectMap.init(self.allocator);
            try obj.put("type", std.json.Value{.string = "ComparisonExpression"});
            try obj.put("left", left);
            try obj.put("op", std.json.Value{.string = op_llvm});
            try obj.put("right", right);
            left = std.json.Value{.object = obj};
        }
        return left;
    }

    fn parseRelational(self: *Parser) anyerror!std.json.Value {
        var left = try self.parseBinaryAddSub();
        while (self.check(">") or self.check("<") or self.check(">=") or self.check("<=")) {
            const op_val = self.advance().value;
            const right = try self.parseBinaryAddSub();
            var op_llvm: []const u8 = "sgt";
            if (std.mem.eql(u8, op_val, "<")) op_llvm = "slt";
            if (std.mem.eql(u8, op_val, ">=")) op_llvm = "sge";
            if (std.mem.eql(u8, op_val, "<=")) op_llvm = "sle";
            var obj = ObjectMap.init(self.allocator);
            try obj.put("type", std.json.Value{.string = "ComparisonExpression"});
            try obj.put("left", left);
            try obj.put("op", std.json.Value{.string = op_llvm});
            try obj.put("right", right);
            left = std.json.Value{.object = obj};
        }
        return left;
    }

    fn parseBinaryAddSub(self: *Parser) anyerror!std.json.Value {
        var left = try self.parseMultiplicative();
        while (self.check("+") or self.check("-")) {
            const op_val = self.advance().value;
            const right = try self.parseMultiplicative();
            var op_str: []const u8 = "add";
            if (std.mem.eql(u8, op_val, "+")) op_str = "add";
            if (std.mem.eql(u8, op_val, "-")) op_str = "sub";
            var obj = ObjectMap.init(self.allocator);
            try obj.put("type", std.json.Value{ .string = "BinaryExpression" });
            try obj.put("left", left);
            try obj.put("op", std.json.Value{ .string = op_str });
            try obj.put("right", right);
            left = std.json.Value{ .object = obj };
        }
        return left;
    }

    fn parseMultiplicative(self: *Parser) anyerror!std.json.Value {
        var left = try self.parseUnary();
        while (self.check("*") or self.check("/")) {
            const op_val = self.advance().value;
            const right = try self.parseUnary();
            var op_str: []const u8 = "mul";
            if (std.mem.eql(u8, op_val, "*")) op_str = "mul";
            if (std.mem.eql(u8, op_val, "/")) op_str = "sdiv";
            var obj = ObjectMap.init(self.allocator);
            try obj.put("type", std.json.Value{ .string = "BinaryExpression" });
            try obj.put("left", left);
            try obj.put("op", std.json.Value{ .string = op_str });
            try obj.put("right", right);
            left = std.json.Value{ .object = obj };
        }
        return left;
    }

    fn parseUnary(self: *Parser) anyerror!std.json.Value {
        if (self.check("-") or self.check("!")) {
            const op = self.advance().value;
            const right = try self.parseUnary();
            var obj = ObjectMap.init(self.allocator);
            try obj.put("type", std.json.Value{ .string = "UnaryExpression" });
            try obj.put("op", std.json.Value{ .string = op });
            try obj.put("right", right);
            return std.json.Value{ .object = obj };
        }
        return self.parsePrimary();
    }

    fn parseAccessors(self: *Parser, base: std.json.Value) anyerror!std.json.Value {
        var current = base;
        while (self.check(".") or self.check("[") or self.check("?")) {
            if (self.check("?")) {
                _ = self.advance();
                var obj = ObjectMap.init(self.allocator);
                try obj.put("type", std.json.Value{ .string = "TryExpression" });
                try obj.put("expr", current);
                current = std.json.Value{ .object = obj };
            } else if (self.check(".")) {
                _ = self.advance(); 
                const f = self.advance().value;
                if (self.check("(")) {
                    _ = self.advance();
                    var c_args = JsonArray.init(self.allocator);
                    while (!self.check(")") and !self.checkType("EOF")) {
                        try c_args.append(try self.parseExpression());
                        if (self.check(",")) _ = self.advance();
                    }
                    _ = try self.consume(")");
                    var mc = ObjectMap.init(self.allocator);
                    try mc.put("type", std.json.Value{ .string = "MethodCall" });
                    try mc.put("base", current);
                    try mc.put("method", std.json.Value{ .string = f });
                    try mc.put("args", std.json.Value{ .array = c_args });
                    current = std.json.Value{ .object = mc };
                } else {
                    var acc = ObjectMap.init(self.allocator);
                    try acc.put("type", std.json.Value{ .string = "MemberAccess" });
                    try acc.put("base", current);
                    try acc.put("field", std.json.Value{ .string = f });
                    current = std.json.Value{ .object = acc };
                }
            } else {
                _ = self.advance(); const idx = try self.parseExpression(); _ = try self.consume("]");
                var idx_obj = ObjectMap.init(self.allocator);
                try idx_obj.put("type", std.json.Value{ .string = "IndexAccess" });
                try idx_obj.put("base", current);
                try idx_obj.put("index", idx);
                current = std.json.Value{ .object = idx_obj };
            }
        }
        return current;
    }

    fn parsePrimary(self: *Parser) anyerror!std.json.Value {
        if (self.check("(")) {
            _ = self.advance();
            const expr = try self.parseExpression();
            _ = try self.consume(")");
            return expr;
        }
        if (self.checkType("String")) {
             const tok = self.advance();
             var obj = ObjectMap.init(self.allocator);
             try obj.put("type", std.json.Value{ .string = "StringLiteral" });
             try obj.put("value", std.json.Value{ .string = tok.value });
             return std.json.Value{ .object = obj };
        }
        if (self.check("[")) {
             _ = self.advance();
             var elements = JsonArray.init(self.allocator);
             while (!self.check("]") and !self.checkType("EOF")) {
                 try elements.append(try self.parseExpression());
                 if (self.check(",")) _ = self.advance();
             }
             _ = try self.consume("]");
             var obj = ObjectMap.init(self.allocator);
             try obj.put("type", std.json.Value{ .string = "ArrayLiteral" });
             try obj.put("elements", std.json.Value{ .array = elements });
             return std.json.Value{ .object = obj };
        }
        if (self.checkType("Identifier")) {
            // Complex lookahead for Struct Literal vs match case block
            if (self.checkAt(1, "{") and self.checkAt(2, "Identifier") and self.checkAt(3, ":")) {
                 const s_name = self.advance().value; _ = self.advance(); // consume name and "{"
                 var fields = JsonArray.init(self.allocator);
                 while (!self.check("}") and !self.checkType("EOF")) {
                     const f_name = self.advance().value; _ = try self.consume(":");
                     const f_expr = try self.parseExpression();
                     var f_obj = ObjectMap.init(self.allocator);
                     try f_obj.put("name", std.json.Value{.string = f_name});
                     try f_obj.put("expr", f_expr);
                     try fields.append(std.json.Value{.object = f_obj});
                     if (self.check(",")) _ = self.advance();
                 }
                 _ = try self.consume("}");
                 var obj = ObjectMap.init(self.allocator);
                 try obj.put("type", std.json.Value{ .string = "StructLiteral" });
                 try obj.put("name", std.json.Value{ .string = s_name });
                 try obj.put("fields", std.json.Value{ .array = fields });
                 return std.json.Value{ .object = obj };
            }

            var full_name = std.ArrayList(u8).empty;
            try full_name.appendSlice(self.allocator, self.advance().value);
            while (self.check("::")) {
                _ = self.advance();
                try full_name.appendSlice(self.allocator, "::");
                try full_name.appendSlice(self.allocator, self.advance().value);
            }
            const name_str = try full_name.toOwnedSlice(self.allocator);

            if (self.check("(")) {
                 _ = self.advance();
                 var c_args = JsonArray.init(self.allocator);
                 while (!self.check(")") and !self.checkType("EOF")) {
                     try c_args.append(try self.parseExpression());
                     if (self.check(",")) _ = self.advance();
                 }
                 _ = try self.consume(")");
                 var obj = ObjectMap.init(self.allocator);
                 try obj.put("type", std.json.Value{ .string = "CallExpression" });
                 try obj.put("name", std.json.Value{ .string = name_str });
                 try obj.put("args", std.json.Value{ .array = c_args });
                 return std.json.Value{ .object = obj };
            } else {
                var node_obj = ObjectMap.init(self.allocator);
                try node_obj.put("type", std.json.Value{ .string = "Atomic" });
                try node_obj.put("value", std.json.Value{ .string = name_str });
                const current = std.json.Value{ .object = node_obj };
                return try self.parseAccessors(current);
            }
        }
        const base_tok = self.advance();
        var node_obj = ObjectMap.init(self.allocator);
        if (std.mem.eql(u8, base_tok.value, "true") or std.mem.eql(u8, base_tok.value, "false")) {
             try node_obj.put("type", std.json.Value{ .string = "BooleanLiteral" });
             try node_obj.put("value", std.json.Value{ .string = base_tok.value });
        } else {
             try node_obj.put("type", std.json.Value{ .string = "Atomic" });
             try node_obj.put("value", std.json.Value{ .string = base_tok.value });
        }
        return try self.parseAccessors(std.json.Value{ .object = node_obj });
    }

    fn checkAt(self: *Parser, offset: usize, val: []const u8) bool {
        const t = self.peekAt(offset);
        return std.mem.eql(u8, t.value, val) or std.mem.eql(u8, t.type, val);
    }

    fn parseVariableDecl(self: *Parser, is_constant: bool) !std.json.Value {
        if (is_constant) {
            _ = try self.consume("const");
        } else {
            _ = try self.consume("let");
        }
        const name = self.advance().value;
        if (self.check(":")) { _ = self.advance(); _ = self.advance(); }
        
        var node_obj = ObjectMap.init(self.allocator);
        try node_obj.put("type", std.json.Value{ .string = "VariableDeclaration" });
        try node_obj.put("name", std.json.Value{ .string = name });

        if (self.check("=")) {
            _ = self.advance();
            const expr = try self.parseExpression();
            const expr_type = expr.object.get("type").?.string;
            
            if (std.mem.eql(u8, expr_type, "Atomic")) {
                 try node_obj.put("value", expr.object.get("value").?);
            } else if (std.mem.eql(u8, expr_type, "StructLiteral")) {
                 try node_obj.put("structInit", expr);
            } else {
                 try node_obj.put("expr", expr);
            }
        } else {
            try node_obj.put("value", std.json.Value{ .string = "0" });
        }
        
        if (self.check(";")) _ = self.advance();
        return std.json.Value{ .object = node_obj };
    }

    fn parseIf(self: *Parser) !std.json.Value {
        _ = try self.consume("if");
        const has_paren = self.check("(");
        if (has_paren) _ = self.advance();
        const cond = try self.parseExpression();
        if (has_paren) _ = try self.consume(")");
        
        const then = try self.parseBlock();
        var els = JsonArray.init(self.allocator);
        if (self.check("else")) {
            _ = self.advance();
            if (self.check("if")) try els.append(try self.parseIf()) else {
                const b = try self.parseBlock();
                for (b.array.items) |s| try els.append(s);
            }
        }
        var obj = ObjectMap.init(self.allocator);
        try obj.put("type", std.json.Value{ .string = "IfStatement" });
        try obj.put("condition", cond);
        try obj.put("then_branch", then);
        try obj.put("else_branch", std.json.Value{ .array = els });
        return std.json.Value{ .object = obj };
    }

    fn parseWhile(self: *Parser) !std.json.Value {
        _ = try self.consume("while");
        const cond = try self.parseExpression();
        const body = try self.parseBlock();
        var obj = ObjectMap.init(self.allocator);
        try obj.put("type", std.json.Value{ .string = "WhileStatement" });
        try obj.put("condition", cond);
        try obj.put("body", body);
        return std.json.Value{ .object = obj };
    }

    fn parseFor(self: *Parser) !std.json.Value {
        _ = try self.consume("for");
        _ = try self.consume("(");
        const f_init = try self.parseStatement();
        const cond = try self.parseExpression();
        _ = try self.consume(";");
        const step = try self.parseExpression();
        _ = try self.consume(")");
        const body = try self.parseBlock();
        
        var while_body = JsonArray.init(self.allocator);
        for (body.array.items) |item| {
            try while_body.append(item);
        }
        try while_body.append(step);
        
        var while_obj = ObjectMap.init(self.allocator);
        try while_obj.put("type", std.json.Value{ .string = "WhileStatement" });
        try while_obj.put("condition", cond);
        try while_obj.put("body", std.json.Value{ .array = while_body });
        
        var block_body = JsonArray.init(self.allocator);
        try block_body.append(f_init);
        try block_body.append(std.json.Value{ .object = while_obj });
        
        var block_obj = ObjectMap.init(self.allocator);
        try block_obj.put("type", std.json.Value{ .string = "Block" });
        try block_obj.put("body", std.json.Value{ .array = block_body });
        
        return std.json.Value{ .object = block_obj };
    }

    fn parseMatch(self: *Parser) !std.json.Value {
        _ = try self.consume("match");
        const expr = try self.parseExpression();
        _ = try self.consume("{");
        
        var arms = JsonArray.init(self.allocator);
        var default_body = std.json.Value{ .null = {} };
        
        while (!self.check("}") and !self.checkType("EOF")) {
            if (self.check("case")) {
                _ = self.advance();
                const val = try self.parseExpression();
                _ = try self.consume(":");
                const body = try self.parseBlock();
                
                var arm_obj = ObjectMap.init(self.allocator);
                try arm_obj.put("value", val);
                try arm_obj.put("body", body);
                try arms.append(std.json.Value{ .object = arm_obj });
            } else if (self.check("default")) {
                _ = self.advance();
                _ = try self.consume(":");
                default_body = try self.parseBlock();
            } else {
                return error.UnexpectedToken;
            }
        }
        _ = try self.consume("}");
        
        var obj = ObjectMap.init(self.allocator);
        try obj.put("type", std.json.Value{ .string = "MatchStatement" });
        try obj.put("expr", expr);
        try obj.put("arms", std.json.Value{ .array = arms });
        try obj.put("default", default_body);
        return std.json.Value{ .object = obj };
    }

    fn parsePrint(self: *Parser) !std.json.Value {
        _ = try self.consume("print");
        _ = try self.consume("("); const expr = try self.parseExpression(); _ = try self.consume(")");
        var obj = ObjectMap.init(self.allocator);
        try obj.put("type", std.json.Value{ .string = "CallExpression" });
        try obj.put("name", std.json.Value{ .string = "io::print" });
        try obj.put("expr", expr);
        return std.json.Value{ .object = obj };
    }
};

fn printJson(file: std.fs.File, val: std.json.Value) anyerror!void {
    switch (val) {
        .null => try file.writeAll("null"),
        .bool => |b| if (b) try file.writeAll("true") else try file.writeAll("false"),
        .integer => |i| { var b: [32]u8 = undefined; try file.writeAll(try std.fmt.bufPrint(&b, "{}", .{i})); },
        .float => |f| { var b: [32]u8 = undefined; try file.writeAll(try std.fmt.bufPrint(&b, "{}", .{f})); },
        .string => |s| {
            try file.writeAll("\"");
            for (s) |c| {
                switch (c) {
                    '\"' => try file.writeAll("\\\""),
                    '\\' => try file.writeAll("\\\\"),
                    '\n' => try file.writeAll("\\n"),
                    '\r' => try file.writeAll("\\r"),
                    '\t' => try file.writeAll("\\t"),
                    else => try file.writeAll(&[_]u8{c}),
                }
            }
            try file.writeAll("\"");
        },
        .number_string => |ns| try file.writeAll(ns),
        .array => |arr| { try file.writeAll("["); for (arr.items, 0..) |item, i| { if (i > 0) try file.writeAll(","); try printJson(file, item); } try file.writeAll("]"); },
        .object => |obj| {
            try file.writeAll("{"); var it = obj.iterator(); var f = true;
            while (it.next()) |e| { if (!f) try file.writeAll(","); f = false; try file.writeAll("\""); try file.writeAll(e.key_ptr.*); try file.writeAll("\":"); try printJson(file, e.value_ptr.*); }
            try file.writeAll("}");
        }
    }
}

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();
    
    var args = std.process.args();
    // Skip binary name
    _ = args.skip();
    const input_file = args.next() orelse return;
    
    const file_content = try std.fs.cwd().readFileAlloc(allocator, input_file, 1024 * 1024);
    const tokens_json = try json.parseFromSlice(json.Value, allocator, file_content, .{});
    
    var tokens = std.ArrayList(Token).empty;
    const tokens_arr = if (tokens_json.value == .object)
        tokens_json.value.object.get("tokens").?.array
    else
        tokens_json.value.array;

    for (tokens_arr.items) |t_val| {
        try tokens.append(allocator, Token{
            .type = t_val.object.get("type").?.string,
            .value = t_val.object.get("value").?.string,
            .line = 0, .column = 0,
        });
    }
    
    var parser = Parser.init(allocator, tokens.items);
    const ast = try parser.parseProgram();
    try printJson(std.fs.File.stdout(), ast);
}
