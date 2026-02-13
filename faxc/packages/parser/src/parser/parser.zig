const std = @import("std");
const json = std.json;
const token = @import("token.zig");

pub const ParseErr = token.ParseErr;

pub const Parser = struct {
    allocator: std.mem.Allocator,
    tokens: []const token.Token,
    pos: usize,

    const Stmt = @import("stmt.zig");
    const Expr = @import("expr.zig");
    const Decl = @import("decl.zig");

    pub fn init(allocator: std.mem.Allocator, tokens: []const token.Token) Parser { 
        return .{ 
            .allocator = allocator, 
            .tokens = tokens, 
            .pos = 0 
        }; 
    }

    pub fn peek(self: *Parser) token.Token {
        return self.peekAt(0);
    }

    pub fn peekAt(self: *Parser, offset: usize) token.Token {
        if (self.pos + offset >= self.tokens.len) {
            return token.Token.init(.EOF, "", 0, 0);
        }
        return self.tokens[self.pos + offset];
    }

    pub fn advance(self: *Parser) token.Token {
        const t = self.peek();
        self.pos += 1;
        return t;
    }

    pub fn expect(self: *Parser, tokenType: token.TokenType) ParseErr!json.Value {
        const t = self.advance();
        if (t.type == tokenType) {
            return .{ .string = t.value };
        }
        return error.SyntaxError;
    }

    pub fn expectValue(self: *Parser, value: []const u8) ParseErr!json.Value {
        const t = self.advance();
        if (std.mem.eql(u8, t.value, value)) {
            return .{ .string = t.value };
        }
        return error.SyntaxError;
    }

    pub fn createNode(self: *Parser, typeName: []const u8, startToken: token.Token) !json.ObjectMap {
        var node = json.ObjectMap.init(self.allocator);
        try node.put("type", .{ .string = typeName });
        
        var loc = json.ObjectMap.init(self.allocator);
        try loc.put("line", .{ .integer = @intCast(startToken.line) });
        try loc.put("col", .{ .integer = @intCast(startToken.column) });
        try node.put("loc", .{ .object = loc });
        
        return node;
    }

    pub fn parse(self: *Parser) ParseErr!json.Value {
        var body = std.ArrayList(json.Value).init(self.allocator);
        while (self.peek().type != .EOF) {
            const t = self.peek();
            const node = if (std.mem.eql(u8, t.value, "const")) 
                try Decl.parseVariable(self, true) 
            else 
                try Stmt.parseStatement(self);
            try body.append(node);
        }
        
        var program = try self.createNode("Program", if (self.tokens.len > 0) self.tokens[0] else token.Token.init(.EOF, "", 0, 0));
        try program.put("body", .{ .array = body });
        return .{ .object = program };
    }
};