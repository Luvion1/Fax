const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    
    // Secara eksplisit menentukan konfigurasi untuk menghindari fitur-fitur yang menyebabkan peringatan
    // Kita akan tetap menggunakan konfigurasi standar tapi menonaktifkan fitur-fitur yang menyebabkan peringatan
    
    const optimize = b.standardOptimizeOption(.{});

    const fgc_lib = b.addLibrary(.{
        .name = "faxruntime",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = optimize,
            .link_libc = true,
        }),
    });
    b.installArtifact(fgc_lib);
}
