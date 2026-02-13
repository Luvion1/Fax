pub const Fgc = @import("gc/fgc.zig").Fgc;
pub const Obj = @import("gc/object.zig").Obj;
pub const Page = @import("gc/page.zig").Page;
pub const GcError = @import("gc/fgc.zig").GcError;
pub const Frame = @import("gc/fgc.zig").Frame;

// Re-export GC functions
pub const initGlobalGc = @import("gc/fgc.zig").initGlobalGc;
pub const getGlobalGc = @import("gc/fgc.zig").getGlobalGc;
pub const deinitGlobalGc = @import("gc/fgc.zig").deinitGlobalGc;
