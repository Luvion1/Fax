import os, shutil, time
from pathlib import Path
from typing import List
from ..ui import UI
from ..toolchain import Toolchain

class MaintenanceCommands:
    @staticmethod
    def stats(tc: Toolchain):
        if not tc.root: UI.error("Not in a Fax project"); return
        UI.status("Analyzing", "Project stats")
        fax_stats = {"Code": [0,0], "Tests": [0,0]}
        for root, dirs, files in os.walk(tc.root):
            if "target" in dirs: dirs.remove("target")
            for f in files:
                if f.endswith(".fax"):
                    with open(Path(root)/f) as file: loc = sum(1 for _ in file)
                    k = "Tests" if "tests" in root or f.startswith("test_") else "Code"
                    fax_stats[k][0] += loc
                    fax_stats[k][1] += 1
        for k, v in fax_stats.items():
            print(f"  {k:<10} {v[1]:<6} files, {v[0]:<8} lines")

    @staticmethod
    def clean(tc: Toolchain):
        if tc.root:
            shutil.rmtree(tc.root / "target", ignore_errors=True)
            for f in tc.root.glob(".temp_*"): f.unlink()
            UI.status("Cleaned", "target/ and temp files removed")
        else: UI.error("Not in a Fax project")

    @staticmethod
    def doctor():
        UI.status("Checking", "Environment health")
        tools = [("rustc","Rust"), ("zig","Zig"), ("stack","Haskell"), ("node","Node.js")]
        for cmd, lbl in tools:
            ok = shutil.which(cmd) is not None
            print(f"  {UI.GREEN if ok else UI.RED}{'OK' if ok else 'MISSING'}{UI.RESET} {lbl}")

    @staticmethod
    def bench(tc: Toolchain, args: List[str]):
        target = args[0] if args else "src/main.fax"
        if not Path(target).exists(): UI.error("File not found"); return
        UI.status("Benchmarking", target)
        s = time.perf_counter()
        if tc.run_pipeline(target, release=True):
            print(f"  Total time: {(time.perf_counter()-s)*1000:.2f}ms")