ARGS =

.PHONY: develop
develop:
	../scripts/build.sh

.PHONY: develop-nogil
develop-nogil:
	../scripts/build.sh --freethreaded

.PHONY: build-gil
build-gil:
	../scripts/build.sh --build

.PHONY: build-nogil
build-nogil:
	../scripts/build.sh --freethreaded --build

.PHONY: build-both
build-both:
	../scripts/build.sh --build && ../scripts/build.sh --freethreaded --build

.PHONY: test
test:
	uv run pytest $(ARGS)

.PHONY: clean
clean:
	@rm -rf .venv
	@rm -rf radiate/*.so
	@rm -rf .coverage
