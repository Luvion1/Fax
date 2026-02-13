import re
from pathlib import Path
from typing import Dict

class FaxConfig:
    def __init__(self, root: Path):
        self.root = root
        self.data = self._load()

    def _load(self) -> Dict:
        toml_path = self.root / "Fax.toml"
        config = {"package": {}, "dependencies": {}, "profile": {}}
        if not toml_path.exists(): return config
        
        section = ""
        try:
            with open(toml_path, "r") as f:
                for line in f:
                    line = line.split('#')[0].strip()
                    if not line: continue
                    section_match = re.match(r'^\[\s*(.+)\s*\]$', line)
                    if section_match:
                        section = section_match.group(1).strip()
                    elif "=" in line:
                        k, v = line.split("=", 1)
                        k, v = k.strip(), v.strip().strip('"')
                        if section == "package": config["package"][k] = v
                        elif section == "dependencies": config["dependencies"][k] = v
                        elif section.startswith("profile."):
                            p = section.split(".")[1]
                            if p not in config["profile"]: config["profile"][p] = {}
                            config["profile"][p][k] = v
        except Exception: pass
        return config
