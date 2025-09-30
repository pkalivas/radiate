import os
import platform
from .utils import info, die, run, require_tool, realpath, which
from pathlib import Path

DEFAULT_PY_SPEC = os.getenv("RADIATE_PYTHON_VERSION", "python3")
DEFAULT_VENV = os.getenv("UV_PROJECT_ENVIRONMENT", ".venv")


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


def configure_uv_env(
    py_bin: str, venv_dir: Path, sync_args: list[str] | None = None
) -> None:
    require_tool("uv")
    venv_dir = venv_dir or Path(DEFAULT_VENV)
    sync_args = list(sync_args or []) or ["--group", "dev"]

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


def maturin_features_for(py_bin: str) -> list[str]:
    return (
        ["--no-default-features", "--features", "nogil"]
        if is_free_threaded(py_bin)
        else ["--features", "gil"]
    )
