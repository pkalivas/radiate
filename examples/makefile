TOOLS := ../tools/examples.py
PYTHON := uv run python

.PHONY: examples-list
examples-list: ## List available examples
	@$(PYTHON) $(TOOLS) list

.PHONY: examples
examples: ## Forward args after --, e.g. `make examples -- run --id nn`
	@$(PYTHON) $(TOOLS) $(filter-out $@ --,$(MAKECMDGOALS))

%:
	@: