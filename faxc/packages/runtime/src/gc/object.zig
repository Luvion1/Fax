const std = @import("std");

pub const ObjType = enum(u8) { Integer, Struct, Array, String };

pub const Obj = extern struct {
    pub const MAGIC: u64 = 0xDEADC0DECAFEBABE;
    pub const FORWARD_MAGIC: u64 = 0xF084A4D146000000;
    pub const COLOR_REMAPPED: u8 = 0x4;

    magic: u64,
    forward: ?*Obj,
    size: usize,
    count: usize,
    pmap_ptr: ?[*]const usize,
    pmap_len: usize,
    type: ObjType,
    color: u8,
    age: u8,
    flags: u8,
    _pad: [12]u8, // Ensure Obj is exactly 64 bytes for 16-byte alignment of payload

    pub const FLAG_PTR: u8 = 0x1;
    pub const FLAG_DIRTY: u8 = 0x2;

    pub fn data(self: *Obj) [*]u8 {
        // Obj is exactly 64 bytes, payload starts at offset 64
        return @ptrCast(@as([*]u8, @ptrCast(self)) + 64);
    }

    pub fn barrier(self: *Obj, current_color: u8) *Obj {
        const m = @atomicLoad(u64, &self.magic, .seq_cst);
        if (m == FORWARD_MAGIC) {
            return @atomicLoad(?*Obj, &self.forward, .seq_cst).?;
        }
        if (@atomicLoad(u8, &self.color, .seq_cst) != current_color) {
            return self.slow(current_color);
        }
        return self;
    }

    fn slow(self: *Obj, current_color: u8) *Obj {
        var curr = self;
        const m = @atomicLoad(u64, &curr.magic, .seq_cst);
        if (m == FORWARD_MAGIC) curr = @atomicLoad(?*Obj, &curr.forward, .seq_cst).?;
        if (current_color != COLOR_REMAPPED) {
            _ = @cmpxchgStrong(u8, &curr.color, curr.color, current_color, .seq_cst, .seq_cst);
        }
        return curr;
    }

    pub fn fullSize(self: *Obj) usize {
        return std.mem.alignForward(usize, 64 + self.size, 16);
    }
};
