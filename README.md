# Test Programs

Test programs for feature activation stability testing.

Build all programs:

```
cargo build-sbf
```

**Deploy the desired program** before attempting tests.

```
solana program deploy target/deploy/program_name.so
```

You can check if the program was already deployed with:

Run test for SIMD-321 (for example):

```
cargo run --release simd-0321
```