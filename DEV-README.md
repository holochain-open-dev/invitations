
# Invitations Module

## Test the module without installation
run `nix develop`, `npm install` and `npm test` in the root folder of repository.

## Project Installation and usage

we assume you have an existing project, otherwise see instrctions about scaffolding a web template here:
https://github.com/holochain-open-dev/templates

### Including the module in your project

This guide assumes you are inside a nix-shell environment with hc scaffold available.

1. Scaffold a new zome pair named profiles with:

```bash
hc scaffold zome invitations
```
Select the "Integrity/coordinator zome pair" option, and accept the path that the scaffolding tool offers to scaffold the zomes.

2. Add the hc_coordinator_zome_invitations and hc_integrity_zome_invitations zomes as dependencies with:

```bash
cargo add -p invitations hc_coordinator_zome_invitations
cargo add -p invitations_integrity hc_integrity_zome_invitations
```

3. Go into the newly scaffolded integrity zome's lib.rs (its path may be similar to dnas/project/zomes/integrity/invitations/src/lib.rs) and replace its contents with:

```rust
extern crate hc_integrity_zome_invitations;
```

4. Go into the newly scaffolded coordinator zome's lib.rs (its path may be similar to dnas/project/zomes/coordinator/invitations/src/lib.rs) and replace its contents with:

```rust
extern crate hc_coordinator_zome_invitations;
```

you toml dependencies should looking something like this

```toml 
[dependencies]
hc_coordinator_zome_invitations = {git = "https://github.com/holochain-engineers/invitations.git", branch = "crates-only"}
```
add this zome as a dependency in the integrity `Cargo.toml` file:
```toml 
[dependencies]
hc_integrity_zome_invitations = {git = "https://github.com/holochain-engineers/invitations.git", branch = "crates-only"}
```

5. check the coordinator and integrity zome details are in your `dna.yaml`.
6. Compile the DNA with the usual `CARGO_TARGET_DIR=target cargo build --release --target wasm32-unknown-unknown`.