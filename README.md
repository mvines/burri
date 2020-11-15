Emit a self-transfer of a random amount, optionally including one or more additional
addresses in the transaction

## Quick Start
1. Install Rust from https://rustup.rs/
2. `cargo run -- --help`


## Basic usage:

```bash
while true; do \
  cargo run -- --keypair your_keypair.json --verbose; \
  sleep 60s; \
done
```

Run for as long as you'd like.

## Advanced usage

Same as "Basic" but the `2ke6K3igbrDh9Tr6AjaJq4BqbpRG42AjdiUb71cAksAm` and
`81aaynHWaKQChcp5Dw47dGxpK1fqzG9MGHgk7EJoz4uu` addresses are also referenced.

```bash
while true; do \
  cargo run -- --keypair your_keypair.json --verbose 2ke6K3igbrDh9Tr6AjaJq4BqbpRG42AjdiUb71cAksAm 81aaynHWaKQChcp5Dw47dGxpK1fqzG9MGHgk7EJoz4uu; \
  sleep 60s; \
done
```
