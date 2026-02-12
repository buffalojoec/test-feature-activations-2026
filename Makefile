# Dynamic program discovery.
PROGRAMS := $(shell find . -maxdepth 1 -type d -name 'simd-*' -exec test -f {}/Cargo.toml \; -print | sed 's|./||' | sort)

.PHONY: list build deploy test get-id help clean $(PROGRAMS)

help:
	@echo "Usage:"
	@echo "  make list              - List available programs."
	@echo "  make build             - Build all programs."
	@echo "  make build-<program>   - Build a specific program."
	@echo "  make deploy-<program>  - Deploy a specific program."
	@echo "  make get-id-<program>  - Get program ID."
	@echo "  make test-<program>    - Run program test binary."
	@echo ""
	@echo "Available programs: $(PROGRAMS)"

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

clean:
	cargo clean
