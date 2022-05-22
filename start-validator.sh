#!/bin/bash
# Change mint to address of local test solana wallet address
# Change bpf-program to be id from anchor program

anchor build
cargo build -p bpl-indexer
solana-test-validator \
--reset \
--ledger .anchor/test-ledger \
--mint 61mVTaw6hBtwWnSaGXRSJePFWEQqipeCka3evytEVNUp \
--bind-address 0.0.0.0 \
--bpf-program \
    3rgtdHtt9gMsmcpjFQDzdFvU6BsuSjbb2oYcoy78kDQB \
    target/deploy/bpl_token_metadata.so \
--clone metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s \
--clone PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT \
--rpc-port 8899 \
--url https://cool-holy-violet.solana-devnet.quiknode.pro/eade67b5a902b1fcc01bd641b488e173fd279eae/ \
--geyser-plugin-config indexer/Config.json