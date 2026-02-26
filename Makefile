PROGRAMS := $(shell find . -maxdepth 1 -type d -name 'simd-*' -exec test -f {}/Cargo.toml \; -print | sed 's|./||' | sort)

.PHONY: build

list:
	@for prog in $(PROGRAMS); do echo $$prog; done

get-id-%:
	solana address -k $*/keypair.json

build: $(addprefix build-,$(PROGRAMS))

build-%:
	cargo build-sbf --manifest-path $*/Cargo.toml

deploy-%:
	solana program deploy target/deploy/$(subst -,_,$*).so --program-id $*/keypair.json

run-%:
	cargo run -p $* --features bin $(if $(NETWORK),-- $(NETWORK))

run-simd-0185-stake:
	@if [ -z "$(VOTE_ACCOUNT)" ]; then \
		echo "Error: VOTE_ACCOUNT is required"; \
		echo "Usage: make run-simd-0185-stake VOTE_ACCOUNT=<pubkey> [NETWORK=<network>]"; \
		exit 1; \
	fi
	cargo run --bin simd-0185-stake --features simd-0185/bin -- $(if $(NETWORK),$(NETWORK),localnet) $(VOTE_ACCOUNT)

test:
	cargo test $(addprefix -p ,helpers $(addsuffix -interface,$(PROGRAMS)))

test-sbf-%:
	cargo test-sbf --manifest-path $*/Cargo.toml

fmt:
	cargo +nightly fmt --all --check

fmt-fix:
	cargo +nightly fmt --all

clippy:
	cargo +nightly clippy --all-targets -- -D warnings

clean:
	cargo clean
