
# Invitations Module

This module is intended for creating and coordinating invitation groups in a public DHT.  
Below is a **communication diagram (Sequence, Structure and State)** that shows the design of this module at a high level.

Structure: - It shows all the Object types stored in the system as well as DTO's (Data Transfer Objects) for input, ouput and listen DTO'S for signals.  
Sequence: - You can see the API for the module, and basic interaction between the conductor and the UI client  
State: - The most important state changes can be seen in the link tags and are followed up by appropriate signal variants.
- agent to invitation (pending,inviter,committed)
- invitation to agent (accepted,rejected)  

          
![image](https://github.com/holochain-open-dev/invitations/assets/17417820/241577ce-055d-4784-aff0-cc06f4d5e4cd)


in this version of the module, the author of the invitation is the only one that can make updates. They can also choose to be an invitee or not. If they choose to create an invitation that doesn't include them, they are linked by the "Inviter" tag otherise invitees are given a "pending" tag for the link.

status changes once the invitee chooses to accept or reject the invitation by
moving the agent link from "pending" to "committed" 

## Test the module without installation
run `nix develop`, `npm install` and `npm test` in the root folder of repository.

## Project Installation and usage

we assume you have an existing project, otherwise see instructions about scaffolding a web template here:
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

we specify a crates-only branch which uses the latest stable release of holochain. 
For older versions set the branch to crates-only-[ver] where [ver] is the version series eg 0.1x 
you toml dependencies should look something like this

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
