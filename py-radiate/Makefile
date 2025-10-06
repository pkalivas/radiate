.DEFAULT_GOAL := help

SHELL=bash
ifeq ($(VENV),)
VENV := ../.venv
endif

ifeq ($(OS),Windows_NT)
	VENV_BIN=$(VENV)/Scripts
else
	VENV_BIN=$(VENV)/bin
endif

.PHONY: .venv
.venv:  ## Set up virtual environment and install requirements
	@$(MAKE) -s -C .. $@

.PHONY: build
build: .venv  ## Compile and install Python Radiate for development
	@$(MAKE) -s -C .. $@

.PHONY: test
test: .venv build  ## Run fast unittests
	@$(VENV_BIN)/pytest -n auto $(PYTEST_ARGS)

.PHONY: clean
clean:  ## Clean up caches and build artifacts
	@rm -rf .benchmarks/
	@rm -rf .pytest_cache/
	@rm -f .coverage
	@rm -f radiate/*.so
	@rm -rf .ruff_cache/
	@find . -type f -name '*.py[co]' -delete -or -type d -name __pycache__ -exec rm -r {} +