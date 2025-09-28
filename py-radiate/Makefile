SHELL := bash

ARGS ?=
GIL  ?= 1

.PHONY: develop build develop-gil develop-nogil build-gil build-nogil test clean

# Choose develop target based on GIL
ifeq ($(GIL),1)
develop: develop-gil
build: build-gil
else
develop: develop-nogil
build: build-nogil
endif

develop-gil:
	$(MAKE) clean
	../scripts/build_wheel.sh

develop-nogil:
	$(MAKE) clean
	../scripts/build_wheel.sh --freethreaded

build-gil:
	$(MAKE) clean
	../scripts/build_wheel.sh --build

build-nogil:
	$(MAKE) clean
	../scripts/build_wheel.sh --freethreaded --build

test: develop
	uv run pytest $(ARGS)

clean:
	@rm -rf .venv
	@rm -rf radiate/*.so
	@rm -rf .coverage
