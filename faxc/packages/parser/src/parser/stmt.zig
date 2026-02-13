const std = @import("std");
const json = std.json;
const Parser = @import("parser.zig").Parser;
const ParseErr = @import("parser.zig").ParseErr;
const Expr = @import("expr.zig");
const Decl = @import("decl.zig");

pub fn parseStatement(self: *Parser) ParseErr!json.Value {
    const t = self.peek().value;
    const startToken = self.peek();
    
    if (std.mem.eql(u8, t, "module")) return try Decl.parseModule(self);
    if (std.mem.eql(u8, t, "use")) return try Decl.parseUse(self);
    if (std.mem.eql(u8, t, "pub")) { 
        _ = self.advance(); 
        var node = try parseStatement(self); 
        if (node == .object) try node.object.put("is_pub", .{ .bool = true }); 
        return node; 
    }
    if (std.mem.eql(u8, t, "extern")) return try Decl.parseFunction(self);
    if (std.mem.eql(u8, t, "fn")) return try Decl.parseFunction(self);
    if (std.mem.eql(u8, t, "struct")) return try Decl.parseStruct(self);
    if (std.mem.eql(u8, t, "if")) return try parseIf(self);
    if (std.mem.eql(u8, t, "while")) return try parseWhile(self);
    if (std.mem.eql(u8, t, "for")) return try parseFor(self);
    if (std.mem.eql(u8, t, "let")) return try Decl.parseVariable(self, false);
    if (std.mem.eql(u8, t, "{")) return try parseBlock(self);
    if (std.mem.eql(u8, t, "return")) return try parseReturn(self);
    
    if (std.mem.eql(u8, t, "break")) { 
        _ = self.advance(); 
        if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance(); 
        const node = try self.createNode("BreakStatement", startToken); 
        return .{ .object = node }; 
    }
    
    if (std.mem.eql(u8, t, "continue")) { 
        _ = self.advance(); 
        if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance(); 
        const node = try self.createNode("ContinueStatement", startToken); 
        return .{ .object = node }; 
    }

    const expression = try Expr.parseExpression(self);
    if (std.mem.eql(u8, self.peek().value, "=")) {
        const assignToken = self.peek();
        _ = self.advance();
        const value = try Expr.parseExpression(self);
        if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance();
        
        var node = try self.createNode("Assignment", assignToken);
        try node.put("target", expression);
        try node.put("expr", value);
        return .{ .object = node };
    }
    
    if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance();
    return expression;
}

pub fn parseBlock(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    _ = try self.expectValue("{");
    
    var body = std.ArrayList(json.Value).init(self.allocator);
    while (!std.mem.eql(u8, self.peek().value, "}")) {
        try body.append(try parseStatement(self));
    }
    
    _ = try self.expectValue("}");
    var node = try self.createNode("Block", startToken);
    try node.put("body", .{ .array = body });
    return .{ .object = node };
}

pub fn parseIf(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    _ = try self.expectValue("if"); 
    
    const condition = try Expr.parseExpression(self);
    _ = try self.expectValue("{");
    
    var thenBranch = std.ArrayList(json.Value).init(self.allocator);
    while (!std.mem.eql(u8, self.peek().value, "}")) {
        try thenBranch.append(try parseStatement(self));
    }
    _ = try self.expectValue("}");
    
    var elseBranch = std.ArrayList(json.Value).init(self.allocator);
    if (std.mem.eql(u8, self.peek().value, "else")) {
        _ = self.advance();
        if (std.mem.eql(u8, self.peek().value, "if")) {
            try elseBranch.append(try parseIf(self));
        } else {
            _ = try self.expectValue("{");
            while (!std.mem.eql(u8, self.peek().value, "}")) {
                try elseBranch.append(try parseStatement(self));
            }
            _ = try self.expectValue("}");
        }
    }
    
    var node = try self.createNode("IfStatement", startToken);
    try node.put("condition", condition);
    try node.put("then_branch", .{ .array = thenBranch });
    try node.put("else_branch", .{ .array = elseBranch });
    return .{ .object = node };
}

pub fn parseWhile(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    _ = try self.expectValue("while"); 
    
    const condition = try Expr.parseExpression(self);
    _ = try self.expectValue("{");
    
    var body = std.ArrayList(json.Value).init(self.allocator);
    while (!std.mem.eql(u8, self.peek().value, "}")) {
        try body.append(try parseStatement(self));
    }
    _ = try self.expectValue("}");
    
    var node = try self.createNode("WhileStatement", startToken);
    try node.put("condition", condition);
    try node.put("body", .{ .array = body });
    return .{ .object = node };
}

pub fn parseFor(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    _ = try self.expectValue("for"); 
    
    const name = (try self.expect(.Identifier)).string;
    _ = try self.expectValue("in");
    
    const start = try Expr.parseExpression(self); 
    _ = try self.expectValue("..");
    const end = try Expr.parseExpression(self); 
    _ = try self.expectValue("{");
    
    var body = std.ArrayList(json.Value).init(self.allocator);
    while (!std.mem.eql(u8, self.peek().value, "}")) {
        try body.append(try parseStatement(self));
    }
    _ = try self.expectValue("}");
    
    var node = try self.createNode("ForStatement", startToken);
    try node.put("var_name", .{ .string = name });
    try node.put("start", start); 
    try node.put("end", end);
    try node.put("body", .{ .array = body });
    return .{ .object = node };
}

pub fn parseReturn(self: *Parser) ParseErr!json.Value {
    const startToken = self.peek();
    _ = self.advance();
    
    var node = try self.createNode("ReturnStatement", startToken);
    if (std.mem.eql(u8, self.peek().value, ";")) {
        _ = self.advance();
        try node.put("argument", .null);
    } else {
        const argument = try Expr.parseExpression(self);
        if (std.mem.eql(u8, self.peek().value, ";")) _ = self.advance();
        try node.put("argument", argument);
    }
    return .{ .object = node };
}