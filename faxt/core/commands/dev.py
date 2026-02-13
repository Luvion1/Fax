import subprocess, time
from pathlib import Path
from typing import List
from ..ui import UI
from ..toolchain import Toolchain

class DevCommands:
    @staticmethod
    def run(tc: Toolchain, args: List[str]):
        if not tc.root: UI.error("Not in a Fax project"); return
        target = next((a for a in args if a.endswith(".fax")), "src/main.fax")
        if not (tc.root / target).exists(): UI.error(f"{target} not found."); return
        UI.status("Compiling", f"{target}")
        bin_path = tc.run_pipeline(target, "--release" in args)
        if bin_path:
            UI.status("Running", f"`{bin_path.relative_to(tc.root)}`")
            print(f"{UI.GRAY}--- Output ---{UI.RESET}")
            try: subprocess.run([str(bin_path)])
            except Exception as e: UI.error(f"Execution failed: {e}")
            print(f"{UI.GRAY}--------------{UI.RESET}")

    @staticmethod
    def test(tc: Toolchain):
        if not tc.root: UI.error("Not in a Fax project"); return
        test_files = list(tc.root.glob("test_*.fax"))
        if (tc.root / "tests").exists(): test_files.extend(list((tc.root / "tests").glob("*.fax")))
        if not test_files: UI.status("Info", "No tests found.", UI.BLUE); return
        UI.status("Testing", f"Found {len(test_files)} tests")
        passed = 0
        for f in test_files:
            UI.status("Test", f"{f.name}", UI.CYAN)
            if tc.run_pipeline(str(f)):
                passed += 1
            else: print(f"      {UI.RED}failed{UI.RESET}")
        print(f"\n{UI.BOLD}Result:{UI.RESET} {passed}/{len(test_files)} passed")

    @staticmethod
    def repl(tc: Toolchain):
        print(f"{UI.PURPLE}Fax REPL{UI.RESET} (Type 'exit' to quit)")
        while True:
            try:
                line = input(f"{UI.BLUE}>> {UI.RESET}")
                if line.strip() in ["exit", "quit"]: break
                if not line.strip(): continue
                code = f'fn main() {{ print({line}); }}'
                if "print" in line or "let " in line: code = f'fn main() {{ {line} }}'
                tmp = Path(".repl_tmp.fax")
                with open(tmp, "w") as f: f.write(code)
                bin_p = tc.run_pipeline(str(tmp))
                if bin_p: subprocess.run([str(bin_p)])
                if tmp.exists(): tmp.unlink()
            except KeyboardInterrupt: break
            except Exception as e: print(f"{UI.RED}Error: {e}{UI.RESET}")

    @staticmethod
    def doc(tc: Toolchain):
        if not tc.root: UI.error("Not in a Fax project"); return
        UI.status("Generating", "Documentation")
        docs_dir = tc.root / "docs"
        docs_dir.mkdir(exist_ok=True)
        content = f"# Documentation for {tc.root.name}\n\n"
        for p in tc.root.glob("**/*.fax"):
            if "deps" in str(p): continue
            with open(p, "r") as f: lines = f.readlines()
            content += f"## {p.relative_to(tc.root)}\n\n"
            for line in lines:
                if line.strip().startswith("///"): content += line.strip()[3:].strip() + "\n"
        with open(docs_dir / "index.md", "w") as f: f.write(content)
        UI.status("Success", "Docs generated at docs/index.md")