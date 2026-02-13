class UI:
    PURPLE = '\x1b[38;5;135m'
    BLUE = '\x1b[38;5;75m'
    GREEN = '\x1b[38;5;84m'
    YELLOW = '\x1b[38;5;220m'
    RED = '\x1b[38;5;196m'
    CYAN = '\x1b[38;5;51m'
    GRAY = '\x1b[38;5;245m'
    BOLD = '\x1b[1m'
    RESET = '\x1b[0m'

    @staticmethod
    def status(action: str, message: str, color=GREEN):
        print(f"{color}{UI.BOLD}{action.rjust(12)}{UI.RESET} {message}")

    @staticmethod
    def error(message: str):
        print(f"{UI.RED}{UI.BOLD}error:{UI.RESET} {message}")

    @staticmethod
    def header(version: str):
        print(f"{UI.PURPLE}⚡ Fax Toolchain{UI.RESET} {UI.GRAY}v{version}{UI.RESET}\n")