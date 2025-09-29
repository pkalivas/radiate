#!/usr/bin/env bash
set -euo pipefail

export DEFAULT_PYTHON_VERSION=3.12
export DEFAULT_PYTHON_VERSION_NOGIL=3.13t
export GIL=1  # 1 = gil, 0 = nogil

export SCRIPT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export REPO_ROOT="$(cd "$SCRIPT_ROOT/.." && pwd)"
export PY_DIR="$REPO_ROOT/py-radiate"

info(){ printf '\033[1;36m[INFO]\033[0m %s\n' "$*"; }
export -f info
warn(){ printf '\033[1;33m[WARN]\033[0m %s\n' "$*"; }
export -f warn
die(){ printf '\033[1;31m[ERROR]\033[0m %s\n' "$*" >&2; exit 1; }
export -f die

_trim() {  
  local s; s="${1-}"; s="${s#"${s%%[![:space:]]*}"}"; s="${s%"${s##*[![:space:]]}"}"; printf '%s' "$s"
}

has_uv() { command -v uv >/dev/null 2>&1; }
export -f has_uv

_realpath_py() {
  python3 - "$1" <<'PY'
import os, sys; print(os.path.realpath(sys.argv[1]))
PY
}
export -f _realpath_py

# Pick the newest interpreter *by mode* from `uv python list --only-installed`
# mode: "gil" | "nogil"
pick_latest_by_mode() {
  local mode="${1:-gil}" id
  has_uv || return 1
  if [[ "$mode" == "nogil" ]]; then
    # free-threaded entries contain '+freethreaded'
    id="$(uv python list --only-installed \
        | awk 'tolower($1) ~ /\+freethreaded/ {print $1}' \
        | sort -rV | head -n1 || true)"
  else
    id="$(uv python list --only-installed \
        | awk 'tolower($1) !~ /\+freethreaded/ {print $1}' \
        | sort -rV | head -n1 || true)"
  fi
  [[ -n "${id:-}" ]] || return 1
  uv python find "$id"
}
export -f pick_latest_by_mode

# Resolve a spec (like '3.13t' or 'cpython-3.12.11-…') or an absolute path to an executable
resolve_py_spec() {
  local spec="$1"
  if [[ -x "$spec" ]]; then
    _realpath_py "$spec"
    return 0
  fi
  has_uv && uv python find "$spec" 2>/dev/null || true
}
export -f resolve_py_spec

# Try to install a spec via uv (best-effort; ok if it fails)
ensure_uv_python() {
  local spec="$1"
  has_uv || return 0
  uv python install "$spec" || true
}
export -f ensure_uv_python


# choose_python <mode> [base_dir]
# mode: "gil" | "nogil"
choose_python() {
  local mode="${1:-GIL}"
  local basedir="${2:-$PY_DIR}"
  local spec="" py=""  

  # 3) defaults if no project spec
  if [[ "$mode" == "nogil" ]]; then
    spec="$DEFAULT_PYTHON_VERSION_NOGIL"   # e.g. 3.13t
  else
    if [[ -f "$basedir/.python-version" ]] && [[ $mode != "nogil" ]]; then
        temp="$(_trim "$( <"$basedir/.python-version")")"
        temp_is_nogil=$(echo "$temp" | grep -c 'freethreaded' || true)
        if [[ $temp_is_nogil -eq 1 ]]; then
            spec="$DEFAULT_PYTHON_VERSION"     # e.g. 3.12
        fi
    else
        spec="$DEFAULT_PYTHON_VERSION"     # e.g. 3.12
    fi
  fi

  # Try uv install + resolve
  if [[ -n "$spec" ]] && has_uv; then
    ensure_uv_python "$spec" >/dev/null 2>&1 || true
    py="$(resolve_py_spec "$spec")"
  fi

  # Fallbacks
  [[ -z "${py:-}" ]] && py="$(pick_latest_by_mode "$mode")"
  [[ -z "${py:-}" ]] && py="$(command -v python3 || command -v python || true)"

  [[ -n "${py:-}" ]] || die "No Python interpreter found (mode=$mode, base=$basedir)."
  printf '%s\n' "$py"
}
export -f choose_python

# Ensure a uv venv pinned to the given interpreter & mode
# Usage: configure_uv_env <mode> <python-abs> [venv_dir] [sync_args...]
configure_uv_env() {
  local mode="$1"
  local py="$2"
  local venv_dir="${3:-.venv}"
  shift 3
  local sync_args=("$@")  # zero or more uv sync args

  has_uv || { echo "uv is required for configure_uv_env"; return 1; }

  # Create venv pinned to the exact interpreter if it doesn't exist
  if [[ ! -d "$venv_dir" ]]; then
    local ver spec
    ver="$("$py" -c 'import sys; print(".".join(map(str, sys.version_info[:3])))')"
    if [[ "$mode" == "nogil" ]]; then
      spec="${ver}+freethreaded"
    else
      spec="${ver}"
    fi
    info "creating ${venv_dir} for ${spec}"
    UV_PROJECT_ENVIRONMENT="$venv_dir" uv venv --python "$spec"
    UV_PROJECT_ENVIRONMENT="$venv_dir" uv python pin "$py"
  fi

  # If there’s a project to sync, run `uv sync` with any extra args
  if [[ -f "pyproject.toml" || -f "requirements.txt" ]]; then
    # Default to `--group dev` only if caller passed no args
    if (( ${#sync_args[@]} == 0 )); then
      sync_args=(--group dev)
    fi
    info "syncing dependencies into ${venv_dir} (${sync_args[*]})"
    UV_PROJECT_ENVIRONMENT="$venv_dir" uv sync "${sync_args[@]}" 
  fi
}
export -f configure_uv_env