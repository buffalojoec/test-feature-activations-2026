# Test Programs

> [Full Makefile overview](#makefile)

### Keypairs

Program keypairs are **gitignored**. Generate one per program before building
binaries or deploying:

```sh
solana-keygen new -o simd-0185/keypair.json
solana-keygen new -o simd-0321/keypair.json
```

The client binaries read their program ID from `<prog>/keypair.json` at
runtime, so the keypair must exist even for local runs (`make run-*`).

## 🗳️ SIMD-0185: Vote State V4

Tests the `vote_state_v4` feature. The program creates a v4 vote account via
CPI to the Vote Program, then reads back and logs its fields.

### Build & deploy

```sh
make build-simd-0185
make deploy-simd-0185
make get-id-simd-0185
```

### Run on testnet

```sh
solana config set -u testnet
make run-simd-0185 NETWORK=testnet
```

The client sends a single transaction with two instructions:

* `Create` — initializes a v4 vote account (10% commission)
* `View` — reads back and logs the vote state fields

### Fetch v4 vote accounts

```sh
./scripts/fetch_vote_v4_accounts.sh testnet
```

Queries `getProgramAccounts` filtering for the v4 discriminator. Results are
saved to `scripts/out/vote_v4_accounts_testnet.txt`. Omit the argument to use
the current `solana config` RPC.

## 🧪 SIMD-0321: Instruction Data Pointer in VM r2

Tests the `provide_instruction_data_offset_in_vm_r2` feature, which passes
instruction data via the r2 register.

### Build & deploy

```sh
make build-simd-0321
make deploy-simd-0321
make get-id-simd-0321
```

### Run on testnet

```sh
solana config set -u testnet
make run-simd-0321 NETWORK=testnet
```

The client sends two instructions in a single transaction:

* Raw bytes (`0xDEADBEEF`) — logged as a byte array
* `EasterEgg` payload — triggers ASCII owl output

## Makefile

| Target | Description |
|---|---|
| `make list` | List all programs |
| `make build` | Build all programs |
| `make build-<prog>` | Build a single program |
| `make deploy-<prog>` | Deploy a program using its keypair |
| `make get-id-<prog>` | Get a program's address from its keypair |
| `make run-<prog>` | Run a program's client binary |
| `make run-<prog> NETWORK=<url>` | Run against a specific network |
| `make test` | Run unit tests (interfaces + helpers) |
| `make test-sbf-<prog>` | Run SBF tests for a program (requires `cargo-build-sbf`) |
| `make fmt` | Check formatting (requires nightly) |
| `make fmt-fix` | Fix formatting (requires nightly) |
| `make clippy` | Run clippy lints (requires nightly) |
| `make clean` | Clean build artifacts |

`<prog>` is a directory name, e.g. `simd-0321`.
