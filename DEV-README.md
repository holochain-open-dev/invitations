
![CI](https://github.com/holochain-engineers/invitations/actions/workflows/main.yml/badge.svg)

# Invitations Zome

## Installation and usage

### Including the zome in your DNA

1. Create a new `invitations` folder in the `zomes/coordinator` and `zomes/integrity` of the consuming DNA.
2. Add a `Cargo.toml` in both folders. In the content, paste the `Cargo.toml` content from any zome.
3. Change the `name` properties of the coordinator `Cargo.toml` file to `invitations` and invitations_integrity in the integrity folder
4. Add this zome as a dependency in the coordinator `Cargo.toml` file:

```toml 
[dependencies]
hc_coordinator_zome_invitations = {git = "https://github.com/holochain-engineers/invitations.git", branch = "crates-only"}
```
add this zome as a dependency in the integrity `Cargo.toml` file:
```toml 
[dependencies]
hc_integrity_zome_invitations = {git = "https://github.com/holochain-engineers/invitations.git", branch = "crates-only"}
```

5. Create a `src` folder besides each `Cargo.toml` with this content:

intergity:
```rust
extern crate hc_integrity_zome_invitations;
```

coordinator:
```rust
extern crate hc_coordinator_zome_invitations;
```

6. Add the zome into your `dna.yaml` file with the name `invitation`.
7. Compile the DNA with the usual `CARGO_TARGET_DIR=target cargo build --release --target wasm32-unknown-unknown`.




