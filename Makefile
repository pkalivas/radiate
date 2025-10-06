.DEFAULT_GOAL := help

PY=$(shell which python3)
SHELL=bash

ifeq ($(VENV),)
VENV := .venv
endif

ifeq ($(OS),Windows_NT)
	VENV_BIN=$(VENV)/Scripts
else
	VENV_BIN=$(VENV)/bin
endif

# Define command to filter pip warnings when running maturin
FILTER_PIP_WARNINGS=| grep -v "don't match your environment"; test $${PIPESTATUS[0]} -eq 0

.venv:
	@echo 'python executable: $(PY)'
	@set -e; \
		PYBIN=$$(uv python find "$(PY)" 2>/dev/null || echo "$(PY)"); \
		"$$PYBIN" -m venv "$(VENV)"; 
	$(MAKE) requirements

.PHONY: requirements
requirements: .venv  ## Install/refresh Python project requirements
	@$(VENV_BIN)/python -m pip install --upgrade uv pip \
	&& $(VENV_BIN)/uv pip install --upgrade --compile-bytecode --no-build \
	   -r py-radiate/requirements-dev.txt 

.PHONY: build
build: .venv  ## Compile and install Python radiate for development
	@$(VENV_BIN)/maturin develop -m py-radiate/Cargo.toml $(ARGS) \
	$(FILTER_PIP_WARNINGS)

.PHONY: wheel
wheel: .venv  ## Build a wheel for Python radiate
	@$(VENV_BIN)/maturin build -i $(PY) -m py-radiate/Cargo.toml $(ARGS) \
	$(FILTER_PIP_WARNINGS)

.PHONY: test-py
test-py:  ## Run Python unittests
	@$(MAKE) -s -C py-radiate test

.PHONY: test-rs
test-rs:  ## Run Rust unittests
	@cargo test --all-features

.PHONY: clean
clean:  ## Clean up build artifacts
	@rm -rf target/ 
	@rm -rf .venv
	@rm -rf .benchmarks/
	@rm -rf .pytest_cache/
	@rm -rf .ruff_cache/
	@rm -f .coverage
	@$(MAKE) -s -C py-radiate clean

.PHONY: help
help:  ## Display this help screen
	@echo -e "\033[1mAvailable commands:\033[0m"
	@grep -E '^[a-z.A-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}' | sort
	@echo
	@echo 'For example to build without default features use: make build ARGS="--no-default-features".'
