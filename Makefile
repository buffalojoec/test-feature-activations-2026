# Dynamic program discovery.
PROGRAMS := $(shell find . -maxdepth 1 -type d -name 'simd-*' -exec test -f {}/Cargo.toml \; -print | sed 's|./||' | sort)

.PHONY: build

list:
	@for prog in $(PROGRAMS); do echo $$prog; done

# Build all programs.
build: $(addprefix build-,$(PROGRAMS))

# Build a specific program.
build-%:
	cargo build-sbf --manifest-path $*/Cargo.toml

# Deploy a specific program.
deploy-%:
	solana program deploy target/deploy/$(subst -,_,$*).so

# Get a program ID.
get-id-%:
	solana address -k $*/keypair.json

# Run a program test binary.
test-%:
	cargo run -p $*

fmt:
	cargo +nightly fmt --all --check

fmt-fix:
	cargo +nightly fmt --all

clippy:
	cargo +nightly clippy --all-targets -- -D warnings

clean:
	cargo clean
