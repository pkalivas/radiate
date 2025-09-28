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
    line="$(echo "$lines" | grep -i '+freethreaded' | awk '{print $1}' | sort -rV | head -n1 || true)"
    [[ -n "$line" ]] || { echo "No free-threaded Python found. Try: uv python install 3.13t"; exit 1; }
    resolve_with_uv "$line"
  else
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

if [[ $FREETHREADED -eq 1 ]]; then
  PY="$(choose_interpreter nogil)"
  
  if [ ! -d '.venv' ]; then
    version=$("$PY" -c 'import sys; print(".".join(map(str, sys.version_info[:3])))')
    uv venv --python $version+freethreaded
    uv python pin $PY 
    uv sync
  fi

  echo "Using free-threaded interpreter: $PY"
  if [[ $DO_BUILD -eq 1 ]]; then
    PYO3_PYTHON="$PY" 
    uvx --python "$PY" maturin build  --release --no-default-features --features nogil 
  else
    PYO3_PYTHON="$PY" 
    uvx --python "$PY" maturin develop  --release --no-default-features --features nogil
  fi
else
  PY="$(choose_interpreter gil)"

  if [ ! -d '.venv' ]; then
    version=$("$PY" -c 'import sys; print(".".join(map(str, sys.version_info[:3])))')
    uv venv --python $version
    uv python pin $PY
    uv sync
  fi

  if [[ $DO_BUILD -eq 1 ]]; then
    PYO3_PYTHON="$PY" uvx --python "$PY" maturin build --release --features gil
  else
    PYO3_PYTHON="$PY" uvx --python "$PY" maturin develop --release --features gil
  fi
fi

echo "Done for: $PY"