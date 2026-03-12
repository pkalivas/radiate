.DEFAULT_GOAL := help

REQUIRE_UV := $(shell command -v uv 2>/dev/null || echo "missing uv command install uv before running this Makefile")

$(REQUIRE_UV):
	@echo "Error: $(REQUIRE_UV)"
	@exit 1

ARGS ?=
PY ?= 3.13

find_python = $(shell uv python find "$(1)" 2>/dev/null || true)
require_python = $(or $(call find_python,$(1)),$(error Could not find Python interpreter '$(1)'))

python = $(call require_python,$(PY))
is_free_threaded = $(shell $(python) -c "import sys; print('yes' if getattr(sys, '_is_gil_enabled', lambda: True)() == False else 'no')")

ifeq ($(is_free_threaded),yes)
SYNC_ARGS = --only-group common 
else
SYNC_ARGS = --group dev
endif

.PHONY: python-info
python-info: 
	@echo "PYTHON     = $(PY)"
	@echo "Using Python: $(python) (free-threaded: $(is_free_threaded))"

.PHONY: sync
sync: python-info  ## Sync Python dependencies with uv
	@uv python install $(PY)
	@uv python pin $(PY)
	@uv sync $(SYNC_ARGS) 

.PHONY: test
test: ## Run fast unittests
	@uv run -m pytest -n auto

.PHONY: coverage
coverage: ## Run tests with coverage report
	@uv run -m pytest -n auto \
		--cov=radiate \
		--cov-report=term-missing \

.PHONY: lint
lint: ## Run linters
	@uv run ruff check radiate tests examples

.PHONY: format
format: ## Run code formatters
	@uv run ruff format radiate tests examples

.PHONY: wheel
wheel: sync  ## Build wheel distribution
	@uv run maturin build -m Cargo.toml --release $(ARGS) 

.PHONY: sdist
sdist: sync ## Build source distribution
	@uv run maturin build -m Cargo.toml --release --sdist $(ARGS)

.PHONY: develop
develop: sync  ## Install Python radiate for development
	@uv run maturin develop --uv

.PHONY: release
release: sync  ## Build radiate in release mode for development
	@uv run maturin develop --release --uv

.PHONY: clean
clean:  ## Clean up caches and build artifacts
	@rm -rf .benchmarks/
	@rm -rf .pytest_cache/
	@rm -f .coverage
	@rm -f radiate/*.so
	@rm -rf .ruff_cache/
	@rm -rf .venv
	@rm -rf examples/data
	@find . -type f -name '*.py[co]' -delete -or -type d -name __pycache__ -exec rm -r {} +
	@rm -rf dist/
	@rm -rf tests/.coverage
	@rm -rf radiate/libradiate.dylib.dSYM
	@rm -rf .ruff_cache/
