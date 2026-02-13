import subprocess, re
from pathlib import Path
from .ui import UI
from .config import FaxConfig

class DepManager:
    def __init__(self, project_root: Path):
        self.root = project_root
        self.deps_dir = self.root / ".fax" / "deps"

    def list_deps(self):
        config = FaxConfig(self.root).data
        deps = config.get("dependencies", {})
        if not deps:
            UI.status("Info", "No dependencies found in Fax.toml", UI.BLUE)
            return
        
        print(f"{UI.BOLD}Direct Dependencies:{UI.RESET}")
        for name, url in deps.items():
            status = f"{UI.GREEN}✓{UI.RESET}" if (self.deps_dir / name).exists() else f"{UI.RED}✗{UI.RESET}"
            print(f"  {status} {UI.CYAN}{name:<12}{UI.RESET} {UI.GRAY}{url}{UI.RESET}")

    def add(self, github_url: str):
        if not github_url.startswith("github.com/"):
            UI.error("Library must be a GitHub link")
            return

        pkg_name = github_url.split("/")[-1]
        config = FaxConfig(self.root).data
        if pkg_name in config.get("dependencies", {}):
            UI.status("Skipping", f"Dependency {pkg_name} already exists", UI.YELLOW)
            return

        UI.status("Adding", f"{github_url} as {pkg_name}")
        toml_path = self.root / "Fax.toml"
        try:
            with open(toml_path, "r") as f: content = f.read()
            dep_pattern = r'^(\s*\[\s*dependencies\s*\].*)$'
            if re.search(dep_pattern, content, re.MULTILINE):
                replacement = r'\1\n' + f'{pkg_name} = "{github_url}"'
                content = re.sub(dep_pattern, replacement, content, count=1, flags=re.MULTILINE)
            else:
                if not content.endswith('\n'): content += '\n'
                content += f'\n[dependencies]\n{pkg_name} = "{github_url}"\n'
            with open(toml_path, "w") as f: f.write(content)
            self.update()
        except Exception as e: UI.error(f"Failed to update Fax.toml: {e}")

    def update(self):
        self.deps_dir.mkdir(parents=True, exist_ok=True)
        config = FaxConfig(self.root).data
        deps = config.get("dependencies", {})
        if not deps: return
        for name, url in deps.items():
            dest = self.deps_dir / name
            full_url = f"https://{url}.git"
            try:
                if dest.exists():
                    UI.status("Updating", f"{name} ({url})")
                    subprocess.run(["git", "pull"], cwd=dest, capture_output=True, check=True)
                else:
                    UI.status("Fetching", f"{name} from {url}")
                    subprocess.run(["git", "clone", "--depth", "1", full_url, str(dest)], capture_output=True, check=True)
            except Exception as e: UI.error(f"Git failed for {name}: {e}")