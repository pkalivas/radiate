TEST_ARGS =
GIL  = 1

.PHONY: develop develop-nogil develop-gil 

ifeq ($(GIL),1)
develop: develop-gil
else
develop: develop-nogil
endif

develop-nogil:
	$(MAKE) -C py-radiate develop GIL=0

develop-gil:
	$(MAKE) -C py-radiate develop GIL=1

.PHONY: build
build:
	cargo build --release

.PHONY: test-rs
test-rs:
	cargo test --all-features

.PHONY: test-py
test-py:
	$(MAKE) -C py-radiate test TEST_ARGS='$(TEST_ARGS)'


.PHONY: test
test: test-rs test-py

.PHONY: clean
clean:
	cargo clean
	$(MAKE) -C py-radiate clean
