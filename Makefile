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
	@unset CONDA_PREFIX \
	&& $(VENV_BIN)/python -m pip install --upgrade uv \
	&& $(VENV_BIN)/uv pip install --upgrade --compile-bytecode --no-build \
	   -r py-radiate/requirements-dev.txt 

.PHONY: build
build: .venv  ## Compile and install Python radiate for development
	@unset CONDA_PREFIX \
	&& $(VENV_BIN)/maturin develop -m py-radiate/Cargo.toml $(ARGS) \
	$(FILTER_PIP_WARNINGS)

.PHONY: wheel
wheel: .venv  ## Build a wheel for Python radiate
	@unset CONDA_PREFIX \
	&& $(VENV_BIN)/maturin build -m py-radiate/Cargo.toml $(ARGS) \
	$(FILTER_PIP_WARNINGS)

.PHONY: clean
clean:  ## Clean up build artifacts
	@rm -rf target/ 
	@rm -rf .venv
	@rm -rf .benchmarks/
	@rm -rf .pytest_cache/
	@rm -f .coverage
	$(MAKE) -s -C py-radiate clean

.PHONY: help
help:  ## Display this help screen
	@echo -e "\033[1mAvailable commands:\033[0m"
	@grep -E '^[a-z.A-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}' | sort
	@echo
	@echo 'For example to build without default features use: make build ARGS="--no-default-features".'
# -------- act settings (Linux-only emulation) --------
WF := .github/workflows/test.yaml

# Map GitHub runners -> act images that include docker CLI
ACT_IMAGES := \
  -P ubuntu-22.04=catthehacker/ubuntu:act-22.04 \
  -P ubuntu-latest=catthehacker/ubuntu:act-22.04 \
  -P macos-latest=nektos/act-environments-macos:latest \

# On Apple Silicon, force amd64 to match GH runners
ACT_ARCH := --container-architecture linux/amd64

# Allow runner container to launch manylinux containers
ACT_DIND := --container-options "--privileged -v /var/run/docker.sock:/var/run/docker.sock"

# Optional: bind-mount workspace instead of copying (faster). Comment out to copy instead.

# Bundle all CLI opts so make expands them correctly
ACT_OPTS := $(ACT_ARCH) $(ACT_IMAGES) 


# Local helper paths
ACT_DOCKER_DIR := .act-docker
ACT_EVENT_DIR  := .github/test-events
ACT_EVENT_FILE := $(ACT_EVENT_DIR)/test.json

.PHONY: act-init
act-init:
	@mkdir -p $(ACT_DOCKER_DIR) $(ACT_EVENT_DIR)
	@printf '{ "auths": {} }\n' > $(ACT_DOCKER_DIR)/config.json
	@printf '{ "ref": "refs/heads/main", "inputs": {} }\n' > $(ACT_EVENT_FILE)

# List jobs in the selected workflow file
.PHONY: act-list
act-list: act-init
	@DOCKER_CONFIG="$(PWD)/$(ACT_DOCKER_DIR)" \
	act -C . -l -W $(WF) $(ACT_OPTS)

# Run the 'wheels' job (Linux matrix entries will execute; macOS/Windows are skipped by act)
.PHONY: act-wheels
act-wheels: act-init
	@DOCKER_CONFIG="$(PWD)/$(ACT_DOCKER_DIR)" \
	act -C . workflow_dispatch -W $(WF) -j wheels $(ACT_OPTS) --eventpath $(ACT_EVENT_FILE)

# Clean local act artifacts
.PHONY: act-clean
act-clean:
	@rm -rf $(ACT_DOCKER_DIR) $(ACT_EVENT_DIR)