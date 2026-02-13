import os
from pathlib import Path
from typing import List
from ..ui import UI
from ..toolchain import Toolchain
from ..deps import DepManager

class ProjectCommands:
    @staticmethod
    def new(tc: Toolchain, args: List[str]):
        if not args: UI.error("Usage: faxt new <name>"); return
        name = args[0]
        path = Path.cwd() / name
        if path.exists(): UI.error(f"Directory `{name}` already exists."); return
        path.mkdir(parents=True)
        original_dir = Path.cwd()
        os.chdir(path)
        ProjectCommands.init(tc, [name])
        os.chdir(original_dir)
        UI.status("Ready", f"Project `{name}` created at ./{name}")

    @staticmethod
    def init(tc: Toolchain, args: List[str]):
        name = args[0] if args else Path.cwd().name
        root = Path.cwd()
        if (root / "Fax.toml").exists(): UI.error("Project already exists."); return
        UI.status("Initializing", f"Fax project `{name}`")
        try:
            with open(root / "Fax.toml", "w") as f:
                f.write(f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2026"\n\n[dependencies]\n')
            (root / "src").mkdir(exist_ok=True)
            main_f = root / "src" / "main.fax"
            if not main_f.exists():
                with open(main_f, "w") as f: f.write('fn main() {\n    print("Hello, Fax!");\n}\n')
            UI.status("Created", "Project structure ready.")
        except Exception as e: UI.error(f"Init failed: {e}")

    @staticmethod
    def add(tc: Toolchain, args: List[str]):
        if not tc.root: UI.error("Not in a Fax project"); return
        if not args: UI.error("Usage: faxt add <url>"); return
        DepManager(tc.root).add(args[0])