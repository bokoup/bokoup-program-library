#!/bin/bash
# Change mint to address of local test solana wallet address
# Change bpf-program to be id from anchor program

BPF_PROGRAM=FtccGbN7AXvtqWP5Uf6pZ9xMdAyA7DXBmRQtmvjGDX7x
WALLET=61mVTaw6hBtwWnSaGXRSJePFWEQqipeCka3evytEVNUp

anchor build
cargo build -p bpl-indexer
solana-test-validator \
--reset \
--ledger .anchor/test-ledger \
--mint $WALLET \
--bind-address 0.0.0.0 \
--bpf-program $BPF_PROGRAM target/deploy/bpl_token_metadata.so \
--clone metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s \
--clone PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT \
--rpc-port 8899 \
--url https://api.devnet.solana.com \
--geyser-plugin-config indexer/Config.json