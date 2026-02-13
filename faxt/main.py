#!/usr/bin/env python3
import sys
from pathlib import Path
from core.ui import UI
from core.toolchain import Toolchain
from core.deps import DepManager
from core.commands.project import ProjectCommands
from core.commands.dev import DevCommands
from core.commands.maintenance import MaintenanceCommands

VERSION = "0.8.0-modular"

def print_help():
    UI.header(VERSION)
    print(f"{UI.BOLD}Usage:{UI.RESET} faxt <command> [args]\n")
    print(f"{UI.BOLD}Project Management:{UI.RESET}")
    print(f"  {UI.CYAN}new{UI.RESET} <name>       Create a new Fax project")
    print(f"  {UI.CYAN}init{UI.RESET}             Initialize current directory")
    print(f"  {UI.CYAN}add{UI.RESET} <url>        Add a dependency")
    print(f"  {UI.CYAN}list{UI.RESET}              List dependencies")
    print(f"  {UI.CYAN}update{UI.RESET}            Update dependencies\n")
    print(f"{UI.BOLD}Development:{UI.RESET}")
    print(f"  {UI.GREEN}run{UI.RESET} [file]       Compile and execute")
    print(f"  {UI.GREEN}build{UI.RESET} [file]     Compile to target/")
    print(f"  {UI.GREEN}check{UI.RESET} [file]     Validate types")
    print(f"  {UI.GREEN}test{UI.RESET}              Run tests")
    print(f"  {UI.GREEN}repl{UI.RESET}              Interactive shell")
    print(f"  {UI.GREEN}doc{UI.RESET}               Generate docs\n")
    print(f"{UI.BOLD}Maintenance:{UI.RESET}")
    print(f"  {UI.YELLOW}stats{UI.RESET}             Project analysis")
    print(f"  {UI.YELLOW}clean{UI.RESET}             Remove artifacts")
    print(f"  {UI.YELLOW}doctor{UI.RESET}            Check toolchain")
    print(f"  {UI.YELLOW}bench{UI.RESET} [file]      Measure time")

def main():
    tc = Toolchain()
    if len(sys.argv) < 2:
        print_help()
        return

    cmd = sys.argv[1]
    args = sys.argv[2:]
    
    try:
        # Project Commands
        if cmd == "new": ProjectCommands.new(tc, args)
        elif cmd == "init": ProjectCommands.init(tc, args)
        elif cmd == "add": ProjectCommands.add(tc, args)
        elif cmd == "list": 
            if tc.root: DepManager(tc.root).list_deps()
            else: UI.error("Not in a Fax project")
        elif cmd == "update":
            if tc.root: DepManager(tc.root).update()
            else: UI.error("Not in a Fax project")
        
        # Development Commands
        elif cmd == "run": DevCommands.run(tc, args)
        elif cmd in ["build", "check"]:
            target = next((a for a in args if not a.startswith("-")), "src/main.fax")
            if tc.run_pipeline(target, release=(cmd == "build")):
                UI.status("Success", f"{cmd.capitalize()} finished")
        elif cmd == "test": DevCommands.test(tc)
        elif cmd == "repl": DevCommands.repl(tc)
        elif cmd == "doc": DevCommands.doc(tc)
        
        # Maintenance Commands
        elif cmd == "stats": MaintenanceCommands.stats(tc)
        elif cmd == "clean": MaintenanceCommands.clean(tc)
        elif cmd == "doctor": MaintenanceCommands.doctor()
        elif cmd == "bench": MaintenanceCommands.bench(tc, args)
        
        else:
            UI.error(f"Unknown command `{cmd}`")
            print_help()
    except KeyboardInterrupt:
        print("\nAborted.")
    except Exception as e:
        UI.error(f"Unexpected error: {e}")

if __name__ == "__main__":
    main()
