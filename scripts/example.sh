#!/usr/bin/env bash

LANG=""
EXAMPLE_NAME=""

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
PY_DIR="$REPO_ROOT/py-radiate"
RUST_DIR="$REPO_ROOT/examples"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --example) EXAMPLE_NAME="${2:-}"; shift 2;;
    -p) LANG="P"; shift;;
    -r) LANG="R"; shift;;
    *) echo "Unknown arg: $1" >&2; exit 1;;
  esac
done

if [[ -z "$EXAMPLE_NAME" ]]; then
  echo "Usage: $0 --example <example_name> [-p|-r]"
  echo "  -p for Python example"
  echo "  -r for Rust example"
  exit 1
fi

if [[ "$LANG" == "P" ]]; then
    cd $PY_DIR
    uv sync
    uv run examples/$EXAMPLE_NAME.py
elif [[ "$LANG" == "R" ]]; then
    cd $RUST_DIR
    example_names=$(ls | grep $EXAMPLE_NAME | tr '\n' ' ')

    for ex in $example_names; do
      if [[ "$ex" == *"$EXAMPLE_NAME"* ]]; then
          cd "$RUST_DIR/$ex"
          cargo run --release
          exit 0
      fi
    done

    is_graph=$(grep -q "graph" <<< "$EXAMPLE_NAME" && echo 1 || echo 0)
    is_tree=$(grep -q "tree" <<< "$EXAMPLE_NAME" && echo 1 || echo 0)

    if [[ $is_graph -eq 0 && $is_tree -eq 0 ]]; then
        echo "Example name must include 'graph' or 'tree' to identify the type."
        exit 1
    fi

    if [[ $is_graph -eq 1 ]]; then
        cd $RUST_DIR/graphs
        example_names=$(ls | grep $EXAMPLE_NAME | tr '\n' ' ')
        for ex in $example_names; do
            if [[ "$ex" == *"$EXAMPLE_NAME"* ]]; then
                cd $ex
                cargo run --release
                exit 0
            fi
        done
        
    elif [[ $is_tree -eq 1 ]]; then
        cd $RUST_DIR/trees
        example_names=$(ls | grep $EXAMPLE_NAME | tr '\n' ' ')
        for ex in $example_names; do
            if [[ "$ex" == *"$EXAMPLE_NAME"* ]]; then
                cd $ex
                cargo run --release
                exit 0
            fi
        done
    fi
else
  echo "Language not specified or unrecognized. Use -p (Python) or -r (R)."
  exit 1
fi