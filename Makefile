.DEFAULT_GOAL := help

REQUIRE_UV := $(shell command -v uv 2>/dev/null || echo "missing uv command install uv before running this Makefile")

$(REQUIRE_UV):
	@echo "Error: $(REQUIRE_UV)"
	@exit 1

ARGS ?=
PY ?= 3.12
GH ?= gh
WORKFLOW ?= publish
REF ?= master
TARGET_GROUP ?= all
FREE_THREADED ?= true
PUBLISH_TO_PYPI ?= false

.PHONY: python-info
python-info:  ## Display Python interpreter information
	@echo "PYTHON     = $(PY)"
	@echo $(PY) > .python-version

.PHONY: sync
sync: python-info  ## Sync Python dependencies with uv
	@uv python install $(PY)
	@uv python pin $(PY)
	@uv venv .venv --clear
	@uv pip install -r py-radiate/requirements.txt -r docs/requirements.txt

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

# PHONY targets for GitHub Actions workflows
.PHONY: gha-build
gha-build:  ## Trigger GitHub Actions workflow

	@$(GH) workflow run $(WORKFLOW) \
		--ref $(REF) \
		-f ref=$(REF) \
		-f target_group=$(TARGET_GROUP) \
		-f free_threaded=$(FREE_THREADED) \
		-f publish_to_pypi=$(PUBLISH_TO_PYPI)

.PHONY: gha-build-linux
gha-build-linux:  ## Trigger GitHub Actions workflow for Linux targets
	@$(MAKE) gha-build TARGET_GROUP=linux

.PHONY: gha-build-musllinux
gha-build-musllinux: ## Trigger GitHub Actions workflow for musllinux targets
	@$(MAKE) gha-build TARGET_GROUP=musllinux

.PHONY: gha-build-windows
gha-build-windows: ## Trigger GitHub Actions workflow for Windows targets
	@$(MAKE) gha-build TARGET_GROUP=windows

.PHONY: gha-build-macos
gha-build-macos: ## Trigger GitHub Actions workflow for macOS targets
	@$(MAKE) gha-build TARGET_GROUP=macos

.PHONY: gha-build-sdist
gha-build-sdist: ## Trigger GitHub Actions workflow for source distribution
	@$(MAKE) gha-build TARGET_GROUP=sdist FREE_THREADED=false

.PHONY: gha-publish
gha-publish: ## Trigger GitHub Actions workflow for publishing to PyPI
	@$(MAKE) gha-build TARGET_GROUP=all FREE_THREADED=true PUBLISH_TO_PYPI=true

.PHONY: gha-watch
gha-watch: ## Watch the latest GitHub Actions workflow run
	@RUN_ID=`$(GH) run list --workflow $(WORKFLOW) --limit 1 --json databaseId --jq '.[0].databaseId'`; \
	$(GH) run watch $$RUN_ID --compact --exit-status

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

# .DEFAULT_GOAL := help

# .PHONY: develop
# develop:
# 	@$(MAKE) -s -C py-radiate develop

# .PHONY: release
# release:
# 	@$(MAKE) -s -C py-radiate release

# .PHONY: wheel
# wheel:
# 	@$(MAKE) -s -C py-radiate wheel $@

# .PHONY: sdist
# sdist:
# 	@$(MAKE) -s -C py-radiate sdist $@

# .PHONY: test-py
# test-py:  ## Run Python unittests
# 	@$(MAKE) -s -C py-radiate test

# .PHONY: py-cov
# py-cov:  ## Run Python tests with coverage report
# 	@$(MAKE) -s -C py-radiate coverage 

# .PHONY: test-rs
# test-rs:  ## Run Rust unittests
# 	@cargo test --all-features

# .PHONY: test
# test: test-py test-rs  ## Run all unittests - Python and Rust

# .PHONY: clean
# clean:  ## Clean up build artifacts
# 	@rm -rf target/ 
# 	@rm -rf .venv/
# 	@$(MAKE) -s -C py-radiate clean

# .PHONY: help
# help:  ## Display this help screen
# 	@echo -e "\033[1mAvailable commands:\033[0m"
# 	@grep -E '^[a-z.A-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}' | sort
# 	@echo
# 	@echo 'For example to build without default features use: make build ARGS="--no-default-features".'
