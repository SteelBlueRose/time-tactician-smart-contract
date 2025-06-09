## How to Build Locally?

1. Commit all the git changes;
2. use in wsl:

```bash
cargo near build
```

## How to Test Locally?

```bash
cargo test
```

## How to Deploy?

```bash
# 1::Delete account from MyNearWallet

# 2
near account delete-account <name>.testnet beneficiary alice.testnet network-config testnet sign-with-keychain send

# 3
near create-account <new_name>.testnet --useFaucet

# 4
near contract deploy <new_name>.testnet use-file ...\target\near\hello_near.wasm with-init-call new json-args {}

#5
near account export-account <new_name>.testnet using-web-wallet

```

## Call get-methods and transaction methods

```bash
near contract call-function as-read-only <new_name>.testnet

near contract call-function as-transaction <new_name>.testnet
```
