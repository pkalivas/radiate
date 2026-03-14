#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VENV_ROOT = ROOT / ".venv"


@dataclass
class Args:
    generate_env: str | None
    deps: str | None


def parse_args() -> Args:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generate-env", metavar="PY_VERSION")
    parser.add_argument(
        "--deps",
        metavar="REQUIREMENTS_FILE",
        help="Install dependencies from a requirements file into the venv.",
    )

    ns = parser.parse_args()
    return Args(
        generate_env=ns.generate_env,
        deps=ns.deps,
    )


def run(cmd: list[str], **kwargs) -> subprocess.CompletedProcess[str]:
    print("+", " ".join(cmd))
    return subprocess.run(cmd, check=True, text=True, **kwargs)


def venv_bin(name: str) -> Path:
    if os.name == "nt":
        return VENV_ROOT / "Scripts" / f"{name}.exe"
    return VENV_ROOT / "bin" / name


def install_uv_into_venv() -> None:
    pip = venv_bin("pip")
    run([str(pip), "install", "--upgrade", "pip"])
    run([str(pip), "install", "--upgrade", "uv"])


def get_uv_python(version: str) -> str:
    try:
        result = run(["uv", "python", "find", version], capture_output=True)
    except FileNotFoundError:
        raise SystemExit("uv is not installed or not on PATH.")
    except subprocess.CalledProcessError:
        run(["uv", "python", "install", version])
        result = run(["uv", "python", "find", version], capture_output=True)

    python_path = result.stdout.strip()
    if not python_path:
        raise SystemExit(f"uv did not return a Python interpreter for '{version}'.")
    return python_path


def generate_env(version: str) -> None:
    if VENV_ROOT.exists():
        shutil.rmtree(VENV_ROOT)

    python_path = get_uv_python(version)

    run(["uv", "python", "pin", version])
    run(["uv", "venv", str(VENV_ROOT), "--python", python_path, "--seed"])

    install_uv_into_venv()


def install_deps(requirements_file: str) -> None:
    uv = venv_bin("uv")
    run([str(uv), "pip", "install", "-r", requirements_file])


def main() -> None:
    args = parse_args()

    did_something = False

    if args.generate_env:
        generate_env(args.generate_env)
        did_something = True

    if args.deps:
        install_deps(args.deps)
        did_something = True

    if not did_something:
        raise SystemExit("No action requested. Use --generate-env and/or --deps.")


if __name__ == "__main__":
    main()
