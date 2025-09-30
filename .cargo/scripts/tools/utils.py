import sys
import subprocess
import shlex

from pathlib import Path


def info(msg: str) -> None:
    print(f"[INFO] {msg}")


def warn(msg: str) -> None:
    print(f"[WARN] {msg}")


def die(msg: str, code: int = 2) -> None:
    print(f"[ERROR] {msg}", file=sys.stderr)
    sys.exit(code)


def run(cmd, cwd=None, check=True, capture_output=False, env=None):
    if isinstance(cmd, str):
        shell = True
        printable = cmd
    else:
        shell = False
        printable = " ".join(shlex.quote(c) for c in cmd)
    info(f"$ {printable}")
    return subprocess.run(
        cmd,
        cwd=cwd,
        check=check,
        capture_output=capture_output,
        text=True,
        shell=shell,
        env=env,
    )


def which(cmd: str) -> str | None:
    from shutil import which as _which

    return _which(cmd)


def require_tool(tool: str) -> None:
    if which(tool) is None:
        die(f"required tool not found on PATH: {tool}")


def realpath(p: str) -> str:
    return str(Path(p).resolve())
