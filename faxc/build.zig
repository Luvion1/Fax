const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const parser_exe = b.addExecutable(.{
        .name = "parser",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/components/parser/parser.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });
    b.installArtifact(parser_exe);

    const gc_lib = b.addLibrary(.{
        .name = "faxruntime",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/runtime/fgc.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });
    b.installArtifact(gc_lib);
}
