# ==== Radiate Actions Makefile (explicit, no defaults) ====
OWNER        ?= pkalivas
REPO         ?= radiate
REF          ?= main

# Workflow filenames (adjust to your repo)
WF_PAGES     ?= pages.yaml
WF_PUBLISH   ?= publish.yml
WF_PYTEST    ?= py-unit-tests.yml
WF_RUSTEST   ?= rust-unit-tests.yml

# Optional GHES host
GITHUB_HOST  ?=
ifdef GITHUB_HOST
GH_HOST_ARG := --hostname $(GITHUB_HOST)
else
GH_HOST_ARG :=
endif

# Load .env if present
ifneq ("$(wildcard .env)","")
include .env
export $(shell sed -n 's/^\([A-Za-z_][A-Za-z0-9_]*\)=.*/\1/p' .env)
endif

# INPUTS="a=b c=d" -> "-f a=b -f c=d"
define map_inputs
$(foreach p,$(INPUTS),-f $(p))
endef

define assert_gh
	@command -v gh >/dev/null 2>&1 || { echo "error: GitHub CLI 'gh' not found"; exit 127; }
	@gh auth status $(GH_HOST_ARG) >/dev/null 2>&1 || { echo "error: gh not authenticated"; gh auth status $(GH_HOST_ARG); exit 1; }
endef

# Require WORKFLOW and fetch latest run id/URL for it
define require_workflow
	@test -n "$${WORKFLOW:-}" || { echo "WORKFLOW=<file.yml> is required"; exit 1; }
endef

define get_latest_run_id
	run_id="$$(gh run list -R $(OWNER)/$(REPO) --workflow "$$WORKFLOW" --json databaseId -q '.[0].databaseId' $(GH_HOST_ARG))"; \
	test -n "$$run_id" || { echo "No runs found for $$WORKFLOW"; exit 1; }
endef

define get_latest_run_url
	url="$$(gh run list -R $(OWNER)/$(REPO) --workflow "$$WORKFLOW" --json url -q '.[0].url' $(GH_HOST_ARG))"; \
	test -n "$$url" || { echo "No runs found for $$WORKFLOW"; exit 1; }
endef

.PHONY: help
help:
	@echo "Repo: $(OWNER)/$(REPO)  (ref=$(REF))"
	@echo "Core:"
	@echo "  make list"
	@echo "  make runs WORKFLOW=<file.yml>"
	@echo "  make trigger WORKFLOW=<file.yml> [REF=...] [INPUTS=\"k=v ...\"]"
	@echo "  make logs   WORKFLOW=<file.yml>"
	@echo "  make logs-failed WORKFLOW=<file.yml>"
	@echo "  make watch  WORKFLOW=<file.yml>"
	@echo "  make open   WORKFLOW=<file.yml>"
	@echo "  make url    WORKFLOW=<file.yml>"
	@echo "  make rerun  WORKFLOW=<file.yml>"
	@echo "  make cancel WORKFLOW=<file.yml>"
	@echo
	@echo "Shortcut triggers: pages | publish | pytest | rusttest | ci"
	@echo "Shortcut inspectors: logs-py | logs-rust | logs-publish | watch-py | watch-rust | watch-publish"

.PHONY: auth
auth:
	$(call assert_gh)
	@gh auth status $(GH_HOST_ARG)

.PHONY: list
list:
	$(call assert_gh)
	gh workflow list -R $(OWNER)/$(REPO) $(GH_HOST_ARG)

.PHONY: runs
runs:
	$(call assert_gh)
	$(call require_workflow)
	gh run list -R $(OWNER)/$(REPO) --workflow "$(WORKFLOW)" --limit 25 $(GH_HOST_ARG)

.PHONY: trigger
trigger:
	$(call assert_gh)
	$(call require_workflow)
	@echo "Triggering $(WORKFLOW) on ref=$(REF) with inputs: $(INPUTS)"
	gh workflow run "$(WORKFLOW)" -R $(OWNER)/$(REPO) --ref "$(REF)" $(call map_inputs) $(GH_HOST_ARG)

# ---- Inspect/manage (explicit WORKFLOW always required) ----
.PHONY: logs
logs:
	$(call assert_gh)
	$(call require_workflow)
	@$(call get_latest_run_id); \
	echo "Logs for $$WORKFLOW (run $$run_id)"; \
	gh run view -R $(OWNER)/$(REPO) "$$run_id" --log $(GH_HOST_ARG)

.PHONY: logs-failed
logs-failed:
	$(call assert_gh)
	$(call require_workflow)
	@$(call get_latest_run_id); \
	echo "Failed-step logs for $$WORKFLOW (run $$run_id)"; \
	gh run view -R $(OWNER)/$(REPO) "$$run_id" --log-failed $(GH_HOST_ARG)

.PHONY: watch
watch:
	$(call assert_gh)
	$(call require_workflow)
	@$(call get_latest_run_id); \
	echo "Watching $$WORKFLOW (run $$run_id)"; \
	gh run watch -R $(OWNER)/$(REPO) "$$run_id" --exit-status $(GH_HOST_ARG)

.PHONY: open
open:
	$(call assert_gh)
	$(call require_workflow)
	@$(call get_latest_run_url); \
	echo "$$url"; \
	gh run view -R $(OWNER)/$(REPO) --web "$$url" 2>/dev/null || \
	open "$$url" 2>/dev/null || xdg-open "$$url" 2>/dev/null || true

.PHONY: url
url:
	$(call assert_gh)
	$(call require_workflow)
	@$(call get_latest_run_url); echo "$$url"

.PHONY: rerun
rerun:
	$(call assert_gh)
	$(call require_workflow)
	@$(call get_latest_run_id); \
	echo "Rerunning $$WORKFLOW (run $$run_id)"; \
	gh run rerun -R $(OWNER)/$(REPO) "$$run_id" $(GH_HOST_ARG)

.PHONY: cancel
cancel:
	$(call assert_gh)
	$(call require_workflow)
	@$(call get_latest_run_id); \
	echo "Cancelling $$WORKFLOW (run $$run_id)"; \
	gh run cancel -R $(OWNER)/$(REPO) "$$run_id" $(GH_HOST_ARG)


.PHONY: pages publish pytest rusttest ci
pages:    ; $(MAKE) trigger WORKFLOW=$(WF_PAGES)
publish:  ; $(MAKE) trigger WORKFLOW=$(WF_PUBLISH)
pytest:   ; $(MAKE) trigger WORKFLOW=$(WF_PYTEST)
rusttest: ; $(MAKE) trigger WORKFLOW=$(WF_RUSTEST)
ci:       ; $(MAKE) pytest && $(MAKE) rusttest

.PHONY: logs-py logs-rust logs-publish watch-py watch-rust watch-publish
logs-py:        ; $(MAKE) logs  WORKFLOW=$(WF_PYTEST)
logs-rust:      ; $(MAKE) logs  WORKFLOW=$(WF_RUSTEST)
logs-publish:   ; $(MAKE) logs  WORKFLOW=$(WF_PUBLISH)

watch-py:       ; $(MAKE) watch WORKFLOW=$(WF_PYTEST)
watch-rust:     ; $(MAKE) watch WORKFLOW=$(WF_RUSTEST)
watch-publish:  ; $(MAKE) watch WORKFLOW=$(WF_PUBLISH)