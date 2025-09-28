#!/usr/bin/env bash

# Usage:
#   ./build.sh                # develop-install against latest regular CPython
#   ./build.sh --freethreaded # develop-install against latest free-threaded CPython
#   ./build.sh --build        # build wheel instead of develop
#   ./build.sh --python 3.13  # pick a specific version/spec or absolute path

FREETHREADED=0
DO_BUILD=0
PY_SPEC=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --freethreaded) FREETHREADED=1; shift;;
    --build) DO_BUILD=1; shift;;
    --python) PY_SPEC="${2:-}"; shift 2;;
    *) echo "Unknown arg: $1" >&2; exit 1;;
  esac
done

need_uv() {
  command -v uv >/dev/null || { echo "uv not found (brew install uv)"; exit 1; }
  command -v uvx >/dev/null || { echo "uvx not found (part of uv)"; exit 1; }
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYPROJECT="$ROOT/../py-radiate/pyproject.toml"

# Resolve absolute interpreter path from a spec or absolute path
resolve_with_uv() {
  local spec="$1"
  if [[ -x "$spec" ]]; then
    python3 - <<'PY' "$spec"
import os, sys; print(os.path.realpath(sys.argv[1]))
PY
  else
    uv python find "$spec"
  fi
}

pick_latest_by_mode() {
  local want="$1"  # "gil" or "nogil"
  local line id path ver
  local lines
  lines="$(uv python list --only-installed)"
  if [[ "$want" == "nogil" ]]; then
    #	nogil branch: keeps only interpreter IDs that contain +freethreaded (case-insensitive).
    # That’s how uv tags free-threaded builds (e.g. cpython-3.13.7+freethreaded-macos-aarch64-none).
    line="$(echo "$lines" | grep -i '+freethreaded' | awk '{print $1}' | sort -rV | head -n1 || true)"
    [[ -n "$line" ]] || { echo "No free-threaded Python found. Try: uv python install 3.13t"; exit 1; }
    resolve_with_uv "$line"
  else
    #	gil branch: keeps everything that does not have +freethreaded.
    # That’s how you get “normal” CPython builds (e.g. cpython-3.12.7-macos-aarch64-none).
    line="$(echo "$lines" | grep -vi '+freethreaded' | awk '{print $1}' | sort -rV | head -n1 || true)"
    [[ -n "$line" ]] || { echo "No regular CPython found. Try: uv python install 3.12"; exit 1; }
    resolve_with_uv "$line"
  fi
}

choose_interpreter() {
  local mode="$1"
  if [[ -n "$PY_SPEC" ]]; then
    resolve_with_uv "$PY_SPEC"
  else
    pick_latest_by_mode "$mode"
  fi
}

need_uv
cd "$ROOT/../py-radiate"

MODE="gil"
[[ ${FREETHREADED:-0} -eq 1 ]] && MODE="nogil"

PY="$(choose_interpreter "$MODE")"

ensure_venv() {
  local mode="$1" py="$2"
  if [[ ! -d ".venv" ]]; then
    local version spec
    version="$("$py" -c 'import sys; print(".".join(map(str, sys.version_info[:3])))')"
    if [[ "$mode" == "nogil" ]]; then
      # works with uv (equivalent to 3.13t)
      spec="${version}+freethreaded"
    else
      spec="${version}"
    fi
    echo "Creating venv for $spec"
    uv venv --python "$spec"
    uv python pin "$py"
    uv sync
  fi
}

run_maturin() {
  local mode="$1" py="$2" cmd flags
  if [[ "$mode" == "nogil" ]]; then
    flags=(--no-default-features --features nogil)
  else
    flags=(--features gil)
  fi
  if [[ ${DO_BUILD:-0} -eq 1 ]]; then
    cmd=(maturin build --release "${flags[@]}")
  else
    cmd=(maturin develop --release "${flags[@]}")
  fi
  PYO3_PYTHON="$py" uvx --python "$py" "${cmd[@]}"
}

ensure_venv "$MODE" "$PY"
run_maturin "$MODE" "$PY"

echo "Successful wheel build: $PY ($MODE)"
