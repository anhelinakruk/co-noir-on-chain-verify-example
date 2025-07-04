# co-noir-on-chain-verify-example

## Project Structure

### `/noir/` - Noir Circuit Definition

Contains the Noir circuit implementation and related files

### `/prover/` - Rust Prover Implementation

Contains the Rust implementation for generating proofs

- `src/bin/party_0.rs` - Party 0 binary for MPC
- `src/bin/party_1.rs` - Party 1 binary for MPC
- `src/bin/party_2.rs` - Party 2 binary for MPC
- `data/` - TLS certificates and keys for secure MPC communication

### `/run.sh` - Execution Script

Shell script to run the complete proof generation process.
