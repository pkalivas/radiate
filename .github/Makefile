# ==== Radiate Actions Makefile ====
# Override on the command line or .env (optional)
OWNER       ?= pkalivas
REPO        ?= radiate
REF         ?= master

# Workflow filenames (use file names or IDs)
WF_PAGES    ?= pages.yaml
WF_PUBLISH  ?= publish.yml
WF_PYTEST   ?= py-unit-tests.yml
WF_RUSTEST  ?= rust-unit-tests.yml

# Optional: load local .env with KEY=VALUE lines
ifneq ("$(wildcard .env)","")
include .env
export $(shell sed -n 's/^\([A-Za-z_][A-Za-z0-9_]*\)=.*/\1/p' .env)
endif

# Map INPUTS="a=b c=d" -> "-f a=b -f c=d"
define map_inputs
$(foreach p,$(INPUTS),-f $(p))
endef

# Helper: get latest run id for a workflow
define latest_run_id
gh run list -R $(OWNER)/$(REPO) --workflow "$(1)" --json databaseId -q '.[0].databaseId'
endef


.PHONY: help
help:
	@echo "Workflows in $(OWNER)/$(REPO) (ref=$(REF))"
	@echo "  make list                        # list workflows"
	@echo "  make runs WORKFLOW=<file.yml>    # list recent runs"
	@echo "  make trigger WORKFLOW=<file.yml> [REF=...] [INPUTS=\"k=v ...\"]"
	@echo "  make watch  WORKFLOW=<file.yml>  # watch latest run"
	@echo "  make logs   WORKFLOW=<file.yml>  # logs for latest run"
	@echo
	@echo "Shortcuts:"
	@echo "  make pages                       # trigger docs/pages deploy"
	@echo "  make publish                     # trigger sdist+wheels pipeline"
	@echo "  make pytest                      # trigger Python unit tests"
	@echo "  make rusttest                    # trigger Rust unit tests"
	@echo "  make ci                          # trigger both test workflows"

.PHONY: auth
auth:
	gh auth status

.PHONY: list
list:
	gh workflow list -R $(OWNER)/$(REPO)

.PHONY: runs
runs:
	@test -n "$(WORKFLOW)" || (echo "WORKFLOW=<file.yml> required"; exit 1)
	gh run list -R $(OWNER)/$(REPO) --workflow "$(WORKFLOW)" --limit 25

.PHONY: trigger
trigger:
	@test -n "$(WORKFLOW)" || (echo "WORKFLOW=<file.yml> required"; exit 1)
	@echo "Triggering $(WORKFLOW) on ref=$(REF) with inputs: $(INPUTS)"
	gh workflow run "$(WORKFLOW)" -R $(OWNER)/$(REPO) --ref "$(REF)" $(call map_inputs)


.PHONY: logs
logs:
	@test -n "$${WORKFLOW:-}" || WORKFLOW="$(WF_PYTEST)"; \
	run_id="$$(gh run list -R $(OWNER)/$(REPO) --workflow "$$WORKFLOW" --json databaseId -q '.[0].databaseId')"; \
	if [ -z "$$run_id" ]; then echo "No runs found for $$WORKFLOW"; exit 1; fi; \
	echo "Showing logs for run $$run_id (workflow=$$WORKFLOW)"; \
	gh run view -R $(OWNER)/$(REPO) "$$run_id" --log

.PHONY: watch
watch:
	@test -n "$${WORKFLOW:-}" || WORKFLOW="$(WF_PYTEST)"; \
	run_id="$$(gh run list -R $(OWNER)/$(REPO) --workflow "$$WORKFLOW" --json databaseId -q '.[0].databaseId')"; \
	if [ -z "$$run_id" ]; then echo "No runs found for $$WORKFLOW"; exit 1; fi; \
	echo "Watching run $$run_id (workflow=$$WORKFLOW)"; \
	gh run watch -R $(OWNER)/$(REPO) "$$run_id" --exit-status

.PHONY: pages publish pytest rusttest ci
pages:
	$(MAKE) trigger WORKFLOW=$(WF_PAGES)

publish:
	$(MAKE) trigger WORKFLOW=$(WF_PUBLISH)

pytest:
	$(MAKE) trigger WORKFLOW=$(WF_PYTEST)

rusttest:
	$(MAKE) trigger WORKFLOW=$(WF_RUSTEST)

ci:
	$(MAKE) pytest
	$(MAKE) rusttest