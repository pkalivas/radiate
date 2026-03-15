set shell := ["bash", "-cu"]

default:
    @just --list

extra-args := ""
py-version := "3.12"

# --------------------------
# Helpers
# --------------------------

help:
    @echo -e "\033[1mAvailable commands:\033[0m"
    @just --list
    @echo

_require-uv:
    @command -v uv >/dev/null 2>&1 || { \
        echo "Error: missing uv command install uv before running this Justfile"; \
        exit 1; \
    }

python-info py=py-version: _require-uv
    @echo "PYTHON = {{py}}"
    @echo "{{py}}" > .python-version

# --------------------------
# Main commands
# --------------------------

sync py=py-version: python-info
    @uv python install {{py}}
    @uv python pin {{py}}
    @uv venv .venv --clear
    @uv pip install --upgrade --compile-bytecode --no-build \
        -r py-radiate/requirements.txt

coverage: _require-uv
    @uv run -m pytest -n auto \
        --cov=radiate \
        --cov-report=term-missing

lint: _require-uv
    @uv run ruff check radiate tests examples

format: _require-uv
    @uv run ruff format radiate tests examples

wheel py=py-version args=extra-args:
    @uv run maturin build -i {{py}} -m py-radiate/Cargo.toml --release {{args}}

sdist py=py-version args=extra-args: 
    @uv run maturin build -i {{py}} -m py-radiate/Cargo.toml --release --sdist {{args}}

develop py=py-version: 
    @just sync {{py}}
    @uv run maturin develop -m py-radiate/Cargo.toml --uv 

release py=py-version: 
    @just sync {{py}}
    @uv run maturin develop -m py-radiate/Cargo.toml --release --uv

# --------------------------
# Test commands
# --------------------------

test lang="":
    @if [ "{{lang}}" = "py" ]; then \
        just test-py; \
    elif [ "{{lang}}" = "rs" ]; then \
        just test-rs; \
    elif [ -z "{{lang}}" ]; then \
        just test-py && just test-rs; \
    else \
        echo "Error: unsupported language '{{lang}}'. Supported: py, rs"; \
        exit 1; \
    fi

test-py: _require-uv
    @uv run -m pytest -n auto

test-rs:
    @cargo test

# --------------------------
# Example commands
# --------------------------

example lang="py" id="":
     @uv run python tools/examples.py run --lang {{lang}} --id {{id}}

# --------------------------
# Cleaning
# --------------------------

clean:
    @rm -rf .venv
    @rm -rf dist/
    @rm -rf target/
    @rm -rf .pytest_cache
    @just py-radiate/clean
