#!/usr/bin/env python3
import argparse
import os
import platform
import shlex
import subprocess
import sys
from pathlib import Path

from dataclasses import dataclass
from typing import List, Optional

REPO_ROOT = Path(os.getenv("CARGO_MAKE_WORKSPACE_WORKING_DIRECTORY")) or Path(__file__).parent.parent.parent

if not REPO_ROOT.exists():
    raise SystemExit(f"REPO_ROOT does not exist: {REPO_ROOT}")
elif not str(REPO_ROOT).split("/")[-1] == "radiate":
    raise SystemExit(f"REPO_ROOT does not look like the radiate repo: {REPO_ROOT}")

PY_DIR = REPO_ROOT / "py-radiate"
PY_TESTS_DIR = PY_DIR / "tests"

DEFAULT_PY_SPEC = os.getenv("RADIATE_PYTHON_VERSION", "python3")
DEFAULT_VENV = os.getenv("UV_PROJECT_ENVIRONMENT", ".venv")


@dataclass
class Norm:
    cmd: str  # "dev" | "wheel" | "test" | "ex"
    py: Optional[str] = None  # interpreter spec/path
    name: Optional[str] = None  # for ex
    rest: List[str] = None  # passthrough after "--"


def _split_passthrough(raw_argv: List[str]) -> List[str]:
    # everything after the first '--' is passthrough
    if "--" in raw_argv:
        i = raw_argv.index("--")
        return raw_argv[i + 1 :]
    return []


def normalize_args(args, raw_argv: List[str]) -> Norm:
    """
    args: parsed Namespace from build_parser()
    raw_argv: sys.argv[1:] (to capture passthrough even for shortcuts)
    """
    rest = _split_passthrough(raw_argv)
    # 1) real subcommands take precedence
    if args.cmd == "dev":
        return Norm(
            cmd="dev",
            py=getattr(args, "py", None),
            rest=getattr(args, "rest", []) or rest,
        )
    if args.cmd == "wheel":
        return Norm(
            cmd="wheel",
            py=getattr(args, "py", None),
            rest=getattr(args, "rest", []) or rest,
        )
    if args.cmd == "test":
        return Norm(
            cmd="test",
            rest=getattr(args, "rest", []) or rest,
        )
    if args.cmd == "ex":
        return Norm(
            cmd="ex",
            name=args.name,
            rest=getattr(args, "rest", []) or rest,
        )
    if args.cmd == "clean":
        return Norm(
            cmd="clean",
            rest=getattr(args, "rest", []) or rest,
        )

    # 2) shortcuts (only one can be present due to mutual-exclusion)
    if args.dev is not None:
        py = args.dev or None  # empty string means "use default"
        return Norm(cmd="dev", py=py, rest=rest)
    if args.wheel is not None:
        py = args.wheel or None
        return Norm(cmd="wheel", py=py, rest=rest)
    if args.test:
        return Norm(cmd="test", rest=rest)
    if args.ex:
        return Norm(cmd="ex", name=args.ex, rest=rest)
    if args.clean:
        return Norm(cmd="clean", rest=rest)

    raise SystemExit(2)


# ---------- small helpers ----------
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


def venv_python_path(venv_dir: Path) -> Path | None:
    if platform.system() == "Windows":
        for cand in ["Scripts/python.exe", "Scripts/python"]:
            p = venv_dir / cand
            if p.exists():
                return p
    else:
        for cand in ["bin/python", "bin/python3"]:
            p = venv_dir / cand
            if p.exists():
                return p
    return None


def is_free_threaded(py_bin: str) -> bool:
    try:
        out = run(
            [
                py_bin,
                "-c",
                "import sys; print('free-threaded' in getattr(sys, 'version','').lower())",
            ],
            capture_output=True,
        ).stdout.strip()
        return out.lower() == "true"
    except Exception:
        return False


def python_abi(py_bin: str) -> str:
    try:
        out = run(
            [
                py_bin,
                "-c",
                "import sys; print(f'{sys.version_info[0]}.{sys.version_info[1]}')",
            ],
            capture_output=True,
        ).stdout.strip()
        return out
    except Exception:
        return ""


# ---------- python spec resolution via uv or system ----------
def resolve_python(spec: str) -> str:
    """Return absolute path to python for the spec (supports paths, 'python3', '3.12', '3.13t', etc.)."""
    if not spec:
        spec = DEFAULT_PY_SPEC

    # direct path?
    p = Path(spec)
    if p.exists() and os.access(p, os.X_OK):
        return realpath(str(p))

    require_tool("uv")

    # try uv find (after best-effort install)
    run(["uv", "python", "install", spec], check=False)
    r = run(["uv", "python", "find", spec], check=False, capture_output=True)
    found = r.stdout.strip()
    if found:
        return realpath(found)

    # last fallback: system python name
    w = which(spec)
    if w:
        return realpath(w)

    die(f"Could not resolve Python interpreter for spec: {spec}")
    return ""  # unreachable


# ---------- uv venv creation/pinning/sync ----------
def configure_uv_env(
    py_bin: str, venv_dir: Path, sync_args: list[str] | None = None
) -> None:
    require_tool("uv")
    venv_dir = venv_dir or Path(DEFAULT_VENV)
    sync_args = list(sync_args or [])
    if not sync_args:
        sync_args = ["--group", "dev"]

    info(f"current_venv_dir={venv_dir}")

    want_real = realpath(py_bin)
    want_nogil = is_free_threaded(py_bin)
    vpy = venv_python_path(venv_dir)

    env = os.environ.copy()
    env["UV_PROJECT_ENVIRONMENT"] = str(venv_dir)

    if vpy and vpy.exists():
        have_real = realpath(str(vpy))
        have_nogil = is_free_threaded(str(vpy))
        want_abi = python_abi(py_bin)
        have_abi = python_abi(str(vpy))

        if want_real == have_real and want_nogil == have_nogil:
            info(f"[reuse] venv pinned to {have_real}, nogil={have_nogil}")
        else:
            if want_abi and want_abi == have_abi and want_nogil == have_nogil:
                info(f"[re-pin] same ABI={want_abi}, nogil={want_nogil}; pinning venv")
                run(["uv", "python", "pin", py_bin], env=env)
            else:
                info("[recreate] ABI/mode mismatch; recreating venv")
                try:
                    import shutil

                    shutil.rmtree(venv_dir)
                except Exception as e:
                    die(f"failed to remove venv {venv_dir}: {e}")
                run(["uv", "venv", "--python", py_bin], env=env)
                run(["uv", "python", "pin", py_bin], env=env)
    else:
        info(f"creating venv: {venv_dir}")
        run(["uv", "venv", "--python", py_bin], env=env)
        run(["uv", "python", "pin", py_bin], env=env)

    # sync deps if project files exist
    if (PY_DIR / "pyproject.toml").exists() or (PY_DIR / "requirements.txt").exists():
        info(f"syncing deps → {venv_dir} ({' '.join(sync_args)})")
        run(["uv", "sync", *sync_args], env=env, cwd=PY_DIR)

    # export for current process (for maturin)
    os.environ["PYO3_PYTHON"] = py_bin
    os.environ["RADIATE_FREE_THREADED"] = "true" if want_nogil else "false"


# ---------- actions ----------
def do_clean() -> None:
    info("cleaning build artifacts")
    try:
        import shutil

        venv = PY_DIR / ".venv"
        if venv.exists():
            shutil.rmtree(venv)

        # remove any .so files under radiate/
        for so_file in (PY_DIR / "radiate").glob("*.so"):
            try:
                os.remove(so_file)
            except Exception as e:
                warn(f"failed to remove {so_file}: {e}")

    except Exception as e:
        die(f"failed to clean: {e}")


def do_dev(py_spec: str, extra_args: list[str]) -> None:
    py_bin = resolve_python(py_spec)
    venv_dir = Path(DEFAULT_VENV)
    configure_uv_env(py_bin, venv_dir)

    # choose features by mode
    feats = ["--features", "gil"]
    if is_free_threaded(py_bin):
        feats = ["--no-default-features", "--features", "nogil"]

    require_tool("uvx")
    cmd = [
        "uvx",
        "--python",
        py_bin,
        "maturin",
        "develop",
        "--release",
        *feats,
        *extra_args,
    ]
    run(cmd, cwd=PY_DIR)


def do_wheel(py_spec: str, extra_args: list[str]) -> None:
    py_bin = resolve_python(py_spec)
    venv_dir = Path(DEFAULT_VENV)
    configure_uv_env(py_bin, venv_dir)

    feats = ["--features", "gil"]
    if is_free_threaded(py_bin):
        feats = ["--no-default-features", "--features", "nogil"]

    require_tool("uvx")
    cmd = [
        "uvx",
        "--python",
        py_bin,
        "maturin",
        "build",
        "--release",
        *feats,
        *extra_args,
    ]
    run(cmd, cwd=PY_DIR)


def do_test(extra_args: list[str]) -> None:
    require_tool("uv")
    cmd = ["uv", "run", "pytest", "--benchmark-disable", *extra_args]
    run(cmd, cwd=PY_DIR)


def resolve_example_py(arg: str) -> Path:
    p = Path(arg)
    if p.is_absolute():
        return p
    # allow bare names like "image.py" or "image"
    if "/" in arg or "\\" in arg:
        return (
            (PY_DIR / "examples" / arg).with_suffix(".py")
            if not arg.endswith(".py")
            else (PY_DIR / "examples" / arg)
        )
    return PY_DIR / "examples" / (arg if arg.endswith(".py") else f"{arg}.py")


def do_example(example: str, extra_args: list[str]) -> None:
    target = resolve_example_py(example)
    if not target.exists():
        die(f"Python example not found: {target}")
    require_tool("uv")
    run(["uv", "sync", "--group", "dev"], cwd=PY_DIR)
    rel = target.relative_to(PY_DIR)
    run(["uv", "run", str(rel), *extra_args], cwd=PY_DIR)


def cmd_dev(py, rest):
    os.chdir(PY_DIR)
    print(f"[dev] python={py}, rest={rest}")
    do_clean()
    do_dev(py or DEFAULT_PY_SPEC, rest)


def cmd_wheel(py, rest):
    os.chdir(PY_DIR)
    print(f"[wheel] python={py}, rest={rest}")
    do_clean()
    do_wheel(py or DEFAULT_PY_SPEC, rest)


def cmd_test(rest):
    os.chdir(PY_TESTS_DIR)
    print(f"[test] rest={rest}")
    do_test(rest)


def cmd_ex(name, rest):
    print(f"[ex] name={name}, rest={rest}")
    do_example(name, rest)

def cmd_clean(rest):
    os.chdir(PY_DIR)
    print(f"[clean] rest={rest}")
    do_clean()


# ---------- CLI ----------
def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="cargo py",
        description="Radiate Python helper (uv + maturin + pytest + examples)",
    )

    # shortcuts (mutually exclusive)
    g = p.add_mutually_exclusive_group()
    g.add_argument(
        "--dev",
        nargs="?",
        const="",
        metavar="PY",
        help="shortcut for dev (optional PY, e.g. 3.13t)",
    )
    g.add_argument(
        "--wheel",
        nargs="?",
        const="",
        metavar="PY",
        help="shortcut for wheel (optional PY)",
    )
    g.add_argument("--test", action="store_true", help="shortcut for test")
    g.add_argument("--ex", metavar="NAME", help="shortcut for example name/path")
    g.add_argument(
        "--clean", action="store_true", help="clean build artifacts before action"
    )

    # subcommands (exact same behavior)
    sub = p.add_subparsers(dest="cmd")

    s_dev = sub.add_parser("dev", help="maturin develop (editable)")
    s_dev.add_argument("--py", help="Python spec (e.g. 3.12, 3.13t)")
    s_dev.add_argument("rest", nargs=argparse.REMAINDER)

    s_wheel = sub.add_parser("wheel", help="maturin build (wheel)")
    s_wheel.add_argument("--py", help="Python spec")
    s_wheel.add_argument("rest", nargs=argparse.REMAINDER)

    s_test = sub.add_parser("test", help="pytest via uv")
    s_test.add_argument("rest", nargs=argparse.REMAINDER)

    s_ex = sub.add_parser("ex", help="run example from py-radiate/examples")
    s_ex.add_argument("name", help="example name or path")
    s_ex.add_argument("rest", nargs=argparse.REMAINDER)

    s_clean = sub.add_parser("clean", help="clean build artifacts")
    s_clean.add_argument("rest", nargs=argparse.REMAINDER)

    return p


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    parser = build_parser()
    args = parser.parse_args(argv)
    norm = normalize_args(args, argv)

    if norm.cmd == "dev":
        return cmd_dev(norm.py, norm.rest or [])
    if norm.cmd == "wheel":
        return cmd_wheel(norm.py, norm.rest or [])
    if norm.cmd == "test":
        return cmd_test(norm.rest or [])
    if norm.cmd == "ex":
        return cmd_ex(norm.name, norm.rest or [])
    if args.clean:
        return cmd_clean(norm.rest or [])

    parser.print_help()
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
