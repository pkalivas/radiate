.DEFAULT_GOAL := help

REQUIRE_UV := $(shell command -v uv 2>/dev/null || echo "missing uv command install uv before running this Makefile")

$(REQUIRE_UV):
	@echo "Error: $(REQUIRE_UV)"
	@exit 1

ARGS ?=
PY ?= 3.12

.PHONY: python-info
python-info:  ## Display Python interpreter information
	@echo "PYTHON     = $(PY)"
	@echo $(PY) > .python-version

.PHONY: sync
sync: python-info  ## Sync Python dependencies with uv
	@uv python install $(PY)
	@uv python pin $(PY)
	@uv venv .venv --clear
	@uv pip install --upgrade --compile-bytecode --no-build \
		-r py-radiate/requirements.txt 

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
	@uv run maturin build -m py-radiate/Cargo.toml --release $(ARGS) 

.PHONY: sdist
sdist: sync ## Build source distribution
	@uv run maturin build -m py-radiate/Cargo.toml --release --sdist $(ARGS)

.PHONY: develop
develop: sync  ## Install Python radiate for development
	@uv run maturin develop -m py-radiate/Cargo.toml 

.PHONY: release
release: sync  ## Build radiate in release mode for development
	@uv run maturin develop -m py-radiate/Cargo.toml --release --uv


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
	@rm -rf uv.toml
	@rm -rf py-radiate/radiate/*.so


.PHONY: help
help:  ## Display this help screen
	@echo -e "\033[1mAvailable commands:\033[0m"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}' | sort
	@echo
	@echo 'For example to build without default features use: make wheel ARGS="--no-default-features".'	
