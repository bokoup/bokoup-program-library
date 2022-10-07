#!/bin/sh

#Run using sol user

#Install the latest stable version of  solana-cli
case "$SOLANA_NETWORK" in
    #case 1
    "testnet") sh -c "$(curl -sSfL https://release.solana.com/beta/install)" ;;

    #case 2
    "devnet") sh -c "$(curl -sSfL https://release.solana.com/1.14.5/install)" ;;

    #case 3
    "mainnet-beta") sh -c "$(curl -sSfL https://release.solana.com/stable/install)" ;;
esac

#Add the installation path to the system $PATH
echo export PATH="/home/sol/.local/share/solana/install/active_release/bin:$PATH" >> /home/sol/.bashrc

#Apply above path to existing shell
export PATH="/home/sol/.local/share/solana/install/active_release/bin:$PATH"

#Check communication to other nodes on the network
solana gossip -u $SOLANA_NETWORK #Network options include mainnet-beta, devnet, testnet, localhost

#Set Solana target network (mainnet-beta, devnet, testnet)
solana config set --url https://api.$SOLANA_NETWORK.solana.com

#Create Authorized Withdrawer Keypair
solana-keygen new -o ~/authorized-withdrawer-keypair-$SOLANA_NETWORK.json #FEAT - NEEDS TO AUTOMATICALLY STORE KEYPAIR & SEEDPHRASE SECURELY IN VAULT

#Create Validator Keypair
solana-keygen new -o ~/validator-keypair-$SOLANA_NETWORK.json #FEAT - NEEDS TO AUTOMATICALLY STORE KEYPAIR & SEEDPHRASE SECURELY IN VAULT

#Create Vote Account Keypair
solana-keygen new -o ~/vote-account-keypair-$SOLANA_NETWORK.json #FEAT - NEEDS TO AUTOMATICALLY STORE KEYPAIR & SEEDPHRASE SECURELY IN VAULT

#Create Stake Account Keypair
solana-keygen new -o ~/stake-account-keypair-$SOLANA_NETWORK.json #FEAT - NEEDS TO AUTOMATICALLY STORE KEYPAIR & SEEDPHRASE SECURELY IN VAULT

#Configure Validator Keypair
solana config set --keypair ~/validator-keypair-$SOLANA_NETWORK.json #FEAT - NEEDS TO AUTOMATICALLY STORE KEYPAIR & SEEDPHRASE SECURELY IN VAULT

#ONLY FOR testnet/devnet (For Mainnet, you must deposit SOL into your Validator Wallet Address)
case "$SOLANA_NETWORK" in
    #case 1
    "testnet") solana airdrop 1 ;;

    #case 2
    "devnet") solana airdrop 1 ;;

    #case 3
    "mainnet-beta")  ;;  #FEAT - NEEDS TO SHOW VALIDATOR WALLET ADDRESS TO USER & PAUSE SCRIPT UNTIL SOL DEPOSIT COMPLETED
esac

#Create Vote Account (Requires SOL)
solana create-vote-account ~/vote-account-keypair-$SOLANA_NETWORK.json ~/validator-keypair-$SOLANA_NETWORK.json ~/authorized-withdrawer-keypair-$SOLANA_NETWORK.json

#Create validator starter script
mkdir -p /home/sol/bin
echo '#!/bin/sh

exec solana-validator \
--identity ~/validator-keypair-'$SOLANA_NETWORK'.json \
--vote-account ~/vote-account-keypair-'$SOLANA_NETWORK'.json \
--entrypoint entrypoint.'$SOLANA_NETWORK'.solana.com:8001 \
--entrypoint entrypoint2.'$SOLANA_NETWORK'.solana.com:8001 \
--entrypoint entrypoint3.'$SOLANA_NETWORK'.solana.com:8001 \' > ~/bin/validator-$SOLANA_NETWORK.sh
case "$SOLANA_NETWORK" in
    #case 1
    "testnet") echo '--known-validator 7DFs6SLK8JG5mJvFavtCzyRD3dbXBF88C2NfKBx3EfGe \
--known-validator 8QfmMtj7p7txWhCXSuFyaizb1Jq3LJRRKDv5XjYzmCoG \
--known-validator 7r5J4kASe9bCJwtENLR4333jbqBazBKg7YFu3yD4StQy \
--known-validator BTKGTVZ4xzkgGDHyVkvp3ouSFS4vAny8NqBEDgEA9uZT \
--known-validator GbALwBSEtrDQ967A2qfyPX2dVyxpvXD2rF2fGjnfmnK6 \' >> ~/bin/validator-$SOLANA_NETWORK.sh ;;

    #case 2
    "devnet") echo '--known-validator dv1ZAGvdsz5hHLwWXsVnM94hWf1pjbKVau1QVkaMJ92 \
--known-validator dv2eQHeP4RFrJZ6UeiZWoc3XTtmtZCUKxxCApCDcRNV \
--known-validator dv4ACNkpYPcE3aKmYDqZm9G5EB3J4MRoeE7WNDRBVJB \
--known-validator dv3qDFk1DTF36Z62bNvrCXe9sKATA6xvVy6A798xxAS \' >> ~/bin/validator-$SOLANA_NETWORK.sh ;;

    #case 3
    "mainnet-beta") echo '--known-validator 7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2 \
--known-validator GdnSyH3YtwcxFvQrVVJMm1JhTS4QVX7MFsX56uJLUfiZ \
--known-validator DE1bawNcRJB9rVm3buyMVfr8mBEoyyu73NBovf2oXJsJ \
--known-validator CakcnaRDHka2gXyfbEd2d3xsvkJkqsLw2akB3zsN1D2S \' >> ~/bin/validator-$SOLANA_NETWORK.sh ;;
esac

echo '--full-snapshot-interval-slots 10000 \
--maximum-full-snapshots-to-retain 2 \
--incremental-snapshot-interval-slots 500 \
--maximum-incremental-snapshots-to-retain 20 \
--maximum-local-snapshot-age 1000 \
--minimal-snapshot-download-speed 100000000 \
--ledger /mnt/solana-ledger \
--dynamic-port-range 8000-8020 \
--no-voting \
--rpc-port 8899 \
--rpc-bind-address 0.0.0.0 \
--private-rpc \
--full-rpc-api \
--tpu-use-quic \
--wal-recovery-mode skip_any_corrupted_record \
--log /var/log/sol/validator.log \
--accounts /mnt/solana-accounts \
--geyser-plugin-config /home/sol/config.json \
# ledger size will be set to 500GB by default
--limit-ledger-size \

# If validator works normally comment out this line to make restarts faster
# --no-port-check \

# Comment out if your machine has a GPU with CUDA
# --cuda' >> ~/bin/validator-$SOLANA_NETWORK.sh

chmod +x ~/bin/validator-$SOLANA_NETWORK.sh

case "$SOLANA_NETWORK" in
    #case 1
    "testnet") export SOLANA_METRICS_CONFIG="host=https://metrics.solana.com:8086,db=tds,u=testnet_write,p=c4fa841aa918bf8274e3e2a44d77568d9861b3ea" ;;

    #case 2
    "devnet") export SOLANA_METRICS_CONFIG="host=https://metrics.solana.com:8086,db=devnet,u=scratch_writer,p=topsecret" ;;

    #case 3
    "mainnet-beta") export SOLANA_METRICS_CONFIG="host=https://metrics.solana.com:8086,db=mainnet-beta,u=mainnet-beta_write,p=password" ;;
esac