const std = @import("std");

pub const ObjType = enum(u8) { Integer, Struct, Array, String };

pub const Obj = extern struct {
    pub const MAGIC: u64 = 0xDEADC0DECAFEBABE;
    pub const FORWARD_MAGIC: u64 = 0xF084A4D146000000;
    pub const COLOR_REMAPPED: u8 = 0x4;

    magic: u64,
    forward: ?*Obj,
    size: usize,
    pmap_ptr: ?[*]const usize,
    pmap_len: usize,
    type: ObjType,
    color: u8,
    flags: u8,
    _pad: [5]u8, // Explicit padding to reach 48 bytes and maintain 16-byte alignment

    pub const FLAG_PTR: u8 = 0x1;

    pub fn data(self: *Obj) [*]u8 {
        return @ptrCast(@as([*]u8, @ptrCast(self)) + 48);
    }

    pub fn barrier(self: *Obj, current_color: u8) *Obj {
        if (self.color != current_color) return self.slow(current_color);
        return self;
    }

    fn slow(self: *Obj, current_color: u8) *Obj {
        var curr = self;
        if (curr.magic == FORWARD_MAGIC) curr = curr.forward.?;
        if (current_color != COLOR_REMAPPED) curr.color = current_color;
        return curr;
    }

    pub fn fullSize(self: *Obj) usize {
        return std.mem.alignForward(usize, 48 + self.size, 16);
    }
};
