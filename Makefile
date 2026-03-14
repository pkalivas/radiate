.DEFAULT_GOAL := help

.PHONY: develop
develop:
	@$(MAKE) -s -C py-radiate develop

.PHONY: release
release:
	@$(MAKE) -s -C py-radiate release

.PHONY: wheel
wheel:
	@$(MAKE) -s -C py-radiate wheel $@

.PHONY: sdist
sdist:
	@$(MAKE) -s -C py-radiate sdist $@

.PHONY: test-py
test-py:  ## Run Python unittests
	@$(MAKE) -s -C py-radiate test

.PHONY: py-cov
py-cov:  ## Run Python tests with coverage report
	@$(MAKE) -s -C py-radiate coverage 

.PHONY: test-rs
test-rs:  ## Run Rust unittests
	@cargo test --all-features

.PHONY: test
test: test-py test-rs  ## Run all unittests - Python and Rust

.PHONY: clean
clean:  ## Clean up build artifacts
	@rm -rf target/ 
	@rm -rf .venv/
	@$(MAKE) -s -C py-radiate clean

.PHONY: help
help:  ## Display this help screen
	@echo -e "\033[1mAvailable commands:\033[0m"
	@grep -E '^[a-z.A-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}' | sort
	@echo
	@echo 'For example to build without default features use: make build ARGS="--no-default-features".'
