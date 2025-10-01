# action.py
import os
from pathlib import Path
from .utils import info, die, run, require_tool, warn
from .env import (
    DEFAULT_PY_SPEC,
    DEFAULT_VENV,
    resolve_python,
    is_free_threaded,
    configure_uv_env,
)

REPO_ROOT = (
    Path(os.getenv("CARGO_MAKE_WORKSPACE_WORKING_DIRECTORY"))
    or Path(__file__).parent.parent.parent
)

if not REPO_ROOT.exists():
    raise SystemExit(f"REPO_ROOT does not exist: {REPO_ROOT}")
elif not str(REPO_ROOT).split("/")[-1] == "radiate":
    raise SystemExit(f"REPO_ROOT does not look like the radiate repo: {REPO_ROOT}")

PY_DIR = REPO_ROOT / "py-radiate"
PY_TESTS_DIR = PY_DIR / "tests"
PY_EXAMPLES_DIR = PY_DIR / "examples"


def do_clean() -> None:
    info("cleaning build artifacts")
    try:
        import shutil

        venv = PY_DIR / ".venv"
        examples_venv = PY_EXAMPLES_DIR / ".venv"
        if venv.exists():
            shutil.rmtree(venv)
        if examples_venv.exists():
            shutil.rmtree(examples_venv)

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
    # feats = ["--features", "gil"]
    # if is_free_threaded(py_bin):
    #     feats = ["--no-default-features", "--features", "nogil"]

    require_tool("uvx")
    cmd = [
        "uvx",
        "--python",
        py_bin,
        "maturin",
        "develop",
        "--release",
        # *feats,
        # *extra_args,
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
