#!/usr/bin/env bash
set -euo pipefail

. "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/env.sh"

# Defaults
MODE="gil"                 # or "nogil"
PY_SPEC=                   # optional override (spec or abs path)
BUILD_KIND="develop"       # or "build"
VENV_DIR=".venv"           # venv directory  
SYNC_ARGS=(--group dev)    # default uv sync group
MATURIN_EXTRA=()           # passthrough to maturin

usage() {
  cat <<EOF
Usage: $0 [--gil|--nogil] [--python <spec|path>] [--build|--develop]
          [--venv-dir <dir>] [--sync-arg <arg>]... [--] [maturin args...]

Options:
  --gil / --nogil         Select mode (defaults from env GIL=${GIL:-1})
  --python <spec|path>    Interpreter to use (e.g. 3.12, 3.13t, /abs/path/python)
  --build                 Build wheel (maturin build)
  --develop               Develop install (maturin develop) [default]
  --venv-dir <dir>        Venv directory (default: .venv)
  --sync-arg <arg>        Extra 'uv sync' arg (repeatable)
  --                      End of script options; rest go to maturin

Examples:
  $0 --nogil --python 3.13t --build
  $0 --gil --venv-dir .venv-312 --sync-arg --group test
  $0 -- --locked
EOF
}

# ---- parse args ----
while [[ $# -gt 0 ]]; do
  case "$1" in
    --gil) MODE="gil"; shift;;
    --nogil) MODE="nogil"; shift;;
    --python|-p) PY_SPEC="${2:-}"; shift 2;;
    --build|-b) BUILD_KIND="build"; shift;;
    --develop|-d) BUILD_KIND="develop"; shift;;
    --venv-dir) VENV_DIR="${2:-.venv}"; shift 2;;
    --sync-arg) SYNC_ARGS+=("${2:-}"); shift 2;;
    -h|--help) usage; exit 0;;
    --) shift; MATURIN_EXTRA=("$@"); break;;
    *)  # passthrough unknowns to maturin
        MATURIN_EXTRA+=("$1"); shift;;
  esac
done

PY=
if [[ -n "${PY_SPEC:-}" ]]; then
  # user provided a spec or a path
  if [[ -x "$PY_SPEC" ]]; then
    # absolute/existing python path -> normalize
    PY="$(_realpath_py "$PY_SPEC")"
  else
    # spec like "3.12" or "3.13t" -> ensure via uv + resolve
    ensure_uv_python "$PY_SPEC" 
    PY="$(resolve_py_spec "$PY_SPEC")"
  fi
else
  # pick according to mode + defaults
  PY="$(choose_python "$MODE" "$PY_DIR")"
fi

if [[ -z "${PY}" ]]; then
  warn "Could not resolve Python interpreter." >&2
  exit 1
fi

pushd "$PY_DIR" >/dev/null
configure_uv_env "$MODE" "$PY" "$VENV_DIR" "${SYNC_ARGS[@]:-}"

if [[ "$MODE" == "nogil" ]]; then
  FEAT=(--no-default-features --features nogil)
else
  FEAT=(--features gil)
fi

if [[ "$BUILD_KIND" == "build" ]]; then
  CMD=(maturin build --release "${FEAT[@]}" "${MATURIN_EXTRA[@]}")
else
  CMD=(maturin develop --release "${FEAT[@]}" "${MATURIN_EXTRA[@]}")
fi

PYO3_PYTHON="$PY" uvx --python "$PY" "${CMD[@]}"

popd >/dev/null