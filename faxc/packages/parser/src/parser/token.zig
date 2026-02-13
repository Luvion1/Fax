const std = @import("std");
const json = std.json;

pub const ParseErr = error{ SyntaxError, UnexpectedEOF, OutOfMemory };

pub const TokenType = enum {
    Identifier,
    String,
    Number,
    Boolean,
    Null,
    Keyword,
    Operator,
    LogicalAnd,
    LogicalOr,
    Not,
    Match,
    Case,
    Default,
    Arrow,
    Symbol,
    ScopeResolution,
    ReturnType,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    Range,
    EOF,

    pub fn from(s: []const u8) TokenType {
        const map = .{ .{ "Identifier", .Identifier }, .{ "String", .String }, .{ "Number", .Number }, .{ "Boolean", .Boolean }, .{ "Null", .Null }, .{ "Keyword", .Keyword }, .{ "Operator", .Operator }, .{ "LogicalAnd", .LogicalAnd }, .{ "LogicalOr", .LogicalOr }, .{ "Not", .Not }, .{ "Match", .Match }, .{ "Case", .Case }, .{ "Default", .Default }, .{ "Arrow", .Arrow }, .{ "Symbol", .Symbol }, .{ "ScopeResolution", .ScopeResolution }, .{ "ReturnType", .ReturnType }, .{ "Range", .Range }, .{ "LeftParen", .LeftParen }, .{ "RightParen", .RightParen }, .{ "LeftBrace", .LeftBrace }, .{ "RightBrace", .RightBrace }, .{ "LeftBracket", .LeftBracket }, .{ "RightBracket", .RightBracket }, .{ "Comma", .Comma }, .{ "Semicolon", .Semicolon }, .{ "Colon", .Colon }, .{ "Dot", .Dot }, .{ "EOF", .EOF } };
        inline for (map) |kv| if (std.mem.eql(u8, s, kv[0])) return kv[1];
        return .Symbol;
    }
};

pub const Token = struct {
    type: TokenType,
    value: []const u8,
    line: usize,
    column: usize,

    pub fn init(t: TokenType, v: []const u8, ln: usize, col: usize) Token {
        return .{ .type = t, .value = v, .line = ln, .column = col };
    }
};
