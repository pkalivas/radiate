#!/usr/bin/env bash
set -euo pipefail

# run.sh
# Usage:
#   ./run.sh --example <name> [-p | -r] [--list] [--debug]
# Notes:
#   - If <name> ends with ".py", Python mode is assumed.
#   - Python examples are run from py-radiate/examples/<name>.py via `uv run`.
#   - Rust examples are searched under examples/** (top-level, graphs/, trees/, etc.) and run with cargo.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
PY_DIR="$REPO_ROOT/py-radiate"
RUST_DIR="$REPO_ROOT/examples"

LANG=""                # "P" or "R"; if empty we'll try to infer
EXAMPLE_NAME=""
DO_LIST=0
CARGO_PROFILE="--release"  # use --debug to flip

usage() {
  cat <<EOF
Usage: $0 --example <name> [-p | -r] [--list] [--debug]

Options:
  --example <name>   Example name or pattern. For Python, you can use "foo" or "foo.py".
  -p                 Force Python example.
  -r                 Force Rust example.
  --list             Show matching candidates and exit.
  --debug            Use debug profile for cargo (default is --release).
  -h, --help         Show this help.

Examples:
  $0 --example pagerank -r
  $0 --example graphs/pagerank   # also works for Rust
  $0 --example image_filter.py -p
EOF
}

info(){ printf '\033[1;36m[INFO]\033[0m %s\n' "$*"; }
warn(){ printf '\033[1;33m[WARN]\033[0m %s\n' "$*"; }
die(){ printf '\033[1;31m[ERROR]\033[0m %s\n' "$*" >&2; exit 1; }

# ---- args ----
while [[ $# -gt 0 ]]; do
  case "$1" in
    --example) EXAMPLE_NAME="${2:-}"; shift 2;;
    -p) LANG="P"; shift;;
    -r) LANG="R"; shift;;
    --list) DO_LIST=1; shift;;
    --debug) CARGO_PROFILE=""; shift;;
    -h|--help) usage; exit 0;;
    *) die "Unknown arg: $1";;
  esac
done

[[ -n "${EXAMPLE_NAME:-}" ]] || { usage; die "Missing --example <name>"; }

# Normalize Python example path (accepts "foo" or "foo.py")
py_example_path() {
  local name="$1"
  local stem="${name%.py}"
  echo "$PY_DIR/examples/${stem}.py"
}

# Find Rust example directories that look runnable (contain Cargo.toml)
# Accepts patterns like "pagerank", "graphs/pagerank", "trees/*pagerank*"
find_rust_examples() {
  local pat="$1"
  # 1) If the user passed a subpath like graphs/foo, prefer that subtree
  local base="$RUST_DIR"
  if [[ "$pat" == graphs/* || "$pat" == trees/* ]]; then
    base="$RUST_DIR/$(dirname "$pat")"
    pat="$(basename "$pat")"
  fi
  # 2) Search 3 levels deep to cover top-level examples + graphs/* + trees/*
  #    Match directory names against the pattern, keep those with Cargo.toml.
  find "$base" -maxdepth 3 -type d -iname "*${pat}*" 2>/dev/null \
    | while read -r d; do
        [[ -f "$d/Cargo.toml" ]] && echo "$d"
      done \
    | sort
}

run_python() {
  local example_file
  example_file="$(py_example_path "$EXAMPLE_NAME")"
  [[ -f "$example_file" ]] || die "Python example not found: $example_file"

  info "Changing dir → $PY_DIR"
  cd "$PY_DIR"

  info "Syncing Python deps with uv (if needed)"
  uv sync

  info "Running: uv run ${example_file#$PY_DIR/}"
  uv run "${example_file#$PY_DIR/}"
}

run_rust() {
  local matches
  mapfile -t matches < <(find_rust_examples "$EXAMPLE_NAME")

  if (( ${#matches[@]} == 0 )); then
    die "No Rust examples matching '$EXAMPLE_NAME' in $RUST_DIR"
  fi

  if (( DO_LIST == 1 )); then
    printf "Matches:\n"
    printf '  %s\n' "${matches[@]}"
    exit 0
  fi

  if (( ${#matches[@]} > 1 )); then
    warn "Multiple matches for '$EXAMPLE_NAME'; choosing the first:"
    printf '  %s\n' "${matches[@]}"
  fi

  local dir="${matches[0]}"
  info "Changing dir → $dir"
  cd "$dir"

  info "Running: cargo run ${CARGO_PROFILE:-"--debug"}"
  cargo run ${CARGO_PROFILE:-}
}

if [[ -z "$LANG" ]]; then
  if [[ "$EXAMPLE_NAME" == *.py ]]; then
    LANG="P"
  else
    LANG="R"
  fi
fi

# ---- dispatch ----
case "$LANG" in
  P) run_python ;;
  R) run_rust ;;
  *) die "Language not specified or unrecognized. Use -p (Python) or -r (Rust)." ;;
esac
