GH ?= gh
WORKFLOW ?= publish
REF ?= master
TARGET_GROUP ?= all
FREE_THREADED ?= true
PUBLISH_TO_PYPI ?= false

.PHONY: gha-build
gha-build:
	@$(GH) workflow run $(WORKFLOW) \
		--ref $(REF) \
		-f ref=$(REF) \
		-f target_group=$(TARGET_GROUP) \
		-f free_threaded=$(FREE_THREADED) \
		-f publish_to_pypi=$(PUBLISH_TO_PYPI)

.PHONY: gha-build-linux
gha-build-linux:
	@$(MAKE) gha-build TARGET_GROUP=linux

.PHONY: gha-build-musllinux
gha-build-musllinux:
	@$(MAKE) gha-build TARGET_GROUP=musllinux

.PHONY: gha-build-windows
gha-build-windows:
	@$(MAKE) gha-build TARGET_GROUP=windows

.PHONY: gha-build-macos
gha-build-macos:
	@$(MAKE) gha-build TARGET_GROUP=macos

.PHONY: gha-build-sdist
gha-build-sdist:
	@$(MAKE) gha-build TARGET_GROUP=sdist FREE_THREADED=false

.PHONY: gha-publish
gha-publish:
	@$(MAKE) gha-build TARGET_GROUP=all FREE_THREADED=true PUBLISH_TO_PYPI=true

.PHONY: gha-watch
gha-watch:
	@RUN_ID=`$(GH) run list --workflow $(WORKFLOW) --limit 1 --json databaseId --jq '.[0].databaseId'`; \
	$(GH) run watch $$RUN_ID --compact --exit-status