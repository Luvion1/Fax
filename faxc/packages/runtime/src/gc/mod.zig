pub const Fgc = @import("gc/fgc.zig").Fgc;
pub const Obj = @import("gc/object.zig").Obj;
pub const Page = @import("gc/page.zig").Page;

pub const initFgc = @import("api/exports.zig").initFgc;
pub const registerRoot = @import("api/exports.zig").registerRoot;
pub const allocate = @import("api/exports.zig").allocate;
pub const allocateString = @import("api/exports.zig").allocateString;
pub const allocateArray = @import("api/exports.zig").allocateArray;
pub const collectGarbage = @import("api/exports.zig").collectGarbage;
pub const pushFrame = @import("api/exports.zig").pushFrame;
pub const popFrame = @import("api/exports.zig").popFrame;
pub const concatStrings = @import("api/exports.zig").concatStrings;
