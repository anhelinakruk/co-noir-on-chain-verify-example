cd noir
nargo compile

cargo run --package prover --bin party_0 & cargo run --package prover --bin party_1 & cargo run --package prover --bin party_2
bb write_solidity_verifier --scheme ultra_honk -k prover/verification_key -c noir/bn254 -o noir/target/Verifier.sol 

anvil --block-gas-limit 100000000 --gas-price 0 --code-size-limit 99999999999999

# PK: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

forge create noir/target/Verifier.sol:HonkVerifier \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast

# Deployed to: 0x5FbDB2315678afecb367f032d93F642f64180aa3

PROOF=0x$(xxd -p prover/proof | tr -d '\n') 
INPUTS=0x$(xxd -p prover/public_inputs | tr -d '\n') 

cast call 0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  "verify(bytes,bytes32[])" \
  "$PROOF" \
  "[$INPUTS]" \
  --rpc-url http://localhost:8545



