ARGS =

.PHONY: build
build:
	cargo build --release

.PHONY: test-rs
test-rs:
	cargo test --all-features

.PHONY: test-py
test-py:
	$(MAKE) -C py-radiate test ARGS='$(ARGS)'

.PHONY: test
test: test-rs test-py

.PHONY: clean
clean:
	cargo clean
	$(MAKE) -C py-radiate clean


.PHONY: nogil-env
nogil-env:
	$(MAKE) -C py-radiate clean
	$(MAKE) -C py-radiate develop-nogil

.PHONY: gil-env
gil-env:
	$(MAKE) -C py-radiate clean
	$(MAKE) -C py-radiate develop
